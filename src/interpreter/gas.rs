use crate::{
    solidity::{compile_single_contract_to_artifacts_path, equi_ver},
    walker::{ModuleState, Walker},
};
use ethers_contract::BaseContract;
use ethers_core::abi::parse_abi;
use ethers_solc::{
    artifacts::{
        visitor::{VisitError, Visitable, Visitor},
        BytecodeObject, PragmaDirective,
    },
    ArtifactId, ConfigurableContractArtifact,
};
use eyre::ContextCompat;
use revm::{
    db::{CacheDB, EmptyDB, InMemoryDB},
    primitives::{Bytes, CreateScheme, ExecutionResult, Output, TransactTo, B160, U256},
    EVM,
};
use semver::Version;
use std::{collections::BTreeMap, path::PathBuf};

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
        }
    }
}

impl GasComparer {
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

    /// Run the gas metering on the "from" and the "to" contract
    /// Returns (from, to) gas usage
    pub fn run(&mut self) -> eyre::Result<(u64, u64)> {
        let cache_db = InMemoryDB::default();
        self.evm.database(cache_db);

        let artifacts =
            compile_single_contract_to_artifacts_path(self.location.clone(), self.version.clone())?;

        let mut art_from = None;
        let mut art_to = None;

        artifacts.into_iter().for_each(|(id, artifact)| {
            if id.source == self.location {
                if id.name == "From" {
                    art_from = Some(BTreeMap::from([(id, artifact)]));
                } else if id.name == "To" {
                    art_to = Some(BTreeMap::from([(id, artifact)]));
                }
            }
        });

        let art_from = art_from.wrap_err("No `From` contract")?;
        let art_to = art_to.wrap_err("No `To` contract")?;

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
        let bytecode = self.check_compliance(artifact)?;

        let addr = self.deploy(bytecode)?;

        if addr.is_zero() {
            eyre::bail!("deployment failed")
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

    // TODO: better error handling
    /// Compiles and deploys metering contracts from location and return the addresses
    pub fn deploy(&mut self, bytecode: Bytes) -> eyre::Result<B160> {
        self.evm.env.tx.caller = self.from;
        self.evm.env.tx.data = bytecode;
        self.evm.env.tx.transact_to = TransactTo::Create(CreateScheme::Create);

        let exec = self.evm.transact_commit().unwrap();

        if let ExecutionResult::Success {
            output: Output::Create(_, create),
            ..
        } = exec
        {
            create.ok_or(eyre::eyre!("deployment failed"))
        } else {
            eyre::bail!("deployment failed with result: `{:#?}`", exec);
        }
    }

    /// applies some sanity checks to make sure of the integrity of the gas metering contract. See the README.md for more informations
    pub fn check_compliance(
        &mut self,
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    ) -> eyre::Result<Bytes> {
        let module: ComplianceModule = ComplianceModule {
            location: self.location.clone(),
            ..Default::default()
        };

        let mut walker = Walker::new(
            artifact.clone(),
            BTreeMap::new(),
            vec![Box::from(module)],
            self.root.clone(),
        );

        walker.traverse()?;

        let artifact = artifact.values().next().unwrap();

        if let BytecodeObject::Bytecode(bytecode) = &artifact.bytecode.as_ref().unwrap().object {
            Ok(bytecode.to_vec().into())
        } else {
            eyre::bail!("No bytecode found");
        }
    }
}

#[derive(Default)]
pub struct ComplianceModule {
    /// amount of pragma definitions, should be only one
    pragma_dir: usize,
    location: PathBuf,
    shared_data: ModuleState,
}

impl ComplianceModule {
    #[cfg(test)]
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
}
