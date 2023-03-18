// Inspired from: https://github.com/Picodes/4naly3er/blob/main/src/issues/M/centralizationRisk.ts

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        (
            0,
                FindingKey {
                    description: "Contracts have owners with privileged rights to perform admin tasks and need to be trusted to not perform malicious updates or drain funds.".to_string(),
                    summary: "Centralization of power".to_string(),
                    severity: Severity::Medium
                }
            ),
        (
            1,
            FindingKey {
                    summary: "Functions guaranteed to revert for normal users should be marked as `payable`".to_string(),
                    description: "When a function is restricted to only one account, it's less expensive to mark it as ".to_string(),
                    severity: Severity::Gas
            }
        )
    ]),
    fn visit_modifier_invocation(&mut self, modifier_invocation: &mut ModifierInvocation) {
        if let IdentifierOrIdentifierPath::IdentifierPath(modifier) = &modifier_invocation.modifier_name {
            let name = &modifier.name;
            // TODO: onlyRole
            if name == "onlyOwner" {
                self.push_finding(0, Some(modifier_invocation.src.clone()));
                self.push_finding(1, Some(modifier_invocation.src.clone()));
            }
        }

        // modifier_invocation.visit(self)
        Ok(())
    }
}

#[test]
fn only_owner_modifier() {
    // TODO: use remappings to reuse OnlyOwner ?
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("OnlyOwnerModifier"),
        String::from(
            r#"pragma solidity ^0.8.0;

contract Ownable {
    address private _owner;

    constructor() {
        _owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == _owner, "Not owner");
        _;
    }

    function owner() public view returns (address) {
        return _owner;
    }
}

contract OnlyOwnerModifer is Ownable {
    function rugEverybody() external onlyOwner {
        payable(owner()).transfer(address(this).balance);
    }
}
"#,
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "centralization", 0),
        vec![21]
    );

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "centralization", 1),
        vec![21]
    );
}
