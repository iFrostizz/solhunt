// https://github.com/code-423n4/2023-01-biconomy-findings/blob/main/data/Rolezn-G.md#gas3-setting-the-constructor-to-payable

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        (
            0,
            FindingKey {
                summary: "Setting the constructor to payable".to_string(),
                description: "Marking the constructor as payable removes any value check from the compiler and saves some gas on deployment.".to_string(),
                severity: Severity::Gas
            }
        ),
        (
            1,
            FindingKey {
                summary: "Use `Clones` to deploy a contract".to_string(),
                description: "The usage of the `new` keyword in solidity is for deploying smart contracts. But the constructor part is very expensive and can usually be replaced by a clone. Learn more from a very nice video from OpenZeppelin: https://www.youtube.com/watch?v=3Mw-pMmJ7TA".to_string(),
                severity: Severity::Gas
            }
        )
    ]),

    fn visit_function_definition(&mut self, function_definition: &mut FunctionDefinition) {
        if function_definition.name.is_empty() && function_definition.kind == Some(FunctionKind::Constructor) && function_definition.state_mutability != Some(StateMutability::Payable) {
            self.push_finding(0, Some(function_definition.src.clone()));
        }

        function_definition.visit(self)
    },

    fn visit_new_expression(&mut self, expr: &mut NewExpression) {
        self.push_finding(1, Some(expr.src.clone()));

        Ok(())
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

#[test]
fn new_deploy() {
    let findings = compile_contract_and_get_findings(String::from(
        "pragma solidity 0.8.0;

contract DeployMe {}

contract Deployer {
    function deployOne() public {
        address deployed = address(new DeployMe());
    }
}",
    ));

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "constructor", 1),
        vec![7]
    );
}
