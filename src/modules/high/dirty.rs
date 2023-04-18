// finds potential cases of dirty bytes

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        (
            0,
            FindingKey {
                summary: "Bug when copying dirty bytes arrays to storage".to_string(),
                description: "On July 1, 2021, a bug in the Solidity code generator was found by differential fuzzing. The bug causes the legacy code generation pipeline to generate code that may write dirty values to storage when copying bytes arrays from calldata or memory. Read more at: https://blog.soliditylang.org/2022/06/15/dirty-bytes-array-to-storage-bug/".to_string(),
                severity: Severity::High
            }
        )
    ]),

    fn visit_function_call(&mut self, fc: &mut FunctionCall) {
        if fc.type_descriptions.type_string == Some("bytes1".to_string()) {
            let expr = &fc.expression;

            if let Expression::MemberAccess(ma) = expr {
                if ma.member_name == "push" {
                    self.push_finding(0, Some(fc.src.clone()));
                }
            }
        }

        Ok(())
    }
}

#[test]
fn rareskills() {
    let findings = compile_contract_and_get_findings(String::from(
        "pragma solidity 0.7.0;

contract Dirty {
    mapping(address => string) public usernameOf;

    function setUsername(uint256 obfuscationDegree) public payable {
        for (uint256 i; i < obfuscationDegree; ++i) {
            bytes(usernameOf[msg.sender]).push();
        }
    }
}",
    ));

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "dirty", 0),
        vec![8]
    );
}
