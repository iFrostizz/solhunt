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

/// Returns a view in the code where the finding is located
pub fn get_finding_content(
    content: String,
    start: usize,
    length: usize,
    lines_to_bytes: &[usize],
) -> String {
    let file_bytes: Vec<u8> = content.as_bytes().to_vec();

    let mut content = String::new();

    content.push_str(&get_finding_content_before(
        &file_bytes,
        start,
        lines_to_bytes,
    ));
    content.push_str(&get_finding_content_middle(&file_bytes, start, length));
    content.push_str(&get_finding_content_after(
        &file_bytes,
        start,
        lines_to_bytes,
    ));

    content
}

pub fn get_finding_content_before(
    file_bytes: &[u8],
    start: usize,
    lines_to_bytes: &[usize],
) -> String {
    if start > lines_to_bytes[0] {
        let start_line = get_last_start_index(file_bytes, start).saturating_sub(1);
        let start_bef_line = get_last_start_index(file_bytes, start_line);
        content_until_end(file_bytes, start_bef_line)
    } else {
        String::new()
    }
}

pub fn get_finding_content_middle(file_bytes: &[u8], start: usize, length: usize) -> String {
    let start_line_byte = get_last_start_index(file_bytes, start);

    let max = if file_bytes.len() < start_line_byte + length {
        // give the last available index
        file_bytes.len()
    } else {
        offset_until_end(file_bytes, start_line_byte + length)
    };

    let content = file_bytes
        .get(start_line_byte..max)
        .unwrap_or_else(|| panic!("{:#?}", start_line_byte..max));

    String::from_utf8(content.to_vec()).unwrap()
}

/// Get the byte index of the last line start
fn get_last_start_index(file_bytes: &[u8], start: usize) -> usize {
    // Avoid starting on the end of a line
    // let mut i = start.saturating_sub(1);
    let mut i = start;

    while i > 0 {
        if file_bytes[i - 1] == b'\n' {
            break;
        }
        i -= 1;
    }

    i
}

pub fn get_finding_content_after(
    file_bytes: &[u8],
    start: usize,
    lines_to_bytes: &[usize],
) -> String {
    let (line, _) = get_position(start, lines_to_bytes);
    match line.checked_sub(1) {
        Some(line_after) => {
            let index = *lines_to_bytes.get(line_after).unwrap();
            content_until_end(file_bytes, index)
        }
        None => String::new(),
    }
}

fn offset_until_end(file_bytes: &[u8], index: usize) -> usize {
    let mut i = index;

    // Iterate until the next char return \n
    let mut last_byte = 0;
    // Either returns because we found a \n or the end of the buffer
    while last_byte != b'\n' {
        if let Some(c) = file_bytes.get(i) {
            i += 1;
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

    let content_with_length = file_bytes.get(index..i).unwrap();
    String::from_utf8(content_with_length.to_vec()).unwrap()
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

#[test]
fn outputs_before_finding_content() {
    let content = String::from(
        "Solhunt is a
new static analyzer,
it lets you write
detection modules
and it's fast!",
    );

    let lines_to_bytes = &get_string_lines(content.clone());
    let file_bytes = content.as_bytes();

    let before_content = get_finding_content_before(file_bytes, 14, lines_to_bytes);
    assert_eq!(before_content, String::from("Solhunt is a\n"));

    let before_content = get_finding_content_before(file_bytes, 0, lines_to_bytes);
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

    let lines_to_bytes = &get_string_lines(content.clone());
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

    let lines_to_bytes = &get_string_lines(content.clone());
    let file_bytes = content.as_bytes();

    let after_content = get_finding_content_after(file_bytes, 14, lines_to_bytes);
    assert_eq!(after_content, String::from("it lets you write\n"));

    let after_content = get_finding_content_after(file_bytes, 0, lines_to_bytes);
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

    let lines_to_bytes = get_string_lines(content.clone());

    let finding_content = get_finding_content(content.clone(), 14, 6, &lines_to_bytes);
    assert_eq!(
        finding_content,
        String::from(
            "Solhunt is a
new static analyzer,
it lets you write\n"
        )
    );

    let finding_content = get_finding_content(content.clone(), 0, 6, &lines_to_bytes);
    assert_eq!(
        finding_content,
        String::from(
            "Solhunt is a
new static analyzer,\n"
        )
    );

    let finding_content = get_finding_content(content.clone(), 35, 2, &lines_to_bytes);
    assert_eq!(
        finding_content,
        String::from(
            "new static analyzer,
it lets you write
detection modules\n"
        )
    );

    let finding_content = get_finding_content(content, 71, 38, &lines_to_bytes);
    assert_eq!(
        finding_content,
        String::from(
            "detection modules
and it's fast!"
        )
    );
}
