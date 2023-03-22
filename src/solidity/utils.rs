use ethers_solc::{
    artifacts::Source, compile::Solc, error::SolcError, AggregatedCompilerOutput, ArtifactId,
    ConfigurableContractArtifact,
};
use semver::{Version, VersionReq};
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

pub fn build_artifacts_source_maps(
    artifacts: &BTreeMap<ArtifactId, ConfigurableContractArtifact>,
) -> BTreeMap<String, (String, Vec<usize>)> {
    artifacts
        .keys()
        .map(|id| {
            let abs_path = id.source.clone().to_str().unwrap().to_string();

            let file_content = fs::read_to_string(abs_path.clone())
                .unwrap_or_else(|e| panic!("Failed to open file `{}` {e}", abs_path));
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

/// Convert bytes source location to (line, width) for easier reference
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

/// Returns a view in the code where the finding is located with an arrow showing where it is
pub fn get_finding_content_arrow(content: String, start: usize, length: usize) -> String {
    let file_bytes: Vec<u8> = content.as_bytes().to_vec();

    let mut content = String::new();

    content.push_str(&(String::from("  ") + &get_finding_content_before(&file_bytes, start)));

    let mid = get_finding_content_middle(&file_bytes, start, length);
    let mid = mid
        .lines()
        .map(|l| String::from("> ") + l + "\n")
        .collect::<String>();

    content.push_str(&mid);
    content
        .push_str(&(String::from("  ") + &get_finding_content_after(&file_bytes, start, length)));

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

pub fn version_from_source(source: String) -> std::result::Result<VersionReq, SolcError> {
    Solc::source_version_req(&Source { content: source })
}

pub fn equi_ver(ver1: &Version, ver2: &Version) -> bool {
    ver1.major == ver2.major && ver1.minor == ver2.minor && ver1.patch == ver2.patch
}
