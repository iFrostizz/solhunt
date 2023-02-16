use ethers_solc::artifacts::Optimizer;

use crate::solidity::compile_path_and_get_findings;
use crate::test::has_with_code_file;

#[test]
fn biconomy_integration() {
    let findings = compile_path_and_get_findings(
        "test-data/biconomy/",
        Some(Optimizer {
            enabled: Some(true),
            runs: Some(200),
            details: None,
        }),
    );

    assert!(!has_with_code_file(
        &findings,
        "SmartAccountFactory.sol",
        "immutable",
        0
    ))
}
