use crate::test::has_with_code_at_line;
use crate::{solidity::compile_path_and_get_findings, test::has_with_code_file};
use ethers_solc::artifacts::Optimizer;

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

    // dbg!(&findings);

    findings.iter().for_each(|(m, f)| {
        println!("{m}: {}", f.len());
    });

    // https://github.com/code-423n4/2023-01-biconomy-findings/blob/main/data/Rolezn-G.md#gas2-state-variables-only-set-in-the-constructor-should-be-declared-immutable
    assert!(!has_with_code_file(
        &findings,
        "SmartAccountFactory.sol",
        "immutable",
        0
    ));

    // https://github.com/code-423n4/2023-01-biconomy-findings/blob/main/data/chrisdior4-G.md#g-01-use-custom-errors-instead-of-revert-strings
    assert!(has_with_code_at_line(
        &findings,
        "aa-4337/core/EntryPoint.sol",
        "custom_errors",
        0,
        36
    ));

    // https://github.com/code-423n4/2023-01-biconomy-findings/blob/main/data/Rolezn-G.md#gas3-setting-the-constructor-to-payable
    assert!(has_with_code_at_line(
        &findings,
        "Proxy.sol",
        "constructor",
        0,
        15
    ));
    assert!(has_with_code_at_line(
        &findings,
        "SmartAccountFactory.sol",
        "constructor",
        0,
        17
    ));
}
