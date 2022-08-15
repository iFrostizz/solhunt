// Find vulns from oz security reports
// https://github.com/OpenZeppelin/openzeppelin-contracts/security/advisories

use core::{
    loader::{DynModule, Module},
};

pub fn get_module() -> DynModule {
    Module::new("oz", Box::new(|_node, _info| vec![]))
}
