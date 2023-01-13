use ethers_solc::artifacts::ast::SourceLocation;
use foundry_common::fs;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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

/// Convert bytes source location to line & location for easier reference
pub fn get_line_position(src: &SourceLocation, lines_to_bytes: &[usize]) -> usize {
    into_ok_or_err!(lines_to_bytes.binary_search(&(src.start.unwrap_or(0))))
}

/// Returns the source map from an absolute file path
pub fn get_path_lines(path: String) -> Result<Vec<usize>, std::io::Error> {
    // Maybe should pass the BufReader instead ?
    let content = fs::read_to_string(path)?;
    Ok(get_string_lines(content))
}

/// Scan the file to get the bytes position of each line start
pub fn get_file_lines(mut file: BufReader<File>) -> Result<Vec<usize>, std::io::Error> {
    let mut acc = vec![];
    let mut buf = String::new();
    let mut pos: usize = 0;

    loop {
        match file.read_line(&mut buf)? {
            0 => break,
            n => {
                acc.push(pos);
                pos += n;
            }
        }
        buf.clear();
    }

    Ok(acc)
}

/// Build source map from the string content of a file by scanning for char return
pub fn get_string_lines(content: String) -> Vec<usize> {
    let mut acc = vec![];
    let mut pos: usize = 0;

    content.lines().for_each(|l| {
        acc.push(pos);
        pos += l.len();
    });

    acc
}

pub fn path_from_id(id: String) -> String {
    id.rsplit_once(":")
        .expect(&format!("Malformed id `{}`", &id))
        .0
        .to_string()
}
