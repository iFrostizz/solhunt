// Make sure that the code style is good, e.g.
// No remaining TODOs: https://code4rena.com/reports/2022-06-badger/#n-02-open-todos
// hardhat's console.log

use core::{loader::Module, walker::Finding};
use ethers_solc::artifacts::ast::Node;

pub fn get_module() -> Module<impl (Fn(&Node) -> Option<Finding>)> {
    Module::new("style", |node| None)
}
