// A module contains the matching logic to be paired with the ast

use crate::{
    modules::*,
    walker::{AllFindings, Finding},
};

use ethers_solc::artifacts::ast::SourceLocation;
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

// #[macro_export]
// macro_rules! get_all_visitors {
//     ($directory:expr) => {
//         // ($obj:ty, ($_foo:ident, $_bar:ident, $_baz:ident), $body:expr) => {
//         fn get_all_visitors(
//         ) -> Vec<Box<(dyn ethers_solc::artifacts::visitor::Visitor<Vec<Finding>> + 'static)>> {
//             let mut visitors = Vec::new();

//             for entry in fs::read_dir($directory).unwrap() {
//                 let entry = entry.unwrap();
//                 let path = entry.path();
//                 if path.is_file() {
//                     let file_name = path.file_name().unwrap().to_str().unwrap();
//                     if file_name.ends_with(".rs") && file_name != "mod.rs" {
//                         // let struct_name = &file_name[..file_name.len() - 3];
//                         // println!("importing struct {} from file {:?}", struct_name, file_name);

//                         visitors.push(Box::<$directory::DetectionModule>::default());
//                     }
//                 }
//             }
//         }
//     };
// }

// TODO: automate this !
// TODO: write benches for detection modules and only run *one* visitor
pub fn get_all_visitors(
) -> Vec<Box<(dyn ethers_solc::artifacts::visitor::Visitor<Vec<Finding>> + 'static)>> {
    // Vec::new()
    vec![
        // Box::<uint256::DetectionModule>::default(),
        Box::<assembly::DetectionModule>::default(),
        Box::<calls::DetectionModule>::default(),
        Box::<erc20::DetectionModule>::default(),
        Box::<overflow::DetectionModule>::default(),
        Box::<oz::DetectionModule>::default(),
        Box::<chainlink::DetectionModule>::default(),
        Box::<style::DetectionModule>::default(),
        Box::<centralization::DetectionModule>::default(),
        Box::<address_zero::DetectionModule>::default(),
        Box::<encode_packed::DetectionModule>::default(),
        Box::<misc::DetectionModule>::default(),
        Box::<proxy::DetectionModule>::default(),
    ]
}
