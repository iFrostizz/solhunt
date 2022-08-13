// Check for non-compliant code (e.g:
// Using safeApprove instead of ... : https://code4rena.com/reports/2022-06-badger/#n-01-safeapprove-is-deprecated

use core::{
    loader::{Information, Module},
    walker::Finding,
};
use ethers_solc::artifacts::ast::Node;

pub fn get_module() -> Module<Box<dyn Fn(&Node, &Information) -> Option<Finding>>> {
    Module::new(
        "erc20",
        Box::new(|_node, _info| {
            None
            /*if let NodeType::VariableDeclaration = node.node_type {
                let type_name = node.other.get("typeName").unwrap().clone();
                if let Some(type_descriptions) = type_name.get("typeDescriptions") {
                    let type_identifier = type_descriptions.get("typeIdentifier").expect("No typeIdentifier");
                    if type_identifier == "t_uint256" {
                        // println!("{} {}", self.name, self.matching);
                        let name = "uint256".to_string();
                        let description = "We just found a uint256 yay!".to_string();

                        // Detect an informal finding for fun, just fiddling...
                        Some(Finding {
                            name,
                            description,
                            severity: Severity::Informal,
                            code: 1,
                        })
                    } else { None }
                } else { None }
            } else { None }*/
        }),
    )
}
