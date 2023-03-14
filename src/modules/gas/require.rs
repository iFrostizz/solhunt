use crate::build_visitor;

build_visitor! {
    // https://github.com/code-423n4/2023-01-biconomy-findings/blob/main/data/chrisdior4-G.md#g-01-use-custom-errors-instead-of-revert-strings
    BTreeMap::from([
       (
            0,
            FindingKey {
                summary: String::from("Use custom errors instead of revert strings"),
                description: String::from("Solidity 0.8.4 added the custom errors functionality, which can be use instead of revert strings, resulting in big gas savings on errors. Replace all revert statements with custom error ones"),
                severity: Severity::Gas
            }
        ),
        (
            1,
            FindingKey {
                summary: "Duplicated require()/revert() Checks Should Be Refactored To A Modifier Or an internal function".to_string(),
                description: "Less code means a less costly deployment".to_string(),
                severity: Severity::Gas
            }
        ),
        (
            2,
            FindingKey {
                summary: "Use `require` instead of `assert`".to_string(),
                description: "assert wastes all the transaction gas. Use require instead".to_string(),
                severity: Severity::Gas
            }
        )
    ]),

    fn visit_source_unit(&mut self, source_unit: &mut SourceUnit) {
        source_unit.visit(self)?;

        self.revert_reasons.clone().values().for_each(|sources| {
            let s_len = sources.len();
            if s_len > 1 {
                let p_findings = sources.iter().map(|src| {
                   PushedFinding {
                       code: 1,
                       src: Some(src.clone())
                   }
                }).collect();

                self.push_findings(p_findings)
            }
        });

        self.revert_reasons.clear();

        Ok(())
    },

    fn visit_identifier(&mut self, identifier: &mut Identifier) {
        let id_name = &identifier.name;
        if id_name == "require" || id_name == "revert" {
            let arg_ty = &identifier.argument_types;

            if let Some(reason) = if identifier.name == "require" {
                let condition = &arg_ty[0];
                if condition == &(TypeDescriptions {
                    type_identifier: Some(String::from("t_bool")),
                    type_string: Some(String::from("bool"))
                }
                ) {
                    arg_ty.get(1)
                } else {
                    None
                }
            } else if identifier.name == "revert" {
arg_ty.get(0)
            } else {
                None
            } {
                    if let Some(id) = &reason.type_identifier {

                        self.revert_reasons.entry(id.to_string()).and_modify(|times| {
                            times.push(identifier.src.clone())
                       }).or_insert(vec![identifier.src.clone()]);

                        if id.starts_with("t_stringliteral_") {
                            self.push_finding(0, Some(identifier.src.clone()))
                        }
                    }
            }
        } else if id_name == "assert" {
            self.push_finding(2, Some(identifier.src.clone()))
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
        lines_for_findings_with_code_module(&findings, "require", 0),
        vec![5]
    );

    assert!(!has_with_code(&findings, "require", 1));
}

#[test]
fn require_twice() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Require"),
        String::from(
            r#"pragma solidity 0.8.0;

contract Require {
    function require1() public {
        require(true, "Repeated error");
    }

    function require2() public {
        require(false, "Repeated error");
    }
}"#,
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "require", 1),
        vec![5, 9]
    );
}

#[test]
fn revert_twice() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Revert"),
        String::from(
            r#"pragma solidity 0.8.0;

contract Revert {
    function revert1() public {
        revert("Repeated error");
    }

    function revert2() public {
        revert("Repeated error");
    }
}"#,
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "require", 1),
        vec![5, 9]
    );
}

#[test]
fn mixed_rev() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Mixed"),
        String::from(
            r#"pragma solidity 0.8.0;

contract Mixed {
    function revert1() public {
        revert("Repeated error");
    }

    function require1() public {
        require(false, "Repeated error");
    }
}"#,
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "require", 1),
        vec![5, 9]
    );
}

#[test]
fn not_repeated() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("NotRep"),
        String::from(
            r#"pragma solidity 0.8.0;

contract NotRep {
    function revert1() public {
        revert("Repeated error");
    }

    function require1() public {
        require(false, "Nope");
    }
}"#,
        ),
    )]);

    assert!(!has_with_code(&findings, "require", 1));
}

#[test]
fn uses_assert() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Assert"),
        String::from(
            r#"pragma solidity 0.8.0;

contract Assert {
    function asserting() public {
        assert(false);
    }
}"#,
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "require", 2),
        vec![5]
    );
}
