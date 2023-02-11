// Module that finds for external and dangerous calls

use crate::build_visitor;
use ethers_solc::artifacts::{yul::YulFunctionCall, InlineAssembly};

build_visitor!(
    BTreeMap::from([
        // (
        //     0,
        //     FindingKey {
        //         description: "usage of inline assembly, take extra care here".to_string(),
        //         summary: "usage of inline assembly".to_string(),
        //         severity: Severity::Informal
        //     }
        // ),
        (
            1,
            FindingKey {
                description: "using extcodesize. Can be an issue if determining if EOA."
                    .to_string(),
                summary: "extcodesize for EOA test".to_string(),
                severity: Severity::Medium
            }
        )
    ]),
    fn visit_inline_assembly(&mut self, inline_assembly: &mut InlineAssembly) {
        // self.push_finding(0, Some(inline_assembly.src.clone()));

        // don't disrupt current ast traversal
        inline_assembly.visit(self)
    },
    fn visit_yul_function_call(&mut self, function_call: &mut YulFunctionCall) {
        let func_name = &function_call.function_name;

        if func_name.name == "extcodesize" {
            self.push_finding(1, Some(function_call.src.clone()))
        }

        function_call.visit(self)
    }
);

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
