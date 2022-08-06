// Find vulns from oz security reports
// https://github.com/OpenZeppelin/openzeppelin-contracts/security/advisories

use core::{loader::Module, walker::Finding};
use ethers_solc::artifacts::ast::Node;

pub fn get_module() -> Module<impl (Fn(&Node) -> Option<Finding>)> {
    Module::new("oz", |node| None)
}
