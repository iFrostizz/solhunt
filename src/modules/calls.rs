// Module that finds for external and dangerous calls

use crate::{
    loader::{DynModule, Module},
    walker::{Finding, Severity},
};
use ethers_solc::artifacts::{
    ast::{ContractDefinitionPart, SourceUnitPart},
    Block, ContractKind, Expression, ParameterList, Statement, TypeName,
};
use std::collections::HashMap;

pub fn get_module() -> DynModule {
    Module::new(
        "calls",
        Box::new(move |source, _info| {
            let mut findings: Vec<Finding> = Vec::new();

            if let SourceUnitPart::ContractDefinition(def) = source {
                // println!("{:#?}", def);

                if def.kind == ContractKind::Contract {
                    // dbg!(&source);

                    def.nodes.iter().for_each(|node| {
                        if let ContractDefinitionPart::FunctionDefinition(func) = node {
                            // parse_func(func);
                            // dbg!(&func);

                            let data = parse_params(&func.parameters);
                            // dbg!(&data);

                            if let Some(body) = &func.body {
                                findings.append(&mut parse_body(body, &data));
                            }
                        }
                    });
                }
            }

            findings
        }),
    )
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
                    println!("todo");
                }
                _ => println!("todo"),
            }
        }
    }

    data
}

fn parse_body(body: &Block, data: &HashMap<String, String>) -> Vec<Finding> {
    let mut findings = Vec::new();

    body.statements
        .iter()
        .for_each(|stat| findings.append(&mut check_for_external_call(stat, data)));

    findings
}

fn check_for_external_call(stat: &Statement, data: &HashMap<String, String>) -> Vec<Finding> {
    let mut findings = Vec::new();

    if let Statement::ExpressionStatement(expr) = stat {
        if let Expression::FunctionCall(call) = &expr.expression {
            // dbg!(&call);
            findings.push(Finding {
                name: "calls".to_owned(),
                description: "external call detected".to_owned(),
                severity: Severity::Informal,
                src: Some(call.src.clone()),
                code: 0,
            });

            let func_expr = &call.expression;
            if let Expression::MemberAccess(mem) = func_expr {
                if let Expression::Identifier(identifier) = &mem.expression {
                    if let Some(arb_type) = data.get(&identifier.name) {
                        if arb_type == "address" {
                            findings.push(Finding {
                                name: "calls".to_owned(),
                                description: "external call with arbitrary address".to_owned(),
                                severity: Severity::Medium,
                                src: Some(call.src.clone()),
                                code: 1,
                            });
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

#[cfg(test)]
mod tests {
    use crate::{
        solidity::ProjectFile,
        test::{compile_and_get_findings, lines_for_findings_with_code},
    };

    #[test]
    fn can_find_call() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("Call"),
            String::from(
                "pragma solidity ^0.8.0;
contract Call {
    address to;

    constructor(address _to) {
        to = _to;
    }

    function doTheThing() public {
        to.call{value: 1 ether}('');
    }
}",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "calls", 0),
            vec![10]
        );
    }

    #[test]
    fn can_find_interface_call() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("CallInt"),
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
    fn can_find_contract_call() {
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
            vec![19]
        );
    }

    #[test]
    fn can_find_arbitrary_call() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("Arbitrary"),
            String::from(
                "pragma solidity ^0.8.0;
            
contract Arbitrary {

    function doTheThing(address to) public {
        to.call('');
    }
}",
            ),
        )]);

        assert_eq!(lines_for_findings_with_code(&findings, "calls", 1), vec![6]);
    }
}
