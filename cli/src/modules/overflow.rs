// Check if overflow may occur in unchecked or < 0.8.0 versions of solc

// use crate::utils::int_as_bytes;
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
                        if let Some(body) = &func.body {
                            if info.version.minor < 8 {
                                findings.append(&mut parse_body(body));
                            } else {
                                // search for some unchecked
                                // dbg!(&func);
                                findings.append(&mut search_unchecked(body));
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

fn check_overflow_stat(stat: &Statement) -> Vec<Finding> {
    let mut findings = Vec::new();

    dbg!(&stat);

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

    findings
}

fn parse_body(body: &Block) -> Vec<Finding> {
    let mut findings = Vec::new();

    body.statements.iter().for_each(|stat| {
        findings.append(&mut check_overflow_stat(stat));
    });

    findings
}

/*fn search_over_in_unchecked(stat: &Statement) -> Vec<Finding> {
    let mut findings = Vec::new();

    stat.statements.iter().for_each()

    findings
}*/

fn search_unchecked(body: &Block) -> Vec<Finding> {
    let mut findings = Vec::new();

    body.statements.iter().for_each(|stat| {
        fn internal_search(stat: &Statement) -> Vec<Finding> {
            let mut i_findings = Vec::new();

            if let Statement::UncheckedBlock(block) = stat {
                i_findings.push(Finding {
                    name: "Unchecked".to_string(),
                    description: "Unchecked block, so extra care here".to_string(),
                    severity: Severity::Informal,
                    src: Some(block.src.clone()),
                    code: 3,
                });

                block.statements.iter().for_each(|s| {
                    i_findings.append(&mut check_overflow_stat(s));
                });
            }

            i_findings
        }

        findings.append(&mut internal_search(stat));
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
                    .filter(|char| char.is_ascii_digit() || char.to_string() == ".")
            })
            .collect::<String>()
            .as_str(),
    )
}

#[cfg(test)]
mod module_overflow_test {
    use crate::test::{
        compile_and_get_findings, has_with_code, has_with_module, lines_for_findings_with_code,
    };

    #[test]
    fn can_find_overflow_old_ver() {
        let findings = compile_and_get_findings(
            "OldVerCheck",
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
}",
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
            "NoOverFlow",
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

        assert!(!has_with_module(&findings, "overflow"));
    }

    #[test]
    fn find_unchecked_overflow() {
        let findings = compile_and_get_findings(
            "Unchecked",
            "pragma solidity ^0.8.10;
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
}",
        );

        assert!(!has_with_code(&findings, "overflow", 0));
        assert_eq!(
            lines_for_findings_with_code(&findings, "overflow", 3),
            vec![6, 12]
        ); // unchecked
        assert_eq!(
            lines_for_findings_with_code(&findings, "overflow", 1),
            vec![7]
        ); // +
        assert_eq!(
            lines_for_findings_with_code(&findings, "overflow", 2),
            vec![13]
        ); // -
    }
}
