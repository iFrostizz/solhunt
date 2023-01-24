// A silly module that finds all uint256

use crate::{
    loader::{DynModule, Module},
    walker::{Finding, Severity},
    ModuleFindings,
};
use ethers_solc::artifacts::{
    ast::{ContractDefinitionPart, SourceUnitPart},
    visitor::VisitError,
    visitor::Visitor,
    VariableDeclaration,
};

#[derive(Default)]
pub struct Uint256Module(ModuleFindings);

impl Visitor for Uint256Module {
    type Error = VisitError;

    fn visit_variable_declaration(
        &mut self,
        var: &mut VariableDeclaration,
    ) -> eyre::Result<(), Self::Error> {
        if let Some(type_id) = &var.type_descriptions.type_identifier {
            if type_id == "t_uint256" {
                self.0.findings.push(Finding {
                    name: "uint256".to_string(),
                    description: "We just found a uint256 yay!".to_string(),
                    severity: Severity::Informal,
                    src: Some(var.src.clone()),
                    code: 0,
                });
            }
        }
        Ok(())
    }
}

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
                                    src: Some(var.src.clone()),
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

#[cfg(test)]
mod test {
    use crate::{
        solidity::ProjectFile,
        test::{compile_and_get_findings, has_with_code},
    };

    #[test]
    fn can_find_dummy_uint256() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("DummyUint256"),
            String::from(
                "pragma solidity 0.8.0;
            contract DummyUint256 {
                uint256 unint;
            }
            ",
            ),
        )]);

        assert!(has_with_code(&findings, "uint256", 0));
    }
}
