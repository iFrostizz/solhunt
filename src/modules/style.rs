// Make sure that the code style is good, e.g.
// No remaining TODOs: https://code4rena.com/reports/2022-06-badger/#n-02-open-todos
// hardhat's console.log

use crate::loader::{DynModule, Module};

pub fn get_module() -> DynModule {
    Module::new("style", Box::new(|_node, _info| vec![]))
}
