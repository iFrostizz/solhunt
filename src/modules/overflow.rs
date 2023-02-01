// Check if overflow may occur in unchecked or < 0.8.0 versions of solc

use crate::{build_visitor, walker::version_from_string_literals};
use ethers_solc::artifacts::ast::{
    AssignmentOperator::{AddAssign, MulAssign, SubAssign},
    Expression,
};

build_visitor!(
    BTreeMap::from([
        (
            0,
            FindingKey {
                description: "Looks like this contract is < 0.8.0".to_string(),
                severity: Severity::Informal
            }
        ),
        (
            1,
            FindingKey {
                description: "Overflow may happen".to_string(),
                severity: Severity::Medium
            }
        ),
        (
            2,
            FindingKey {
                description: "Underflow may happen".to_string(),
                severity: Severity::Medium
            }
        ),
        (
            3,
            FindingKey {
                description: "Unchecked block".to_string(),
                severity: Severity::Informal
            }
        )
    ]),
    fn visit_pragma_directive(&mut self, pragma_directive: &mut PragmaDirective) {
        let sem_ver = version_from_string_literals(pragma_directive.literals.clone());

        if sem_ver.minor < 8 {
            self.push_finding(None, 0);
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
        self.push_finding(Some(unchecked_block.src.clone()), 3);
        unchecked_block.visit(self)?;
        self.inside.unchecked = false;

        Ok(())
    },
    fn visit_expression_statement(&mut self, expression_statement: &mut ExpressionStatement) {
        if self.inside.unchecked || matches!(&self.version, Some(version) if version.minor < 8) {
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

                match &ass.operator {
                    // TODO: if uses AddAssign and msg.value, it's probably fine, if > u64 (20 ETH doesn't hold in u64)
                    AddAssign | MulAssign => self.push_finding(Some(ass.src.clone()), 1),
                    SubAssign => self.push_finding(Some(ass.src.clone()), 2),
                    _ => (),
                }
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
