// Inspired from: https://github.com/Picodes/4naly3er/blob/main/src/issues/M/centralizationRisk.ts

use crate::build_visitor;

build_visitor! {
    BTreeMap::from(
        [
           (0,
             FindingKey {
                 description: "Contracts have owners with privileged rights to perform admin tasks and need to be trusted to not perform malicious updates or drain funds.".to_string(),
                 summary: "Centralization of power".to_string(),
                 severity: Severity::Medium
             }
             )
        ]
    ),
    fn visit_function_definition(&mut self, function_definition: &mut FunctionDefinition) {
        // function_definition.modifiers.iter().for_each(|m| {
        // });

        // dbg!(&function_definition);

        function_definition.visit(self)
    },
    fn visit_modifier_invocation(&mut self, modifier_invocation: &mut ModifierInvocation) {
        // dbg!(&modifier_invocation);
        if let IdentifierOrIdentifierPath::IdentifierPath(modifier) = &modifier_invocation.modifier_name {
            let name = &modifier.name;
            // TODO: onlyRole
            if name == "onlyOwner" {
                self.push_finding(0, Some(modifier_invocation.src.clone()));
            }
        }

        modifier_invocation.visit(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        solidity::ProjectFile,
        test::{compile_and_get_findings, lines_for_findings_with_code},
    };

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
}"#,
            ),
        )]);

        // TODO: found at l.22 but is actually at 21.
        assert_eq!(
            lines_for_findings_with_code(&findings, "centralization", 0),
            vec![21]
        );
    }
}
