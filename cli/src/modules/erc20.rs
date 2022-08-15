// Check for non-compliant code (e.g:
// Using safeApprove instead of ... : https://code4rena.com/reports/2022-06-badger/#n-01-safeapprove-is-deprecated

use core::loader::{DynModule, Module};

pub fn get_module() -> DynModule {
    Module::new("erc20", Box::new(|_node, _info| vec![]))
}
