// Module that finds for external and dangerous calls

use crate::build_visitor;

build_visitor!(
    BTreeMap::from([(
        0,
        FindingKey {
            description: "using extcodesize. Can be an issue if determining if EOA.".to_string(),
            summary: "extcodesize for EOA test".to_string(),
            severity: Severity::Medium
        }
    )]),
    fn visit_yul_identifier(&mut self, yul_identifier: &mut YulIdentifier) {
        if yul_identifier.name == "extcodesize" {
            self.push_finding(0, Some(yul_identifier.src.clone()))
        }

        yul_identifier.visit(self)
    }
);

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
        lines_for_findings_with_code_module(&findings, "assembly", 0), // extcodesize
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

    assert!(!has_with_code(&findings, "assembly", 0)); // extcodesize);
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
        lines_for_findings_with_code_module(&findings, "assembly", 0), // extcodesize
        vec![9]
    );
}
