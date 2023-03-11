use crate::walker::ModuleState;
use ethers_solc::{
    artifacts::{
        ast::SourceLocation,
        visitor::{VisitError, Visitor},
        InlineAssembly,
    },
    AggregatedCompilerOutput,
};
use semver::{Error, Version, VersionReq};
use std::{collections::BTreeMap, fs};

pub fn build_source_maps(
    output: AggregatedCompilerOutput,
) -> BTreeMap<String, (String, Vec<usize>)> {
    output
        .contracts
        .iter()
        .map(|(id, _)| {
            let abs_path = id.to_string();

            let file_content = fs::read_to_string(abs_path.clone())
                .unwrap_or_else(|e| panic!("Failed to open file `{abs_path}` {e}"));
            (
                abs_path,
                (file_content.clone(), get_source_map(&file_content)),
            )
        })
        .collect()
}

/// Unwrap binary_search output as no error has to be hanlded and we don't need precise index for value
// macro_rules! into_ok_or_err {
//     ($res:expr) => {
//         match $res {
//             Ok(val) => val,
//             Err(e) => e,
//         }
//     };
// }

/// Convert bytes source location to line for easier reference
pub fn get_position(start: usize, lines_to_bytes: &[usize]) -> (usize, usize) {
    let mut line = 0;
    let mut line_start = 0;
    let mut width = 0;

    for (i, b) in lines_to_bytes.iter().enumerate() {
        // find the first time that start is in the bounds of the bytes offset
        // remove the total bytes offset from the start of the line to get the width

        if start <= *b {
            line = i + 1;
            width = start - line_start + 1;
            break;
        }

        // update offset of the start of the line
        line_start = b + 1;

        // TODO: should instead get the bytes representation of the line
    }

    (line, width)
}

/// Returns a view in the code where the finding is located
pub fn get_finding_content(content: String, start: usize, length: usize) -> String {
    let file_bytes: Vec<u8> = content.as_bytes().to_vec();

    let mut content = String::new();

    content.push_str(&get_finding_content_before(&file_bytes, start));
    content.push_str(&get_finding_content_middle(&file_bytes, start, length));
    content.push_str(&get_finding_content_after(&file_bytes, start, length));

    content
}

pub fn get_finding_content_before(file_bytes: &[u8], start: usize) -> String {
    let last_i = get_last_start_index(file_bytes, start);
    if last_i > 0 {
        let start_line = get_last_start_index(file_bytes, start).saturating_sub(1);
        let start_bef_line = get_last_start_index(file_bytes, start_line);
        content_until_end(file_bytes, start_bef_line)
    } else {
        String::new()
    }
}

/// returns the content corresponding to the line where the "start" is
pub fn get_finding_content_middle(file_bytes: &[u8], start: usize, length: usize) -> String {
    let start_line_byte = get_last_start_index(file_bytes, start);

    // max is excluded in the range, so we should do index + 1
    let max = if file_bytes.len() < start_line_byte + length {
        // give the last available index if it would go over the array
        file_bytes.len()
    } else {
        offset_until_end(file_bytes, start_line_byte + length) + 1
    };

    let content = file_bytes.get(start_line_byte..max).unwrap_or_default();
    // .unwrap_or_else(|| panic!("{:#?}", start_line_byte..max));

    String::from_utf8(content.to_vec()).unwrap()
}

/// Get the byte index of the last line start
fn get_last_start_index(file_bytes: &[u8], start: usize) -> usize {
    let mut i = start;

    while i > 0 {
        if file_bytes.get(i - 1).unwrap_or(&b'\n') == &b'\n' {
            break;
        }
        i -= 1;
    }

    i
}

pub fn get_finding_content_after(file_bytes: &[u8], start: usize, length: usize) -> String {
    let content_i = offset_until_end(file_bytes, start + length);
    // overtake the current line by adding 1
    content_until_end(file_bytes, content_i + 1)
}

/// Get the next index for which the char is \n
fn offset_until_end(file_bytes: &[u8], start: usize) -> usize {
    let mut i = start;

    // Iterate until the next char return \n
    let mut last_byte = *file_bytes.get(i).unwrap_or(&0);

    // Either returns because we found a \n or the end of the buffer
    while last_byte != b'\n' {
        i += 1;

        if let Some(c) = file_bytes.get(i) {
            last_byte = *c;
        } else {
            break;
        }
    }

    i
}

