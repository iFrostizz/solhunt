use crate::{build_visitor, walker::is_unspecific_version};

build_visitor! {
    BTreeMap::from([
        (0,
            FindingKey {
                description: "Unspecific compiler version pragma. Please lock the compiler version to avoid unexpected compilation results" .to_string(),
                summary: "Unlocked compiler pragma".to_string(),
                severity: Severity::Low,
            }
        ),
        (1,
            FindingKey {
                description: "Do not use deprecated functions" .to_string(),
                summary: "Deprecated functions".to_string(),
                severity: Severity::Low,
            }
        )
    ]),

    fn visit_pragma_directive(&mut self, pragma_directive: &mut PragmaDirective) {
        if is_unspecific_version(pragma_directive.literals.clone()) {
            self.push_finding(0, Some(pragma_directive.src.clone()));
        }

        pragma_directive.visit(self)
    },

    fn visit_identifier(&mut self, identifier: &mut Identifier) {
        if identifier.name == "_setupRole" &&
            identifier.type_descriptions.type_identifier ==
                Some("t_function_internal_nonpayable$_t_bytes32_$_t_address_$returns$__$".to_string()) {
            self.push_finding(1, Some(identifier.src.clone()));
        }

        identifier.visit(self)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlocked_pragma() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("UnlockedPragma"),
            String::from(
                "pragma solidity ^0.8.0;

contract UnlockedPragma {
}",
            ),
        )]);

        assert_eq!(lines_for_findings_with_code(&findings, "misc", 0), vec![1]);
    }

    #[test]
    fn deprecated_functions() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("DeprecatedFunctions"),
            String::from(
                "pragma solidity ^0.8.0;

contract DeprecatedFunctions {
    function _setupRole(bytes32, address) internal virtual {}

    function doDeprecatedThings(bytes32 role, address account) public {
        _setupRole(role, account);
    }
}",
            ),
        )]);

        assert_eq!(lines_for_findings_with_code(&findings, "misc", 1), vec![7]);
    }
}
