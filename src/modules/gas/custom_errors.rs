use crate::build_visitor;

build_visitor! {
    // https://github.com/code-423n4/2023-01-biconomy-findings/blob/main/data/chrisdior4-G.md#g-01-use-custom-errors-instead-of-revert-strings
    BTreeMap::from([(
            0,
            FindingKey {
                summary: String::from("Use custom errors instead of revert strings"),
                description: String::from("Solidity 0.8.4 added the custom errors functionality, which can be use instead of revert strings, resulting in big gas savings on errors. Replace all revert statements with custom error ones"),
                severity: Severity::Gas
            }
    )]),

    fn visit_identifier(&mut self, identifier: &mut Identifier) {
        if identifier.name == "require" {
            let arg_ty = &identifier.argument_types;

            let condition = &arg_ty[0];
            if condition == &(TypeDescriptions {
                type_identifier: Some(String::from("t_bool")),
                type_string: Some(String::from("bool"))
            }) {
                // that's definitely a "require" statement
                if let Some(reason) = arg_ty.get(1) {
                    if let Some(id) = &reason.type_identifier {
                        if id.starts_with("t_stringliteral_") {
                            self.push_finding(0, Some(identifier.src.clone()))
                        }
                    }
                }
            }
        }

        identifier.visit(self)
    }
}

#[test]
fn custom_error_string() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("CustomError"),
        String::from(
            r#"pragma solidity 0.8.0;

contract CustomError {
    function reverts() public {
        require(1 == 0, "This is some costly revert reason string");
    }
}"#,
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code(&findings, "custom_errors", 0),
        vec![5]
    );
}