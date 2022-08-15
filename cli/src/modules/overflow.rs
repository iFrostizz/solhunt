// Check if overflow may occur in unchecked or < 0.8.0 versions of solc

use core::{
    loader::{DynModule, Module},
    walker::{Finding, Severity},
};
use ethers_solc::artifacts::ast::SourceUnitPart;

pub fn get_module() -> DynModule {
    Module::new(
        "overflow",
        Box::new(|_node, _info| {
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
                                    Some(_statements) => {
                                        // dbg!(&statements);
                                    }
                                    _ => (),
                                }
                                if info.version.minor < 8 {
                                    Some(Finding {
                                        name: "overflow".to_string(),
                                        description:
                                            "the function may overflow, please bump version > 0.8.0"
                                                .to_string(),
                                        severity: Severity::Low,
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
                    }
                    _ => None,
                },
                _ => None,
            }*/
            vec![]
        }),
    )
}
