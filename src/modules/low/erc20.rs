// Check for non-compliant code (e.g:
// Using safeApprove instead of ... : https://code4rena.com/reports/2022-06-badger/#n-01-safeapprove-is-deprecated

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
       (0,
            FindingKey {
                description: "Unsafe ERC20 operation(s), use `safeTransfer` instead" .to_string(),
                summary: "Unsafe ERC20 operation".to_string(),
                severity: Severity::Low,
            }
        ),
        (1,
            FindingKey {
                description: "Unsafe ERC20 operation(s), use `safeIncreaseAllowance` instead" .to_string(),
                summary: "Unsafe ERC20 operation".to_string(),
                severity: Severity::Low,
            }
        ),
        (2,
            FindingKey {
                description: "Unsafe ERC20 operation(s), use `safeTransfer` instead" .to_string(),
                summary: "Unsafe ERC20 operation".to_string(),
                severity: Severity::Low,
            }
        ),
    ]),

    fn visit_member_access(&mut self, member_access: &mut MemberAccess) {
        let unsafe_ops = vec!["transfer".to_owned(), "transferFrom".to_owned(), "approve".to_owned()];
        let mem_name = &member_access.member_name;
        let type_d = &member_access.type_descriptions;

        if mem_name == "transfer"   {
            if let Some(type_string) = &type_d.type_string {
                if type_string.contains("(address,uint256)") {
            self.push_finding(0, Some(member_access.src.clone()));
                }
            }
        }

        if (unsafe_ops).contains(mem_name) {
        } else if mem_name == "safeApprove" {
            self.push_finding(1, Some(member_access.src.clone()))
        }

        member_access.visit(self)
    }
}

#[test]
fn usage_of_transfer() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("SafeTransfer"),
        String::from(
            "pragma solidity 0.8.0;

interface IERC20 {
  function transfer(address, uint256) external view returns (bool);
}

contract SafeTransfer {
    address immutable owner = msg.sender;

    function pull(IERC20 token) public view returns (int256) {
        token.transfer(owner, 100);
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code(&findings, "erc20", 0),
        vec![11]
    );
}

// https://github.com/Picodes/4naly3er/blob/main/src/issues/L/deprecatedFunctions.ts
#[test]
fn deprecated_safe_approve() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("SafeApprove"),
        String::from(
            "pragma solidity ^0.8.0;

interface IERC20 {
    function safeApprove(address, uint256) external;
}

contract SafeApprove {
    function approve(IERC20 token) public {
        token.safeApprove(address(0), 123456); 
    }
}",
        ),
    )]);

    assert_eq!(lines_for_findings_with_code(&findings, "erc20", 1), vec![9]);
}

#[test]
fn eth_transfer() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("EthTransfer"),
        String::from(
            "pragma solidity ^0.8.0;

contract EthTransfer {
    function ethTransfer() public {
        payable(msg.sender).transfer(address(this).balance);
    }
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "erc20", 0));
}