fn content_until_end(file_bytes: &[u8], index: usize) -> String {
    let mut i = index;

    // Iterate until the next char return \n
    let mut last_byte = 0;
    while last_byte != b'\n' {
        if let Some(c) = file_bytes.get(i) {
            i += 1;
            last_byte = *c;
        } else {
            break;
        }
    }

    let content_with_length = file_bytes.get(index..i).unwrap_or_default();
    String::from_utf8(content_with_length.to_vec()).unwrap()
}

/// Returns the source map from an absolute file path
#[allow(unused)]
pub fn get_path_lines(path: String) -> Result<Vec<usize>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(get_source_map(&content))
}

/// Build source map from the string content of a file by scanning for char return
pub fn get_source_map(content: &String) -> Vec<usize> {
    // index => bytes_offset
    // always starts at 0 (content start)
    let mut acc = vec![];

    let bytes = content.as_bytes();

    let last_i = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i + 1 == last_i {
            acc.push(last_i);
            break;
        }
        if b == &b'\n' {
            // LF (\n)
            acc.push(i + 1);
        } else if b == &b'\r' {
            if let Some(next_b) = bytes.get(i + 1) {
                if next_b == &b'\n' {
                    // CRLF (\r\n)
                    acc.push(i + 1);

                    // also skip this char
                    continue;
                }
            }
        }
    }

    acc
}

/// Extract the path from the unique identifier
#[allow(unused)]
pub fn path_from_id(id: String) -> String {
    id.rsplit_once(':')
        .unwrap_or_else(|| panic!("Malformed id `{}`", &id))
        .0
        .to_string()
}

pub fn get_pragma_from_source(source: String) -> Option<String> {
    source
        .lines()
        .find(|l| l.contains("pragma solidity "))
        .map(|s| {
            s.to_string()
                .strip_prefix("pragma solidity ")
                .unwrap()
                // .trim()
                .to_owned()
        })
}

pub fn version_from_source(source: String) -> Result<VersionReq, semver::Error> {
    let mut pragma_str = get_pragma_from_source(source).unwrap();

    if pragma_str.chars().next().unwrap().is_ascii_digit() {
        pragma_str = "=".to_owned() + &pragma_str;
    }

    // add comma before the < after the first bound (if any)
    for (i, c) in pragma_str.clone().chars().enumerate() {
        if let Some(cp) = pragma_str.chars().nth(i + 1) {
            if c.is_ascii_digit() && (cp.is_whitespace() || cp == '<') {
                pragma_str.insert(i + 1, ',');
                break;
            }
        }
    }

    VersionReq::parse(&pragma_str)
}

pub fn char_pos(chars: String, of: char, from: usize) -> Option<usize> {
    chars
        .chars()
        .enumerate()
        .filter(|(i, _)| *i >= from)
        .position(|(_, c)| c == of)
}

// TODO: file:///home/franfran/Projects/own/solhunt/target/doc/ethers_solc/utils/fn.find_version_pragma.html
#[test]
fn parses_versions() {
    let source = String::from("pragma solidity ^0.8.0");
    let req = version_from_source(source).unwrap();

    assert!(!req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
    assert!(!req.matches(&Version::new(0, 9, 0)));

    let source = String::from("pragma solidity =0.8.0");
    let req = version_from_source(source).unwrap();

    assert!(!req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
    assert!(!req.matches(&Version::new(0, 8, 1)));

    let source = String::from("pragma solidity 0.8.0");
    let req = version_from_source(source).unwrap();

    assert!(!req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
    assert!(!req.matches(&Version::new(0, 8, 1)));

    let source = String::from("pragma solidity >=0.8.0");
    let req = version_from_source(source).unwrap();

    assert!(!req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
    assert!(req.matches(&Version::new(0, 9, 0)));
    assert!(req.matches(&Version::new(1, 0, 0)));

    let source = String::from("pragma solidity >=0.5.0 <0.8.0");
    let req = version_from_source(source).unwrap();

    assert!(req.matches(&Version::new(0, 7, 0)));
    assert!(!req.matches(&Version::new(0, 8, 0)));

    let source = String::from("pragma solidity >=0.5.0 <=0.8.0");
    let req = version_from_source(source).unwrap();

    assert!(req.matches(&Version::new(0, 7, 0)));
    assert!(req.matches(&Version::new(0, 8, 0)));
}

#[allow(unused)]
fn parse_literals(literals: Vec<String>) -> Result<Version, Error> {
    Version::parse(
        literals
            .iter()
            .flat_map(|literal| {
                literal
                    .chars()
                    .filter(|char| char.is_ascii_digit() || char.to_string() == ".")
            })
            .collect::<String>()
            .as_str(),
    )
}

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
        vec![Box::from(module)],
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
