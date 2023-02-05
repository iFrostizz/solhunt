use ethers_solc::artifacts::ast::SourceLocation;
use foundry_common::fs;
use semver::{Error, Version};

#[allow(unused)]
pub struct Position {
    pub line: u32, // line num
                   // pub position: u32, // horizontal pos
}

/// Unwrap binary_search output as no error has to be hanlded and we don't need precise index for value
macro_rules! into_ok_or_err {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => e,
        }
    };
}

/// Convert bytes source location to line for easier reference
pub fn get_position(src: &SourceLocation, lines_to_bytes: &[usize]) -> usize {
    into_ok_or_err!(lines_to_bytes.binary_search(&(src.start.unwrap_or(0))))
}

/// Returns the source map from an absolute file path
pub fn get_path_lines(path: String) -> Result<Vec<usize>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(get_string_lines(content))
}

/// Build source map from the string content of a file by scanning for char return
pub fn get_string_lines(content: String) -> Vec<usize> {
    // index => bytes_offset
    // always starts at 0 (content start)
    let mut acc = vec![];

    let bytes = content.as_bytes();

    for (i, b) in bytes.iter().enumerate() {
        if b == &b'\n' {
            // LF (\n)
            acc.push(i + 1);
        } else if b == &b'\r' {
            if let Some(next_b) = bytes.get(i + 1) {
                if next_b == &b'\r' {
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

    let source_map = get_string_lines(content);

    assert_eq!(source_map.len(), 1);
    assert_eq!(source_map[0], 13);

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

    let source_map = get_string_lines(content);

    assert_eq!(source_map.len(), 4);
    assert_eq!(source_map[0], 13);
    assert_eq!(source_map[1], 34);
    assert_eq!(source_map[2], 52);
    assert_eq!(source_map[3], 70);
}
