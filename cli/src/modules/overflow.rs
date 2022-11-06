// Check if overflow may occur in unchecked or < 0.8.0 versions of solc

use crate::utils::int_as_bytes;
use core::{
    loader::{DynModule, Module},
    walker::{Finding, Severity},
};
use ethers_solc::artifacts::{
    ast::{
        AssignmentOperator::{AddAssign, MulAssign, SubAssign},
        ContractDefinitionPart, Expression, SourceUnitPart, Statement,
    },
    Block,
};
use semver::{Error, Version};

pub fn get_module() -> DynModule {
    Module::new(
        "overflow",
        Box::new(|source, info| {
            // TODO: call a "setup" hook with info
            let mut findings: Vec<Finding> = Vec::new();

            if info.version.minor < 8 {
                findings.push(Finding {
                    name: "No built-il overflow check".to_string(),
                    description: "Looks like this contract is < 0.8.0, there is no built-in overflow check, be careful!".to_string(),
                    severity: Severity::Informal, // no real finding so it's informal for now
                    src: None, // SourceLocation::from_str("0:0:0").unwrap(),
                    code: 0,
                })
            } else {
                // Less likely but will need to check for "unchecked"
            }

            if let SourceUnitPart::ContractDefinition(def) = source {
                def.nodes.iter().for_each(|node| {
                    if let ContractDefinitionPart::FunctionDefinition(func) = node {
                        if info.version.minor < 8 {
                            if let Some(body) = &func.body {
                                findings.append(&mut parse_body(body));
                            }
                        }
                    } /*else {
                          unimplemented!("Overflow module: ContractDefinitionPart TBD");
                      }*/
                });
            }

            findings
        }),
    )
}

fn parse_body(body: &Block) -> std::vec::Vec<core::walker::Finding> {
    let mut findings = Vec::new();

    body.statements.iter().for_each(|stat| {
        if let Statement::ExpressionStatement(expr) = stat {
            if let Expression::Assignment(ass) = &expr.expression {
                // huh

                /*let lhs = &ass.lhs;
                let rhs = &ass.rhs;

                if let Expression::IndexAccess(idx) = lhs {
                    if let Some(typ) = &idx.type_descriptions.type_string {
                        if let Some(bytes) = int_as_bytes(typ) {
                            if bytes <= 64 {}
                        }
                    }
                }

                if let Expression::IndexAccess(idx) = rhs {
                    if let Some(typ) = &idx.type_descriptions.type_string {
                        if let Some(bytes) = int_as_bytes(typ) {
                            if bytes <= 64 {}
                        }
                    }
                }*/

                match &ass.operator {
                    // TODO: if uses AddAssign and msg.value, it's probably fine, if > u64 (20 ETH doesn't hold in u64)
                    AddAssign | MulAssign => findings.push(Finding {
                        name: "Overflow".to_string(),
                        description: "Overflow may happen".to_string(),
                        severity: Severity::Medium,
                        src: Some(ass.src.clone()),
                        code: 1,
                    }),
                    SubAssign => findings.push(Finding {
                        name: "Underflow".to_string(),
                        description: "Underflow may happen".to_string(),
                        severity: Severity::Medium,
                        src: Some(ass.src.clone()),
                        code: 2,
                    }),
                    _ => (),
                }
            } else {
                // unimplemented!("Overflow module: Expression TBD");
            }
        } else {
            // unimplemented!("Overflow module: Statement TBD");
        }
    });

    findings
}

#[allow(unused)]
fn parse_literals(literals: Vec<String>) -> Result<Version, Error> {
    Version::parse(
        literals
            .iter()
            .flat_map(|literal| {
                literal
                    .chars()
                    .filter(|char| char.is_digit(10) || char.to_string() == ".")
            })
            .collect::<String>()
            .as_str(),
    )
}

#[cfg(test)]
mod module_overflow_test {
    use crate::test::{
        compile_and_get_findings, finding_num, has_with_code, has_with_code_at_line,
        lines_for_findings_with_code,
    };

    #[test]
    fn can_find_overflow_old_ver() {
        let findings = compile_and_get_findings(
            "OldVerCheck.sol",
            "pragma solidity 0.7.0;
contract Foo {
    mapping(address => uint256) bal;

    function deposit() external payable {
        bal[msg.sender] += msg.value;
    }

    function withdraw(uint256 amount) external {
        bal[msg.sender] -= amount;
        payable(msg.sender).transfer(amount);
    }

    fallback() external payable {}
} ",
        );

        assert!(has_with_code(&findings, "overflow", 0)); // ver
        assert_eq!(
            lines_for_findings_with_code(&findings, "overflow", 1),
            vec![6]
        ); // +
        assert_eq!(
            lines_for_findings_with_code(&findings, "overflow", 2),
            vec![10]
        ); // -
    }

    #[test]
    fn dont_find_overflow() {
        let findings = compile_and_get_findings(
            "NoOverFlow.sol",
            "pragma solidity ^0.8.10;
        contract Foo {
        mapping(address => uint256) bal;
            
            function deposit() external payable {
            bal[msg.sender] += msg.value;
            }
            
            function withdraw(uint256 amount) external {
            bal[msg.sender] -= amount;
            payable(msg.sender).transfer(amount);
            }
            
            fallback() external payable {}
        }",
        );

        assert_eq!(finding_num(&findings, "overflow"), 0);
    }

    #[test]
    fn find_unchecked_overflow() {
        let findings = compile_and_get_findings(
            "Unchecked.sol",
            "
            pragma solidity ^0.8.10;
            contract Foo {
            mapping(address => uint256) bal;
            
            function deposit() external payable {
                unchecked {
                    bal[msg.sender] += msg.value;
                }
            }
            
            function withdraw(uint256 amount) external {
                unchecked {
                    bal[msg.sender] -= amount;
                }
                payable(msg.sender).transfer(amount);
            }
            
            fallback() external payable {}
        }
        ",
        );

        assert!(!has_with_code(&findings, "overflow", 0));
        assert!(has_with_code(&findings, "overflow", 1)); // +
        assert!(has_with_code(&findings, "overflow", 2)); // -
    }
}
