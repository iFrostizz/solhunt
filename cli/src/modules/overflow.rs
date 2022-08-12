// Check if overflow may occur in unchecked or < 0.8.0 versions of solc

use core::{loader::Module, walker::Finding};
use ethers_solc::artifacts::ast::{Node, NodeType};
use semver::{Version, VersionReq};
use serde_json::from_value;

pub fn get_module(version: Version) -> Module<impl (Fn(&Node) -> Option<Finding>)> {
    Module::new("overflow", move |node| {
        /*dbg!(&node);
        match node.node_type {
            _ => None,
        }*/

        /*match node.other.get("kind") {
            Some(kind) => match kind {
                serde_json::value::Value::String(kind) => {
                    if kind == "function" {
                        // We investigate overflow in functions
                        // dbg!(&node);
                        // TODO: nodeType = String("Assignement")
                        // TODO: operator = String("+=")
                        if let Some(body) = &node.body {
                            // Function has stuff inside
                            // "body" is a Node
                            match body.other.get("statements") {
                                Some(statements) => {
                                    dbg!(&statements);
                                }
                                _ => (),
                            }
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        }*/

        None
    })
}

/*fn extract_version(node: &Node) -> Version {
    let literals = node.other.get("literals").unwrap();
    let mut lit_vec: Vec<String> = from_value(literals.clone()).unwrap();
    lit_vec.remove(0);
    let cleaned = lit_vec.join("");

    Version::parse(&cleaned).expect("Failed to parse version")
}*/
