use crate::{build_visitor, walker::is_unspecific_version};

build_visitor! {
    BTreeMap::from([
       (0,
            FindingKey {
                description: "Unspecific compiler version pragma. Please lock the compiler version to avoid unexpected compilation results" .to_string(),
                severity: Severity::Low,
            }
        )
    ]),

    fn visit_pragma_directive(&mut self, pragma_directive: &mut PragmaDirective) {
        if is_unspecific_version(pragma_directive.literals.clone()) {
            // dbg!(&pragma_directive.src.clone());
            self.push_finding(Some(pragma_directive.src.clone()), 0);
        } // else will need to check for "unchecked"

        pragma_directive.visit(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_of_transfer() {
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
}
