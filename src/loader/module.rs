// A module contains the matching logic to be paired with the ast

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
    pub src: Option<SourceLocation>,
    pub code: usize,
}

// TODO: automate this !
// TODO: write benches for detection modules and only run *one* visitor
pub fn get_all_visitors() -> Vec<Box<(dyn Visitor<ModuleState> + 'static)>> {
    // Vec::new()
    vec![
        Box::<high::calls::DetectionModule>::default(),
        Box::<medium::assembly::DetectionModule>::default(),
        Box::<medium::overflow::DetectionModule>::default(),
        Box::<medium::chainlink::DetectionModule>::default(),
        Box::<medium::centralization::DetectionModule>::default(),
        Box::<medium::encode_packed::DetectionModule>::default(),
        Box::<medium::proxy::DetectionModule>::default(),
        Box::<low::misc::DetectionModule>::default(),
        Box::<low::erc20::DetectionModule>::default(),
        Box::<info::style::DetectionModule>::default(),
        Box::<gas::address_zero::DetectionModule>::default(),
        Box::<gas::tree::DetectionModule>::default(),
        Box::<gas::tight_pack::DetectionModule>::default(),
        Box::<gas::immutable::DetectionModule>::default(),
        Box::<gas::state::DetectionModule>::default(),
        Box::<gas::custom_errors::DetectionModule>::default(),
        Box::<gas::constructor::DetectionModule>::default(),
        Box::<oz::DetectionModule>::default(),
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
