// Module that finds for external and dangerous calls

use crate::walker::{Finding, Severity};
use ethers_solc::artifacts::{
    visitor::{VisitError, Visitable, Visitor},
    yul::YulFunctionCall,
    InlineAssembly,
};

#[derive(Default)]
pub struct DetectionModule {
    findings: Vec<Finding>,
}

impl Visitor<Vec<Finding>> for DetectionModule {
    fn visit_inline_assembly(
        &mut self,
        inline_assembly: &mut InlineAssembly,
    ) -> eyre::Result<(), VisitError> {
        self.findings.push(Finding {
            name: "assembly".to_string(),
            description: "usage of inline assembly, take extra care here".to_string(),
            severity: Severity::Informal,
            src: Some(inline_assembly.src.clone()),
            code: 0,
        });

        // don't disrupt current ast traversal
        inline_assembly.visit(self)
    }

    fn visit_yul_function_call(
        &mut self,
        function_call: &mut YulFunctionCall,
    ) -> eyre::Result<(), VisitError> {
        let func_name = &function_call.function_name;

        if func_name.name == "extcodesize" {
            self.findings.push(Finding {
                name: "assembly".to_string(),
                description: "using extcodesize. Can be an issue if determining if EOA."
                    .to_string(),
                severity: Severity::Medium,
                src: Some(func_name.src.clone()),
                code: 1,
            });
        }

        function_call.visit(self)
    }

    fn shared_data(&mut self) -> &Vec<Finding> {
        &self.findings
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        solidity::ProjectFile,
        test::{compile_and_get_findings, has_with_code, lines_for_findings_with_code},
    };

    #[test]
    fn with_extcodesize() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("ExtCodeSize"),
            String::from(
                "pragma solidity ^0.8.0;

contract ExtCodeSize {
    function make(address to) public {
        uint256 size;
            
        assembly {
            size := extcodesize(to)
        }
    }
}",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 0), // usage of assembly
            vec![7]
        );

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 1), // extcodesize
            vec![8]
        );
    }

    #[test]
    fn without_extcodesize() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("WithoutExtCodeSize"),
            String::from(
                "pragma solidity ^0.8.0;

contract WithoutExtCodeSize {
    function make(address to) public {
        uint256 bal;
            
        assembly {
            bal := balance(to)
        }
    }
}",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 0), // usage of assembly
            vec![7]
        );

        assert!(!has_with_code(&findings, "assembly", 1)); // extcodesize);
    }

    #[test]
    fn nested_extcodesize() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("NestedExtCodeSize"),
            String::from(
                "pragma solidity ^0.8.0;

contract NestedExtCodeSize {
    function make(address to) public {
        uint256 size;
            
        assembly {
            for { let i:= 0 } lt(i, 10) { i := add(i, 1) } {
                size := extcodesize(to)
            }
        }
    }
}",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 0), // usage of assembly
            vec![7]
        );

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 1), // extcodesize
            vec![9]
        );
    }
}
