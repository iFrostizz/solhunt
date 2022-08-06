// A silly module that finds all uint256

use core::{
    loader::Module,
    walker::{Finding, Severity},
};
use ethers_solc::artifacts::ast::{Node, NodeType};

pub fn get_module() -> Module<impl (Fn(&Node) -> Option<Finding>)> {
    Module::new("uint256", |node| {
        if let NodeType::VariableDeclaration = node.node_type {
            let type_name = node.other.get("typeName").unwrap().clone();
            if let Some(type_descriptions) = type_name.get("typeDescriptions") {
                let type_identifier = type_descriptions
                    .get("typeIdentifier")
                    .expect("No typeIdentifier");
                if type_identifier == "t_uint256" {
                    // println!("{} {}", self.name, self.matching);
                    let name = "uint256".to_string();
                    let description = "We just found a uint256 yay!".to_string();

                    // Detect an informal finding for fun, just fiddling...
                    Some(Finding {
                        name,
                        description,
                        severity: Severity::Informal,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    })
}
