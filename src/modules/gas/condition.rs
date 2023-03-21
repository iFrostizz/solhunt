use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        (
            0,
            FindingKey {
                summary: "use ternary operators rather than `if/else`".to_string(),
                description: "`if/else` gas overhead is higher than a ternary operator".to_string(),
                severity: Severity::Gas }
        )
    ]),
    fn visit_if_statement(&mut self, ifs: &mut IfStatement) {
        if ifs.false_body.is_some() {
            self.push_finding(0, Some(ifs.src.clone()));
        }

        Ok(())
    }
}

// https://github.com/code-423n4/2022-10-traderjoe-findings/issues/250
#[test]
fn if_else() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("IfElse"),
        String::from(
            "pragma solidity ^0.8.0;

contract IfElse {
    function flip() public returns(bool) {
        if (true) {
            return false;
        } else {
            return true;
        }
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "condition", 0),
        vec![5]
    );
}

#[test]
fn ternary() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Ternary"),
        String::from(
            "pragma solidity ^0.8.0;

contract Ternary {
    function flip() public returns(bool) {
        return true ? false : true;
    }
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "condition", 0),);
}

#[test]
fn if_only() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("IfOnly"),
        String::from(
            "pragma solidity ^0.8.0;

contract IfOnly {
    function flip() public returns(bool) {
        if (true) {
            return false;
        }    }
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "condition", 0));
}
