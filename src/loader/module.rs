// A module contains the matching logic to be paired with the ast

use std::{cell::RefCell, rc::Rc};

use crate::{
    modules::*,
    walker::{AllFindings, Finding, ModuleState},
};
use ethers_solc::artifacts::{ast::SourceLocation, visitor::Visitor};
use semver::Version;

#[derive(Debug, Default, Clone)]
pub struct ModuleFindings {
    pub name: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug)]
pub struct Module<F> {
    pub name: String,
    pub findings: AllFindings,
    pub func: F,
}

#[derive(Debug, Clone)]
pub struct Information {
    pub name: String,
    pub version: Version,
}

#[derive(Debug)]
pub struct PushedFinding {
    pub code: usize,
    pub src: Option<SourceLocation>,
}

// TODO: automate this !
// TODO: write benches for detection modules and only run *one* visitor
pub fn get_all_visitors() -> Vec<Rc<RefCell<(dyn Visitor<ModuleState> + 'static)>>> {
    // Vec::new()
    vec![
        Rc::from(RefCell::from(high::calls::DetectionModule::default())),
        Rc::from(RefCell::from(medium::assembly::DetectionModule::default())),
        Rc::from(RefCell::from(medium::overflow::DetectionModule::default())),
        Rc::from(RefCell::from(medium::chainlink::DetectionModule::default())),
        Rc::from(RefCell::from(
            medium::centralization::DetectionModule::default(),
        )),
        Rc::from(RefCell::from(
            medium::encode_packed::DetectionModule::default(),
        )),
        Rc::from(RefCell::from(medium::proxy::DetectionModule::default())),
        Rc::from(RefCell::from(low::misc::DetectionModule::default())),
        Rc::from(RefCell::from(low::erc20::DetectionModule::default())),
        Rc::from(RefCell::from(info::style::DetectionModule::default())),
        Rc::from(RefCell::from(gas::address_zero::DetectionModule::default())),
        Rc::from(RefCell::from(gas::tree::DetectionModule::default())),
        Rc::from(RefCell::from(gas::tight_pack::DetectionModule::default())),
        Rc::from(RefCell::from(gas::immutable::DetectionModule::default())),
        Rc::from(RefCell::from(gas::state::DetectionModule::default())),
        Rc::from(RefCell::from(gas::require::DetectionModule::default())),
        Rc::from(RefCell::from(gas::constructor::DetectionModule::default())),
        Rc::from(RefCell::from(gas::condition::DetectionModule::default())),
        Rc::from(RefCell::from(oz::DetectionModule::default())),
    ]
}

// TODO: does not work
// #[macro_export]
// macro_rules! get_visitors {
//     () => {
//         let visitors: Vec<Box<(dyn Visitor<Vec<Finding>> + 'static)>> = Vec::new();

//         for entry in fs::read_dir($crate::modules).unwrap() {
//             let entry = entry.unwrap();
//             let path = entry.path();
//             if path.is_file() {
//                 let file_name = path.file_name().unwrap().to_str().unwrap();
//                 if file_name.ends_with(".rs") && file_name != "mod.rs" {
//                     visitors.push($directory::DetectionModule::default());
//                 }
//             }
//         }
//     };
// }
