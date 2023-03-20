use eyre::Context;
use semver::{Version, VersionReq};

// https://docs.soliditylang.org/en/v0.8.17/layout-of-source-files.html?#version-pragma
pub fn smallest_version_from_literals(literals: Vec<String>) -> Option<eyre::Result<Version>> {
    let mut pragma = literals;
    if pragma.remove(0) == "solidity" {
        let maybe_upper = pragma.remove(0);

        let version_str = if maybe_upper == "^" || maybe_upper == ">=" {
            // version is eq or more than ...
            pragma[0].clone() + &pragma[1]
        } else {
            // fixed pragma
            maybe_upper + &pragma[0]
        };

        Some(Version::parse(&version_str).wrap_err_with(|| "issue parsing version"))
    } else {
        None
    }
}

pub fn is_unspecific_version(literals: Vec<String>) -> bool {
    let mut pragma = literals;
    if pragma.remove(0) == "solidity" {
        let maybe_upper = pragma.remove(0);

        maybe_upper == "^" || maybe_upper == ">="
    } else {
        false
    }
}

#[allow(unused)]
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
fn parses_version_req_from_literals_1() {
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
fn parses_version_req_from_literals_2() {
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

#[test]
fn finds_smallest_version() {
    let ver = smallest_version_from_literals(vec![
        String::from("solidity"),
        String::from("^"),
        String::from("0.8"),
        String::from(".4"),
    ])
    .unwrap()
    .unwrap();

    assert_eq!(ver, Version::new(0, 8, 4));
    assert_eq!(ver.major, 0);
    assert_eq!(ver.minor, 8);
    assert_eq!(ver.patch, 4);

    let ver = smallest_version_from_literals(vec![
        String::from("solidity"),
        String::from(">="),
        String::from("0.4"),
        String::from(".22"),
        String::from("<"),
        String::from("0.9"),
        String::from(".0"),
    ])
    .unwrap()
    .unwrap();

    assert_eq!(ver, Version::new(0, 4, 22));
    assert_eq!(ver.major, 0);
    assert_eq!(ver.minor, 4);
    assert_eq!(ver.patch, 22);
}

#[test]
fn exp_pragma() {
    assert!(smallest_version_from_literals(vec![
        String::from("experimental"),
        String::from("ABIEncoderV2"),
    ])
    .is_none());
}
