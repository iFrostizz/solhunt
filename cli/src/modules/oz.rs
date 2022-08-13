// Find vulns from oz security reports
// https://github.com/OpenZeppelin/openzeppelin-contracts/security/advisories

use core::{
    loader::{Information, Module},
    walker::Finding,
};
use ethers_solc::artifacts::ast::Node;

pub fn get_module() -> Module<Box<dyn Fn(&Node, &Information) -> Option<Finding>>> {
    Module::new("oz", Box::new(|_node, _info| None))
}
