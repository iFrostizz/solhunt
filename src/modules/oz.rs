// Find vulns from oz security reports
// https://github.com/OpenZeppelin/openzeppelin-contracts/security/advisories

use crate::{build_visitor, walker::Severity};

build_visitor! {
    fn visit_member_access(&mut self, member_access: &mut MemberAccess) {
        if member_access.member_name == "transfer" {
            self.findings.push(Finding {
                name: "oz".to_string(),
                description: "usage of transfer for an ERC20 token, use safeTransfer instead".to_string(),
                severity: Severity::Medium,
                src: Some(member_access.src.clone()),
                code: 0
            });
        }

        member_access.visit(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        solidity::ProjectFile,
        test::{compile_and_get_findings, lines_for_findings_with_code},
    };

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

        assert_eq!(lines_for_findings_with_code(&findings, "oz", 0), vec![11]);
    }
}
