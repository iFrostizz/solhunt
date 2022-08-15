// A silly module that finds all uint256

use core::{
    loader::{DynModule, Module},
    walker::{Finding, Severity},
};
use ethers_solc::artifacts::ast::{ContractDefinitionPart, SourceUnitPart};

pub fn get_module() -> DynModule {
    Module::new(
        "uint256",
        Box::new(move |source, _info| {
            let mut findings: Vec<Finding> = Vec::new();

            if let SourceUnitPart::ContractDefinition(def) = source {
                def.nodes.iter().for_each(|node| {
                    if let ContractDefinitionPart::VariableDeclaration(var) = node {
                        if let Some(type_id) = &var.type_descriptions.type_identifier {
                            if type_id == "t_uint256" {
                                findings.push(Finding {
                                    name: "uint256".to_string(),
                                    description: "We just found a uint256 yay!".to_string(),
                                    severity: Severity::Informal,
                                    code: 0,
                                });
                            }
                        }
                    }
                })
            }

            findings
        }),
    )
}
