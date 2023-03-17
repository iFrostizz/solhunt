use crate::{
    solidity::equi_ver,
    walker::{ModuleState, Walker},
};
use ethers_contract::BaseContract;
use ethers_core::abi::parse_abi;
use ethers_solc::{
    artifacts::{
        visitor::{VisitError, Visitable, Visitor},
        BytecodeObject, FunctionDefinition, FunctionKind, PragmaDirective,
    },
    ArtifactId, ConfigurableContractArtifact,
};
use revm::{
    db::{CacheDB, EmptyDB, InMemoryDB},
    primitives::{Bytes, CreateScheme, ExecutionResult, Output, TransactTo, B160, U256},
    EVM,
};
use semver::Version;
use std::{cell::RefCell, collections::BTreeMap, ops::Deref, path::PathBuf, rc::Rc};

pub struct GasComparer {
    /// root of the metering project
    root: PathBuf,
    /// location of the contract to meter
    location: PathBuf,
    /// solc version
    version: Version,
    /// sender of the transaction
    from: B160,
    /// value of the tx
    value: U256,
    /// data of the tx
    data: Bytes,
    /// evm environment
    evm: EVM<CacheDB<EmptyDB>>,
    /// ("from", "to") artifacts
    artifacts: Option<(
        BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    )>,
}

#[derive(Eq, PartialEq)]
pub enum MeteringKind {
    Deploy,
    Call,
}

impl Default for GasComparer {
    fn default() -> Self {
        let abi = BaseContract::from(parse_abi(&["function gasMeter() external"]).unwrap());
        let data = abi.encode("gasMeter", ()).unwrap().0;

        Self {
            root: Default::default(),
            location: Default::default(),
            version: Version::new(0, 5, 0),
            from: Default::default(),
            value: Default::default(),
            data,
            evm: Default::default(),
            artifacts: Default::default(),
        }
    }
}

impl GasComparer {
    #[allow(unused)]
    pub fn new(location: PathBuf, from: B160, data: Bytes, value: U256) -> Self {
        Self {
            location,
            from,
            data,
            value,
            ..Default::default()
        }
    }

    pub fn with_root(mut self, root: PathBuf) -> Self {
        self.root = root;
        self
    }

    pub fn with_location(mut self, location: PathBuf) -> Self {
        self.location = location;
        self
    }

    pub fn with_version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    pub fn with_artifacts(
        mut self,
        artifacts: (
            BTreeMap<ArtifactId, ConfigurableContractArtifact>,
            BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        ),
    ) -> Self {
        self.artifacts = Some(artifacts);
        self
    }

    /// Run the gas metering on the "from" and the "to" contract
    /// Returns (from, to) gas usage
    pub fn run(&mut self) -> eyre::Result<(u64, u64)> {
        let cache_db = InMemoryDB::default();
        self.evm.database(cache_db);

        // let artifacts =
        //     compile_single_contract_to_artifacts_path(self.location.clone(), self.version.clone())?;

        // let mut art_from = None;
        // let mut art_to = None;

        // self.artifacts.into_iter().for_each(|(id, artifact)| {
        //     if id.source == self.location {
        //         if id.name == "From" {
        //             art_from = Some(BTreeMap::from([(id, artifact)]));
        //         } else if id.name == "To" {
        //             art_to = Some(BTreeMap::from([(id, artifact)]));
        //         }
        //     }
        // });

        let artifacts = self
            .artifacts
            .clone()
            .ok_or(eyre::eyre!("No from / to artifacts"))?;

        let (art_from, art_to) = (artifacts.0, artifacts.1);

        // make sure that compiled versions are matching requested one
        let (ver_from, ver_to) = (
            &art_from.keys().next().unwrap().version,
            &art_to.keys().next().unwrap().version,
        );

        if !equi_ver(ver_from, &self.version) || !equi_ver(ver_to, &self.version) {
            eyre::bail!("requested solc version is not matching in artifacts");
        }

        let gas_from = self.gas_meter(art_from)?;
        let gas_to = self.gas_meter(art_to)?;

        Ok((gas_from, gas_to))
    }

