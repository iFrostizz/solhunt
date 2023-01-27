// A silly module that finds all uint256

use crate::walker::{Finding, Severity};
use ethers_solc::artifacts::{
    visitor::{VisitError, Visitor},
    VariableDeclaration,
};

#[derive(Default)]
pub struct DetectionModule {
    findings: Vec<Finding>,
}

impl Visitor<Vec<Finding>> for DetectionModule {
    fn shared_data(&mut self) -> &Vec<Finding> {
        &self.findings
    }

    fn visit_variable_declaration(
        &mut self,
        var: &mut VariableDeclaration,
    ) -> eyre::Result<(), VisitError> {
        if let Some(type_id) = &var.type_descriptions.type_identifier {
            if type_id == "t_uint256" {
                self.findings.push(Finding {
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
