// https://github.com/code-423n4/2023-01-biconomy-findings/blob/main/data/Rolezn-G.md#gas3-setting-the-constructor-to-payable

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        (
            0,
            FindingKey {
                summary: String::from("Setting the constructor to payable"),
                description: String::from("Marking the constructor as payable removes any value check from the compiler and saves some gas on deployment."),
                severity: Severity::Gas
            }
        )
    ]),

    fn visit_function_definition(&mut self, function_definition: &mut FunctionDefinition) {
        if function_definition.name.is_empty() && function_definition.kind == Some(FunctionKind::Constructor) && function_definition.state_mutability != Some(StateMutability::Payable) {
            self.push_finding(0, Some(function_definition.src.clone()));
        }

        function_definition.visit(self)
    }
}

#[test]
fn payable_constructor() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("NoPayableConstructor"),
        String::from(
            r#"pragma solidity 0.8.0;

contract NoPayableConstructor {
    constructor() {
        revert("I'm not payable at all ahah!");
    }
}"#,
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "constructor", 0),
        vec![4]
    );

    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("PayableConstructor"),
        String::from(
            r#"pragma solidity 0.8.0;

contract PayableConstructor {
    constructor() payable {
        revert("I'm payable!");
    }
}"#,
        ),
    )]);

    assert!(!has_with_code(&findings, "constructor", 0),);
}
