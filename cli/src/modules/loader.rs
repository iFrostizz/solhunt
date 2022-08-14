use crate::modules::{erc20, overflow, oz, style, uint256};
use core::loader::DynModule;

pub fn get_all_modules() -> Vec<DynModule> {
    vec![
        erc20::get_module(),
        overflow::get_module(),
        oz::get_module(),
        style::get_module(),
        uint256::get_module(),
    ]
}
