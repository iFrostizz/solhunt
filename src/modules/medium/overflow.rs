// Check if overflow may occur in unchecked or < 0.8.0 versions of solc

use crate::{build_visitor, walker::smallest_version_from_literals};

build_visitor!(
    BTreeMap::from([
        (
            0,
            FindingKey {
                description: "Looks like this contract is < 0.8.0".to_string(),
                summary: "No built-in overflow checker".to_string(),
                severity: Severity::Informal
            }
        ),
        (
            1,
            FindingKey {
                description: "Overflow may happen".to_string(),
                summary: "Overflow".to_string(),
                severity: Severity::Medium
            }
        ),
        (
            2,
            FindingKey {
                description: "Underflow may happen".to_string(),
                summary: "Underflow".to_string(),
                severity: Severity::Medium
            }
        ),
        (
            3,
            FindingKey {
                description: "Unchecked block".to_string(),
                summary: "Unchecked".to_string(),
                severity: Severity::Informal
            }
        )
    ]),
    fn visit_pragma_directive(&mut self, pragma_directive: &mut PragmaDirective) {
        let sem_ver = smallest_version_from_literals(pragma_directive.literals.clone()).unwrap();

        if sem_ver.minor < 8 {
            // self.push_finding(0, Some(pragma_directive.src.clone()));
        } // else will need to check for "unchecked"

        self.version = Some(sem_ver);

        pragma_directive.visit(self)
    },
    fn visit_assignment(&mut self, assignment: &mut Assignment) {
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

        assignment.visit(self)
    },
    fn visit_unchecked_block(&mut self, unchecked_block: &mut UncheckedBlock) {
        self.inside.unchecked = true;
        // self.push_finding(3, Some(unchecked_block.src.clone()));
        unchecked_block.visit(self)?;
        self.inside.unchecked = false;

        Ok(())
    },
    fn visit_expression_statement(&mut self, expression_statement: &mut ExpressionStatement) {
        if self.inside.unchecked || matches!(&self.version, Some(version) if version.minor < 8) {
            #[allow(unused)]
            if let Expression::Assignment(ass) = &expression_statement.expression {
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

                // match &ass.operator {
                //     // TODO: if uses AddAssign and msg.value, it's probably fine, if > u64 (20 ETH doesn't hold in u64)
                //     AddAssign | MulAssign => self.push_finding(1, Some(ass.src.clone())),
                //     SubAssign => self.push_finding(2, Some(ass.src.clone())),
                //     _ => (),
                // }
            } else {
                // unimplemented!("Overflow module: Expression TBD");
            }
        }
        Ok(())
    },
    fn visit_function_definition(&mut self, function_definition: &mut FunctionDefinition) {
        self.inside.function = true;
        function_definition.visit(self)?;
        self.inside.function = false;

        Ok(())
    }
);

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

    // TODO: check if any version under 0.8.0 can be selected
    assert_eq!(
        lines_for_findings_with_code_module(&findings, "overflow", 0),
        vec![1]
    ); // ver
    assert_eq!(
        lines_for_findings_with_code_module(&findings, "overflow", 1),
        vec![6]
    ); // +
    assert_eq!(
        lines_for_findings_with_code_module(&findings, "overflow", 2),
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
        lines_for_findings_with_code_module(&findings, "overflow", 3),
        vec![6, 12]
    ); // unchecked
    assert_eq!(
        lines_for_findings_with_code_module(&findings, "overflow", 1),
        vec![7]
    ); // +
    assert_eq!(
        lines_for_findings_with_code_module(&findings, "overflow", 2),
        vec![13]
    ); // -
}
