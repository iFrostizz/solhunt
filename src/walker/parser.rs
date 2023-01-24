use semver::Version;
use serde_json::value::Value;

pub fn version_from_literals(literals: Value) -> Version {
    let literals: Vec<String> = serde_json::from_value(literals).unwrap();
    // dbg!(&literals);
    let joined = literals.join("");
    let mut split: Vec<&str> = joined.split(".").collect();
    split[0] = "0";

    Version::parse(&split.join(".")).unwrap()
}
