// A silly module that finds all uint256

use core::{
    loader::{Information, Module},
    walker::{Finding, Severity},
};
use ethers_solc::artifacts::ast::{Node, NodeType};

pub fn get_module() -> Module<Box<dyn Fn(&Node, &Information) -> Option<Finding>>> {
    Module::new(
        "uint256",
        Box::new(move |node, _info| {
            if let NodeType::VariableDeclaration = node.node_type {
                let type_name = node
                    .other
                    .get("typeName")
                    .expect("no typeName node in VariableDeclaration")
                    .clone();
                if let Some(type_descriptions) = type_name.get("typeDescriptions") {
                    let type_identifier = type_descriptions
                        .get("typeIdentifier")
                        .expect("No typeIdentifier");
                    if type_identifier == "t_uint256" {
                        let name = "uint256".to_string();
                        let description = "We just found a uint256 yay!".to_string();

                        // Detect an informal finding for fun, just fiddling...
                        Some(Finding {
                            name,
                            description,
                            severity: Severity::Informal,
                            code: 0,
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
        }),
    )
}
