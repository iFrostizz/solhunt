// Module that finds for external and dangerous calls

use core::{
    loader::{DynModule, Module},
    walker::{Finding, Severity},
};
use ethers_solc::artifacts::{
    ast::{ContractDefinitionPart, SourceUnitPart},
    Block, Contract, ContractKind, Expression, ExpressionStatement, FunctionCall,
    FunctionDefinition, Statement,
};

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
                            if let Some(body) = &func.body {
                                // dbg!(&body);

                                findings.append(&mut parse_body(body));
                            }
                        }
                    });
                }
            }

            findings
        }),
    )
}

fn parse_body(body: &Block) -> Vec<Finding> {
    let mut findings = Vec::new();

    body.statements
        .iter()
        .for_each(|stat| findings.append(&mut check_for_external_call(stat)));

    findings
}

fn check_for_external_call(stat: &Statement) -> Vec<Finding> {
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
    use crate::test::{compile_and_get_findings, lines_for_findings_with_code};

    #[test]
    fn can_find_call() {
        let findings = compile_and_get_findings(
            "Call.sol",
            "pragma solidity ^0.8.0;
contract Foo {
    address to;

    constructor(address _to) {
        to = _to;
    }

    function doTheThing() public {
        to.call{value: 1 ether}('');
    }
}",
        );

        assert_eq!(
            lines_for_findings_with_code(&findings, "calls", 0),
            vec![10]
        );
    }

    #[test]
    fn can_find_interface_call() {
        let findings = compile_and_get_findings(
            "CallInt.sol",
            "pragma solidity ^0.8.0;
interface Coll {
    function setStuff() external;
}
            
contract Foo {
    Coll to;

    constructor(Coll _to) {
        to = _to;
    }

    function doTheThing() public {
        to.setStuff();
    }
}",
        );

        assert_eq!(
            lines_for_findings_with_code(&findings, "calls", 0),
            vec![14]
        );
    }
}
