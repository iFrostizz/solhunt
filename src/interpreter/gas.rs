use revm::{
    db::{CacheDB, EmptyDB, InMemoryDB},
    primitives::{Bytes, TransactTo, B160, U256},
    EVM,
};

pub struct GasComparer {
    contracts: (String, String),
    from: B160,
    value: U256,
    data: Bytes,
    evm: EVM<CacheDB<EmptyDB>>,
}

impl GasComparer {
    #[cfg(test)]
    pub fn new(
        contract_from: String,
        contract_to: String,
        from: B160,
        data: Bytes,
        value: U256,
    ) -> Self {
        let cache_db = InMemoryDB::default();
        let mut evm = EVM::new();
        evm.database(cache_db);

        Self {
            contracts: (contract_from, contract_to),
            from,
            data,
            value,
            evm,
        }
    }

    /// Run the gas metering on the "from" and the "to" contract
    /// Returns (from, to) gas usage
    #[cfg(test)]
    pub fn run(&mut self) -> (u64, u64) {
        let gas_from = self.gas_meter(self.contracts.0.clone());
        let gas_to = self.gas_meter(self.contracts.1.clone());

        (gas_from, gas_to)
    }

    /// Deploys a contract and runs a call to it, return the used gas
    #[cfg(test)]
    pub fn gas_meter(&mut self, contract: String) -> u64 {
        use revm::primitives::ExecutionResult;

        let addr = self.deploy(contract);

        self.evm.env.tx.caller = self.from;
        self.evm.env.tx.transact_to = TransactTo::Call(addr);
        self.evm.env.tx.data = self.data.clone();
        self.evm.env.tx.value = self.value;

        let exec = self.evm.transact_commit().unwrap();

        if let ExecutionResult::Success { gas_used, .. } = exec {
            gas_used - 21000
        } else {
            panic!("gas metering failed!");
        }
    }

    // TODO: better error handling
    /// Compiles and deploys a contract from source and return the address
    #[cfg(test)]
    pub fn deploy(&mut self, contract: String) -> B160 {
        use revm::primitives::{CreateScheme, ExecutionResult, Output};

        use crate::test::compile_single_contract;

        let bytecode = compile_single_contract(contract);

        self.evm.env.tx.caller = self.from;
        self.evm.env.tx.data = bytecode.into();
        self.evm.env.tx.transact_to = TransactTo::Create(CreateScheme::Create);

        let exec = self.evm.transact_commit().unwrap();

        if let ExecutionResult::Success {
            output: Output::Create(_, create),
            ..
        } = exec
        {
            return create.unwrap();
        }

        panic!("Contract not deployed");
    }
}
