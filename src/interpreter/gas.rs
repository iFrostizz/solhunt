use crate::solidity::{compile_single_contract, compile_single_contract_to_artifacts_path};
use ethers_contract::BaseContract;
use ethers_core::abi::parse_abi;
use ethers_solc::artifacts::BytecodeObject;
use ethers_solc::ConfigurableContractArtifact;
use eyre::ContextCompat;
use revm::primitives::{CreateScheme, ExecutionResult, Output};
use revm::{
    db::{CacheDB, EmptyDB, InMemoryDB},
    primitives::{Bytes, TransactTo, B160, U256},
    EVM,
};
use std::path::PathBuf;

pub struct GasComparer {
    /// location of the contract to meter
    location: PathBuf,
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
            location: Default::default(),
            from: Default::default(),
            value: Default::default(),
            data,
            evm: Default::default(),
        }
    }
}

impl From<PathBuf> for GasComparer {
    fn from(path: PathBuf) -> Self {
        Self {
            location: path,
            ..Default::default()
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

    /// Run the gas metering on the "from" and the "to" contract
    /// Returns (from, to) gas usage
    pub fn run(&mut self) -> eyre::Result<(u64, u64)> {
        let cache_db = InMemoryDB::default();
        self.evm.database(cache_db);

        let artifacts = compile_single_contract_to_artifacts_path(self.location.clone())?;

        let mut art_from = None;
        let mut art_to = None;

        artifacts.iter().for_each(|(id, artifact)| {
            if id.source == self.location {
                if id.name == "From" {
                    art_from = Some(artifact);
                } else if id.name == "To" {
                    art_to = Some(artifact);
                }
            }
        });

        let gas_from = self.gas_meter(art_from.wrap_err("No `From` contract")?)?;
        let gas_to = self.gas_meter(art_to.wrap_err("No `To` contract")?)?;

        Ok((gas_from, gas_to))
    }

    /// Deploys a contract and runs a call to it, return the used gas
    pub fn gas_meter(&mut self, artifact: &ConfigurableContractArtifact) -> eyre::Result<u64> {
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
        artifact: &ConfigurableContractArtifact,
    ) -> eyre::Result<Bytes> {
        if let BytecodeObject::Bytecode(bytecode) = &artifact.bytecode.as_ref().unwrap().object {
            Ok(bytecode.to_vec().into())
        } else {
            eyre::bail!("No bytecode found");
        }
    }
}
