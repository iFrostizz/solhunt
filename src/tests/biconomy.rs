use crate::test::{has_with_code_at_line, lines_for_findings_with_code_file_module};
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
    )
    .unwrap();

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
        "require",
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
    assert!(has_with_code_at_line(
        &findings,
        "aa-4337/core/BasePaymaster.sol",
        "constructor",
        0,
        20
    ));
    assert!(has_with_code_at_line(
        &findings,
        "libs/MultiSend.sol",
        "constructor",
        0,
        12
    ));
    assert!(has_with_code_at_line(
        &findings,
        "paymasters/BasePaymaster.sol",
        "constructor",
        0,
        20
    ));
    assert!(has_with_code_at_line(
        &findings,
        "paymasters/verifying/singleton/VerifyingSingletonPaymaster.sol",
        "constructor",
        0,
        35
    ));

    // https://github.com/code-423n4/2023-01-biconomy-findings/blob/main/data/Rolezn-G.md#gas4-duplicated-requirerevert-checks-should-be-refactored-to-a-modifier-or-function
    assert_eq!(
        lines_for_findings_with_code_file_module(&findings, "SmartAccount.sol", "require", 1),
        vec![167, 168, 170, 171, 262, 265, 286, 289, 348, 351]
    );

    assert_eq!(
        lines_for_findings_with_code_file_module(
            &findings,
            "SmartAccount.sol",
            "centralization",
            1
        ),
        vec![449, 455, 460, 465, 536]
    );
    assert_eq!(
        lines_for_findings_with_code_file_module(
            &findings,
            "aa-4337/core/BasePaymaster.sol",
            "centralization",
            1
        ),
        vec![24, 67, 75, 90, 99]
    );
    assert_eq!(
        lines_for_findings_with_code_file_module(
            &findings,
            "aa-4337/core/BasePaymaster.sol",
            "centralization",
            1
        ),
        vec![24, 67, 75, 90, 99]
    );
    assert!(has_with_code_at_line(
        &findings,
        "paymasters/verifying/singleton/VerifyingSingletonPaymaster.sol",
        "centralization",
        1,
        65
    ));
}
