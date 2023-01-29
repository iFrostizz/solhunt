use semver::Version;
use serde_json::value::Value;

#[allow(unused)]
pub fn version_from_literals(literals: Value) -> Version {
    let literals: Vec<String> = serde_json::from_value(literals).unwrap();
    // dbg!(&literals);
    let joined = literals.join("");
    let mut split: Vec<&str> = joined.split('.').collect();
    split[0] = "0";

    Version::parse(&split.join(".")).unwrap()
}

pub fn version_from_string_literals(literals: Vec<String>) -> Version {
    let mut pragma = literals;
    assert_eq!(pragma.remove(0), "solidity");
    // pragma.get_mut(0).replace(value)

    if pragma.get(0).unwrap() == "^" {
        pragma.remove(0);
    }

    let pragma: String = pragma.iter().map(|v| v.to_string()).collect();

    Version::parse(&pragma).expect("failed to parse the sem ver")
}
