use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
       (
           0,
           FindingKey {
                        summary: "Avoid contract existence checks by using low level calls".to_string(),
                        description: "Prior to 0.8.10 the compiler inserted extra code, including EXTCODESIZE (100 gas), to check for contract existence for external function calls. In more recent solidity versions, the compiler will not insert these checks if the external call has a return value. Similar behavior can be achieved in earlier versions by using low-level calls, since low level calls never check for contract existence".to_string(),
                        severity: Severity::Gas
           }
       ),
       // (
       //     1,
       //     FindingKey {
       //         description: "external call with arbitrary address".to_string(),
       //         summary: "external call with arbitrary address".to_string(),
       //         severity: Severity::Medium
       //     }
       // ),
       (
           2,
           FindingKey {
               description: "delegatecall in a loop".to_string(),
               summary: "delegatecall in a loop".to_string(),
               severity: Severity::High
           }
       )
    ]),

    fn visit_function_definition(
        &mut self,
        function_definition: &mut FunctionDefinition
    ) {
        let data = parse_params(&function_definition.parameters);

        if let Some(body) = &function_definition.body {
            self.push_findings(parse_body(body, &data));
        }

        function_definition.visit(self)
    },

    fn visit_member_access(&mut self, member_access: &mut MemberAccess) {
        if (self.inside.for_loop || self.inside.while_loop) && member_access.member_name == "delegatecall" {
            self.push_finding(2, Some(member_access.src.clone()));
        }

        member_access.visit(self)
    },

    fn visit_for_statement(&mut self, for_statement: &mut ForStatement) {
        self.inside.for_loop = true;
        for_statement.visit(self)?;
        self.inside.for_loop = false;
        Ok(())
    },

    fn visit_while_statement(&mut self, while_statement: &mut WhileStatement) {
        self.inside.while_loop = true;
        while_statement.visit(self)?;
        self.inside.while_loop = false;
        Ok(())
    }
}

fn parse_params(params: &ParameterList) -> HashMap<String, String> {
    let mut data: HashMap<String, String> = HashMap::new();

    for param in params.parameters.clone().into_iter() {
        if let Some(type_name) = param.type_name {
            match type_name {
                TypeName::ElementaryTypeName(type_name) => {
                    if type_name.name == "address" {
                        data.insert(param.name, "address".to_string());
                    }
                }
                TypeName::ArrayTypeName(_type_name) => {
                    // todo!()
                }
                _ => {
                    // todo!()
                }
            }
        }
    }

    data
}

fn parse_body(body: &Block, data: &HashMap<String, String>) -> Vec<PushedFinding> {
    let mut findings = Vec::new();

    body.statements
        .iter()
        .for_each(|stat| findings.append(&mut check_for_external_call(stat, data)));

    findings
}

fn check_for_external_call(stat: &Statement, data: &HashMap<String, String>) -> Vec<PushedFinding> {
    #[allow(unused_mut)]
    let mut findings = Vec::new();

    if let Statement::ExpressionStatement(expr) = stat {
        if let Expression::FunctionCall(call) = &expr.expression {
            // dbg!(&call);
            // findings.push(PushedFinding {
            //     src: Some(call.src.clone()),
            //     code: 0,
            // });

            let func_expr = &call.expression;
            if let Expression::MemberAccess(mem) = func_expr {
                if let Expression::Identifier(identifier) = &mem.expression {
                    if let Some(arb_type) = data.get(&identifier.name) {
                        if arb_type == "address" {
                            // findings.push(PushedFinding {
                            //     src: Some(call.src.clone()),
                            //     code: 1,
                            // });
                        }
                    }
                }
            }

            /*if let Expression::FunctionCallOptions(opt) = &call.expression {
                if let Expression::MemberAccess(acc) = &opt.expression {
                    let type_desc = &acc.type_descriptions;
                    if let Some(id) = &type_desc.type_identifier {
                        if id.starts_with("t_function_barecall") {
                            findings.push(Finding {
                                name: "calls".to_owned(),
                                description: "external call detected".to_owned(),
                                severity: Severity::Informal,
                                src: Some(call.src.clone()),
                                code: 0,
                            });
                        }
                    }
                }
            }*/
        }
    }

    findings
}

// #[test]
// fn can_find_arbitrary_call() {
//     let findings = compile_and_get_findings(vec![ProjectFile::Contract(
//         String::from("Arbitrary"),
//         String::from(
//             "pragma solidity ^0.8.0;

// contract Arbitrary {

// function doTheThing(address to) public {
//     to.call('');
// }
// }",
//         ),
//     )]);

//     assert_eq!(lines_for_findings_with_code(&findings, "calls", 1), vec![6]);
// }

// https://github.com/Picodes/4naly3er/blob/main/src/issues/H/delegateCallInLoop.ts
// TODO: add payable function condition ? Security concecrn here is the msg.value
#[test]
fn delegatecall_in_for_loop() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("DelegateCallLoop"),
        String::from(
            r#"pragma solidity ^0.8.0;

contract DelegateCallForLoop {
    function causeTrouble(address to) public {
        for (uint256 i; i < 10; i++) {
            to.delegatecall("");
        }
    }
}"#,
        ),
    )]);

    assert_eq!(lines_for_findings_with_code(&findings, "calls", 2), vec![6]);
}

#[test]
fn delegatecall_in_while_loop() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("DelegateCallLoop"),
        String::from(
            r#"pragma solidity ^0.8.0;

contract DelegateCallWhileLoop {
    function causeTrouble(address to) public {
        uint256 i = 0;
        while (i < 10) {
            to.delegatecall("");
        }
    }
}"#,
        ),
    )]);

    assert_eq!(lines_for_findings_with_code(&findings, "calls", 2), vec![7]);
}

#[test]
fn use_low_level_interface() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("ContractExistence"),
        String::from(
            "pragma solidity ^0.8.0;
interface Coll {
    function setStuff() external;
}

contract CallInt {
    Coll to;

    constructor(Coll _to) {
        to = _to;
    }

    function doTheThing() public {
        to.setStuff();
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code(&findings, "calls", 0),
        vec![14]
    );
}

#[test]
fn use_low_level_contract() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("CallContract"),
        String::from(
            "pragma solidity ^0.8.0;
contract Coll {
    uint256 val;

    function setStuff(uint256 _val) external {
                val = _val;
    }
}

contract CallContract {
    Coll to;

    constructor(Coll _to) {
        to = _to;
    }

    function doTheThing() public {
        to.setStuff(10);
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code(&findings, "calls", 0),
        vec![18]
    );
}