    /// Deploys a contract and runs a call to it, return the used gas
    pub fn gas_meter(
        &mut self,
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    ) -> eyre::Result<u64> {
        let (bytecode, kind) = self.check_compliance(artifact)?;

        let (addr, gas_used) = self.deploy(bytecode)?;

        if addr.is_zero() {
            eyre::bail!("deployment failed")
        }

        if kind == MeteringKind::Deploy {
            return Ok(gas_used);
        }

        self.evm.env.tx.caller = self.from;
        self.evm.env.tx.transact_to = TransactTo::Call(addr);
        self.evm.env.tx.data = self.data.clone();
        self.evm.env.tx.value = self.value;

        let exec = self.evm.transact_commit().unwrap();

        if let ExecutionResult::Success { gas_used, .. } = exec {
            // stipend the tx base price
            Ok(gas_used - 21000)
        } else {
            eyre::bail!("function call failed: {:#?}", exec);
        }
    }

    /// Compiles and deploys metering contracts from location and return the addresses
    pub fn deploy(&mut self, bytecode: Bytes) -> eyre::Result<(B160, u64)> {
        self.evm.env.tx.caller = self.from;
        self.evm.env.tx.data = bytecode;
        self.evm.env.tx.transact_to = TransactTo::Create(CreateScheme::Create);

        let exec = self.evm.transact_commit().unwrap();

        if let ExecutionResult::Success {
            gas_used,
            output: Output::Create(_, create),
            ..
        } = exec
        {
            let addr = create.ok_or(eyre::eyre!("deployment failed"))?;

            Ok((addr, gas_used))
        } else {
            eyre::bail!("deployment failed with result: `{:#?}`", exec);
        }
    }

    /// applies some sanity checks to make sure of the integrity of the gas metering contract. See the README.md for more informations
    pub fn check_compliance(
        &mut self,
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    ) -> eyre::Result<(Bytes, MeteringKind)> {
        let module: Rc<RefCell<dyn Visitor<ModuleState>>> =
            Rc::from(RefCell::from(ComplianceModule {
                location: self.location.clone(),
                ..Default::default()
            }));

        let visitors: Vec<Rc<RefCell<dyn Visitor<ModuleState>>>> = vec![Rc::clone(&module)];

        let mut walker = Walker::new(artifact.clone(), BTreeMap::new(), visitors);

        walker.traverse()?;

        let mut_mod = module.borrow_mut();
        let der_mod = mut_mod.deref();
        let comp_mod = der_mod.as_any().downcast_ref::<ComplianceModule>().unwrap();

        let kind = if comp_mod.has_constructor {
            MeteringKind::Deploy
        } else {
            MeteringKind::Call
        };

        let artifact = artifact.values().next().unwrap();

        if let BytecodeObject::Bytecode(bytecode) = &artifact.bytecode.as_ref().unwrap().object {
            Ok((bytecode.to_vec().into(), kind))
        } else {
            eyre::bail!("No bytecode found");
        }
    }
}

#[derive(Default, Clone)]
pub struct ComplianceModule {
    /// amount of pragma definitions, should be only one
    pragma_dir: usize,
    location: PathBuf,
    has_constructor: bool,
    shared_data: ModuleState,
}

impl ComplianceModule {
    #[allow(unused)]
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Visitor<ModuleState> for ComplianceModule {
    fn shared_data(&mut self) -> &ModuleState {
        &self.shared_data
    }

    fn visit_source_unit(
        &mut self,
        source_unit: &mut ethers_solc::artifacts::SourceUnit,
    ) -> eyre::Result<(), VisitError> {
        source_unit.visit(self)?;

        if self.pragma_dir != 1 {
            return Err(VisitError::MsgError(format!(
                "err for location `{:#?}` should have only one pragma directive !",
                self.location,
            )));
        }

        Ok(())
    }

    fn visit_pragma_directive(
        &mut self,
        pragma_directive: &mut PragmaDirective,
    ) -> eyre::Result<(), VisitError> {
        self.pragma_dir += 1;

        pragma_directive.visit(self)
    }

    fn visit_function_definition(
        &mut self,
        function_definition: &mut FunctionDefinition,
    ) -> eyre::Result<(), VisitError> {
        if function_definition.kind == Some(FunctionKind::Constructor) {
            self.has_constructor = true;
        }

        function_definition.visit(self)
    }
}
