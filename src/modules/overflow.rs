// Check if overflow may occur in unchecked or < 16.8.0 versions of solc

// use crate::utils::int_as_bytes;
use crate::{
    loader::{DynModule, Module},
    walker::{version_from_string_literals, Finding, Severity},
};
use ethers_solc::artifacts::{
    ast::{
        AssignmentOperator::{AddAssign, MulAssign, SubAssign},
        ContractDefinitionPart, Expression, SourceUnitPart, Statement,
    },
    visitor::{VisitError, Visitor},
    Assignment, Block, FunctionDefinition, PragmaDirective, UncheckedBlock,
};
use semver::{Error, Version};

#[derive(Default)]
pub struct DetectionModule {
    findings: Vec<Finding>,
    version: Option<Version>,
}

impl Visitor<Vec<Finding>> for DetectionModule {
    fn visit_pragma_directive(
        &mut self,
        pragma_directive: &mut PragmaDirective,
    ) -> eyre::Result<(), VisitError> {
        let sem_ver = version_from_string_literals(pragma_directive.literals.clone());

        if sem_ver.minor < 8 {
            self.findings.push(Finding {
                    name: "overflow".to_string(),
                    description: "Looks like this contract is < 0.8.0, there is no built-in overflow check, be careful!".to_string(),
                    severity: Severity::Informal, // no real finding so it's informal for now
                    src: None, // SourceLocation::from_str("0:0:0").unwrap(),
                    code: 0,
                })
        } // else will need to check for "unchecked"

        self.version = Some(sem_ver);

        Ok(())
    }

    fn visit_assignment(&mut self, assignment: &mut Assignment) -> eyre::Result<(), VisitError> {
        // println!("{:#?}", assignment);
        // match assignment.operator {
        //     // TODO: if uses AddAssign and msg.value, it's probably fine, if > u64 (20 ETH doesn't hold in u64)
        //     AddAssign | MulAssign => self.findings.push(Finding {
        //         name: "overflow".to_string(),
        //         description: "Overflow may happen".to_string(),
        //         severity: Severity::Medium,
        //         src: Some(assignment.src.clone()),
        //         code: 1,
        //     }),
        //     SubAssign => self.findings.push(Finding {
        //         name: "overflow".to_string(),
        //         description: "Underflow may happen".to_string(),
        //         severity: Severity::Medium,
        //         src: Some(assignment.src.clone()),
        //         code: 2,
        //     }),
        //     _ => (),
        // }

        Ok(())
    }

    fn visit_unchecked_block(
        &mut self,
        unchecked_block: &mut UncheckedBlock,
    ) -> eyre::Result<(), VisitError> {
        // self.findings.push(Finding {
        //     name: "overflow".to_string(),
        //     description: "Unchecked block, so extra care here".to_string(),
        //     severity: Severity::Informal,
        //     src: Some(unchecked_block.src.clone()),
        //     code: 3,
        // });

        Ok(())
    }

    fn visit_function_definition(
        &mut self,
        function_definition: &mut FunctionDefinition,
    ) -> eyre::Result<(), VisitError> {
        if let Some(body) = &function_definition.body {
            if let Some(version) = self.version.clone() {
                if version.minor < 8 {
                    self.findings.append(&mut parse_body(body));
                } else {
                    // search for some unchecked
                    // dbg!(&func);
                    self.findings.append(&mut search_unchecked(body));
                }
            }
        }

        Ok(())
    }

    fn shared_data(&mut self) -> &Vec<Finding> {
        &self.findings
    }
}

// pub fn search_overflow(assignment: &mut Assignment) -> Vec<Finding> {
//     let mut findings = Vec::new();

//     match &assignment.operator {
//         // TODO: if uses AddAssign and msg.value, it's probably fine, if > u64 (20 ETH doesn't hold in u64)
//         AddAssign | MulAssign => findings.push(Finding {
//             name: "Overflow".to_string(),
//             description: "Overflow may happen".to_string(),
//             severity: Severity::Medium,
//             src: Some(assignment.src.clone()),
//             code: 1,
//         }),
//         SubAssign => findings.push(Finding {
//             name: "Underflow".to_string(),
//             description: "Underflow may happen".to_string(),
//             severity: Severity::Medium,
//             src: Some(assignment.src.clone()),
//             code: 2,
//         }),
//         _ => (),
//     }

//     findings
// }

pub fn get_module() -> DynModule {
    Module::new(
        "overflow",
        Box::new(|source, info| {
            // TODO: call a "setup" hook with info
            let mut findings: Vec<Finding> = Vec::new();

            if info.version.minor < 8 {
                findings.push(Finding {
                    name: "overflow".to_string(),
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
                    name: "overflow".to_string(),
                    description: "Overflow may happen".to_string(),
                    severity: Severity::Medium,
                    src: Some(ass.src.clone()),
                    code: 1,
                }),
                SubAssign => findings.push(Finding {
                    name: "overflow".to_string(),
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
                    name: "overflow".to_string(),
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
mod test {
    use crate::{
        solidity::ProjectFile,
        test::{
            compile_and_get_findings, has_with_code, has_with_module, lines_for_findings_with_code,
        },
    };

    #[test]
    fn can_find_overflow_old_ver() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("OldVerCheck"),
            String::from(
                "pragma solidity 0.7.0;
contract OldVerCheck {
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
            ),
        )]);

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
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("NoOverFlow"),
            String::from(
                "pragma solidity ^0.8.10;
contract NoOverFlow {
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
            ),
        )]);

        assert!(!has_with_module(&findings, "overflow"));
    }

    #[test]
    fn find_unchecked_overflow() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("Unchecked"),
            String::from(
                "pragma solidity ^0.8.10;
contract Unchecked {
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
            ),
        )]);

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
