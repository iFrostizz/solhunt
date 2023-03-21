use eyre::Context;
use semver::Version;

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
