// https://github.com/code-423n4/2022-12-backed-findings/blob/main/data/IllIllI-G.md#g02--internal-functions-only-called-once-can-be-inlined-to-save-gas
use crate::build_visitor;

// unstable module
build_visitor! {
    BTreeMap::from([
        (
            0,
            FindingKey {
                summary: "`internal` functions only called once can be inlined to save gas".to_string(),
                description: "Not inlining costs 20 to 40 gas because of two extra JUMP instructions and additional stack operations needed for function calls.".to_string(),
                severity: Severity::Gas
            }
        )
    ]),
    fn visit_source_unit(&mut self, source_unit: &mut SourceUnit) {
        // pre-hook
        source_unit.visit(self)?;
        // post-hook

        let mut function_calls: HashMap<String, Vec<FunctionCall>> = HashMap::new();

        self.function_calls.iter().for_each(|fc| {
            // let fc_name = match &fc.expression {
            //     Expression::Identifier(e) => {
            //         e.name.clone()
            //     }
            //     _ =>
            //         String::from("")
            // };

            match &fc.expression {
                Expression::Identifier(e) => {
                    function_calls.entry(e.name.clone()).and_modify(|c| c.push(fc.clone())).or_insert(vec![fc.clone()]);
                }
                Expression::Assignment(_e) => (),
                _ => ()
            };

            // function_calls.entry(fc_name).and_modify(|c| c.push(fc.clone())).or_insert(vec![fc.clone()]);
        });

        self.function_definitions
            .clone()
            .into_iter()
            .filter(|fd|
                 fd.visibility == Visibility::Internal
                 || fd.visibility == Visibility::Private)
            .for_each(|fd| {

            let calls = match function_calls.get(&fd.name) {
                Some(c) => c.clone(),
                None => Vec::new()
            };

            // don't count if called 0 or anything strictly more than 1
            if calls.len() == 1 {
                self.push_finding(0, Some(calls[0].src.clone()));
            }
        });

        // clear for the next contract (won't work with overidden contracts)
        self.function_calls.clear();
        self.function_definitions.clear();

        Ok(())
    },
    fn visit_function_definition(&mut self, function_definition: &mut FunctionDefinition) {
        self.function_definitions.push(function_definition.clone());

        function_definition.visit(self)?;

        Ok(())
    },
    fn visit_function_call(&mut self, function_call: &mut FunctionCall) {
        if function_call.kind == FunctionCallKind::FunctionCall {
            self.function_calls.push(function_call.clone());
        }

        function_call.visit(self)
    }
}

#[test]
fn only_called_once() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("OnlyOnce"),
        String::from(
            "pragma solidity ^0.8.0;

contract OnlyOnce {
    function _once() internal {
        //
    }

    function make() public {
        _once();
    }
}
",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "tree", 0),
        vec![9]
    );
}

#[test]
fn called_twice() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("OnlyOnce"),
        String::from(
            "pragma solidity ^0.8.0;

contract OnlyOnce {
    function _once() internal {
        //
    }

    function make() public {
        _once();
    }

    function act() public {
        _once();
    }
}
",
        ),
    )]);

    assert!(!has_with_code(&findings, "tree", 0));
}

// https://github.com/code-423n4/2023-01-timeswap-findings/blob/main/data/0xSmartContract-G.md#g-02-remove-checkdoesnotexist-function

#[test]
fn called_in_comment() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("InComment"),
        String::from(
            "pragma solidity ^0.8.0;

contract InComment {
    function _once() internal {
        //
    }

    function make() public {
        _once();
    }

    /** @notice this is a very cool function
      * @dev in reality, you can just call _once
      * @dev to rug everybody ...
      */
    function act() public {
        // send it
    }
}
",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "tree", 0),
        vec![9]
    );
}

#[test]
fn overriden() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Overriden"),
        String::from(
            "pragma solidity ^0.8.0;

contract Kid {
    function _once() internal {
        //
    }
}

contract Overriden is Kid {
    function make() public {
        _once();
    }

    function act() public {
        // send it
    }
}
",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "tree", 0),
        vec![11]
    );
}
