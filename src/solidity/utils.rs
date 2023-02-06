use foundry_common::fs;
use semver::{Error, Version};

#[allow(unused)]
pub struct Position {
    pub line: u32, // line num
                   // pub position: u32, // horizontal pos
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
    }

    (line, width)
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

    let source_map = get_string_lines(content);

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

    let source_map = get_string_lines(content);
    assert_eq!(get_position(13, &source_map), (1, 14));
    assert_eq!(get_position(14, &source_map), (2, 1));
    assert_eq!(get_position(25, &source_map), (2, 12));
    assert_eq!(get_position(72, &source_map), (5, 2));
}
