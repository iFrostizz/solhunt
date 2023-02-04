use semver::{Version, VersionReq};

pub fn version_from_string_literals(literals: Vec<String>) -> Result<VersionReq, semver::Error> {
    let mut pragma = literals;
    assert_eq!(pragma.remove(0), "solidity");

    let pragma = pragma.into_iter().collect::<String>();
    let mut pragma = pragma.replace('<', ", <");

    // not compliant ! >:( https://blockchainknowledge.in/understanding-pragma-in-solidity/
    if let Some(stippended) = pragma.strip_prefix('^') {
        let version = Version::parse(stippended).unwrap();
        let major = version.major;
        let excluded_minor = version.minor + 1;
        pragma.push_str(&format!(", <{major}.{excluded_minor}.0"));
    };

    VersionReq::parse(&pragma)
}

#[test]
fn parses_version_from_literals_1() {
    let ver = version_from_string_literals(vec![
        String::from("solidity"),
        String::from("^"),
        String::from("0.8"),
        String::from(".4"),
    ])
    .unwrap();

    assert!(ver.matches(&Version::new(0, 8, 4)));
    assert!(!ver.matches(&Version::new(0, 9, 0)));
}

#[test]
fn parses_version_from_literals_2() {
    let ver = version_from_string_literals(vec![
        String::from("solidity"),
        String::from(">="),
        String::from("0.4"),
        String::from(".22"),
        String::from("<"),
        String::from("0.9"),
        String::from(".0"),
    ])
    .unwrap();

    // very much old!
    assert!(!ver.matches(&Version::new(0, 3, 5)));
    assert!(ver.matches(&Version::new(0, 4, 22)));
    assert!(ver.matches(&Version::new(0, 6, 22)));
    assert!(ver.matches(&Version::new(0, 8, 18)));
    assert!(!ver.matches(&Version::new(0, 9, 0)));
    assert!(!ver.matches(&Version::new(0, 9, 1)));
}
