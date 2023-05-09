use crate::build_visitor;

// use ethers_solc::Artifact

build_visitor! {
    BTreeMap::from([
        (
            0,
            FindingKey {
                summary: "use left shift".to_string(),
                description: "use left shift rather than mul".to_string(),
                severity: Severity::Gas
            }
        )
    ]),

    fn visit_binary_operation(&mut self, bo: &mut BinaryOperation) {
        // dbg!(&bo);

        if bo.operator == Operator::Mul {
            self.push_finding(0, Some(bo.src.clone()));
        }

        Ok(())
    }
}

#[test]
fn mul_ama() {
    let findings = compile_contract_and_get_findings(String::from(
        "pragma solidity 0.8.0;

contract Mul {
    function calculateStuff(uint256 a) public {
        uint256 val = a * 2;
    }
}",
    ));

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "maths", 0),
        vec![5]
    );
}
