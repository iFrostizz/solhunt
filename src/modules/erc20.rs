// Check for non-compliant code (e.g:
// Using safeApprove instead of ... : https://code4rena.com/reports/2022-06-badger/#n-01-safeapprove-is-deprecated

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
       (0,
            FindingKey {
                description: "Unsafe ERC20 operation(s), use `safeTransfer` instead" .to_string(),
                severity: Severity::Low,
            }
        ),
        (1,
            FindingKey {
                description: "Unsafe ERC20 operation(s), use `safeTransfer` instead" .to_string(),
                severity: Severity::Low,
            }
        ),
        (2,
            FindingKey {
                description: "Unsafe ERC20 operation(s), use `safeTransfer` instead" .to_string(),
                severity: Severity::Low,
            }
        ),
    ]),

    fn visit_member_access(&mut self, member_access: &mut MemberAccess) {
        let unsafe_ops = vec!["transfer".to_owned(), "transferFrom".to_owned(), "approve".to_owned()];
        let mem_name = &member_access.member_name;
        if (unsafe_ops).contains(mem_name) {
            self.push_finding(Some(member_access.src.clone()), 0)
        }

        member_access.visit(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
