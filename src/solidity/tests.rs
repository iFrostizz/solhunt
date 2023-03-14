use crate::{
    solidity::{
        get_finding_content, get_finding_content_after, get_finding_content_before,
        get_finding_content_middle, get_position, get_source_map, version_from_source,
    },
    walker::ModuleState,
};
use ethers_solc::artifacts::{
    ast::SourceLocation,
    visitor::{VisitError, Visitor},
    InlineAssembly,
};
use semver::Version;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

#[test]
fn parses_versions() {
    let source = String::from("pragma solidity ^0.8.0;");
    let req = version_from_source(source).unwrap();

    assert!(!req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
    assert!(!req.matches(&Version::new(0, 9, 0)));

    let source = String::from("pragma solidity =0.8.0;");
    let req = version_from_source(source).unwrap();

    assert!(!req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
    assert!(!req.matches(&Version::new(0, 8, 1)));

    let source = String::from("pragma solidity 0.8.0;");
    let req = version_from_source(source).unwrap();

    assert!(!req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
    assert!(!req.matches(&Version::new(0, 8, 1)));

    let source = String::from("pragma solidity >=0.8.0;");
    let req = version_from_source(source).unwrap();

    assert!(!req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
    assert!(req.matches(&Version::new(0, 9, 0)));
    assert!(req.matches(&Version::new(1, 0, 0)));

    let source = String::from("pragma solidity >=0.5.0 <0.8.0;");
    let req = version_from_source(source).unwrap();

    assert!(req.matches(&Version::new(0, 7, 0)));
    assert!(!req.matches(&Version::new(0, 8, 0)));

    let source = String::from("pragma solidity >=0.5.0 <=0.8.0;");
    let req = version_from_source(source).unwrap();

    assert!(req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
}

#[test]
fn get_smallest() {}

#[test]
fn correct_source_maps() {
    let content = String::from(
        "Hello, world
Goodbye, world",
    );

    let source_map = get_source_map(&content);

    assert_eq!(source_map.len(), 2);
    assert_eq!(source_map[0], 13);
    assert_eq!(source_map[1], 27);

    // Solhunt is a\n
    // new static analyzer,\n
    // ...
    let content = String::from(
        "Solhunt is a
new static analyzer,
it lets you write
detection modules
and it's fast!",
    );

    let source_map = get_source_map(&content);

    assert_eq!(source_map.len(), 5);
    assert_eq!(source_map[0], 13);
    assert_eq!(source_map[1], 34);
    assert_eq!(source_map[2], 52);
    assert_eq!(source_map[3], 70);
    assert_eq!(source_map[4], 84);
}

#[test]
fn correct_position_map() {
    let content = String::from(
        "Solhunt is a
new static analyzer,
it lets you write
detection modules
and it's fast!",
    );

    /*
         [
             13,
             34,
             52,
             70
         ]
    */

    let source_map = get_source_map(&content);
    assert_eq!(get_position(13, &source_map), (1, 14));
    assert_eq!(get_position(14, &source_map), (2, 1));
    assert_eq!(get_position(25, &source_map), (2, 12));
    assert_eq!(get_position(72, &source_map), (5, 2));
}

#[test]
fn outputs_before_finding_content() {
    let content = String::from(
        "Solhunt is a
new static analyzer,
it lets you write
detection modules
and it's fast!",
    );

    let file_bytes = content.as_bytes();

    let before_content = get_finding_content_before(file_bytes, 14);
    assert_eq!(before_content, String::from("Solhunt is a\n"));

    let before_content = get_finding_content_before(file_bytes, 0);
    assert_eq!(before_content, String::from(""));
}

#[test]
fn outputs_middle_finding_content() {
    let content = String::from(
        "Solhunt is a
new static analyzer,
it lets you write
detection modules
and it's fast!",
    );

    let file_bytes = content.as_bytes();

    let middle_content = get_finding_content_middle(file_bytes, 14, 3);
    assert_eq!(middle_content, String::from("new static analyzer,\n"));

    let middle_content = get_finding_content_middle(file_bytes, 14, 24);
    assert_eq!(
        middle_content,
        String::from(
            "new static analyzer,
it lets you write\n"
        )
    );

    let middle_content = get_finding_content_middle(file_bytes, 0, 6);
    assert_eq!(middle_content, String::from("Solhunt is a\n"));
}

#[test]
fn outputs_after_finding_content() {
    let content = String::from(
        "Solhunt is a
new static analyzer,
it lets you write
detection modules
and it's fast!",
    );

    let file_bytes = content.as_bytes();

    let after_content = get_finding_content_after(file_bytes, 14, 3);
    assert_eq!(after_content, String::from("it lets you write\n"));

    let after_content = get_finding_content_after(file_bytes, 0, 5);
    assert_eq!(after_content, String::from("new static analyzer,\n"));
}

#[test]
fn outputs_finding_content() {
    let content = String::from(
        "Solhunt is a
new static analyzer,
it lets you write
detection modules
and it's fast!",
    );

    let finding_content = get_finding_content(content.clone(), 14, 6);
    assert_eq!(
        finding_content,
        String::from(
            "Solhunt is a
new static analyzer,
it lets you write\n"
        )
    );

    let finding_content = get_finding_content(content.clone(), 0, 6);
    assert_eq!(
        finding_content,
        String::from(
            "Solhunt is a
new static analyzer,\n"
        )
    );

    let finding_content = get_finding_content(content.clone(), 35, 2);
    assert_eq!(
        finding_content,
        String::from(
            "new static analyzer,
it lets you write
detection modules\n"
        )
    );

    let finding_content = get_finding_content(content.clone(), 71, 38);
    assert_eq!(
        finding_content,
        String::from(
            "detection modules
and it's fast!"
        )
    );

    let finding_content = get_finding_content(content, 18, 21);
    assert_eq!(
        finding_content,
        String::from(
            "Solhunt is a
new static analyzer,
it lets you write
detection modules\n"
        )
    );
}

pub struct SourceModule {
    shared_data: ModuleState,
    expected_source: SourceLocation,
}

#[cfg(test)]
impl SourceModule {
    fn new(expected_source: SourceLocation) -> Self {
        Self {
            expected_source,
            shared_data: Default::default(),
        }
    }
}

impl Visitor<ModuleState> for SourceModule {
    fn shared_data(&mut self) -> &ModuleState {
        &self.shared_data
    }

    fn visit_inline_assembly(
        &mut self,
        inline_assembly: &mut InlineAssembly,
    ) -> eyre::Result<(), VisitError> {
        assert_eq!(inline_assembly.src, self.expected_source);

        Ok(())
    }
}

#[cfg(test)]
use crate::walker::Walker;

/// Test if source locations are matching with our calculations on some representatives nodes
#[test]
fn ast_source_locations() {
    let content = String::from(
        "pragma solidity 0.8.0;

contract SourceLocations {
    function doAssembly() public {
        assembly {


        }
    }
}",
    );

    let (project, artifacts) = super::compile_single_contract_to_artifacts(content.clone());

    // note: solidity source mappings are giving the byte before the first character
    let source = SourceLocation {
        start: Some(94),
        length: Some(22),
        index: Some(0),
    };

    let module = SourceModule::new(source.clone());

    let mut walker = Walker::new(
        artifacts,
        BTreeMap::new(),
        vec![Rc::from(RefCell::from(module))],
        project.root().into(),
    );

    walker.traverse().unwrap();

    let finding_content = get_finding_content_middle(
        content.as_bytes(),
        source.start.unwrap(),
        source.length.unwrap(),
    );

    assert_eq!(
        finding_content,
        String::from(
            "        assembly {


        }\n"
        )
    );
}
