use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        (
            0,
            FindingKey {
                summary: "use ternary operators rather than `if/else`".to_string(),
                description: "`if/else` gas overhead is higher than a ternary operator".to_string(),
                severity: Severity::Gas }
        )
    ]),
}
