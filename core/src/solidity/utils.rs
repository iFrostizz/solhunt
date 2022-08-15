use ethers_solc::artifacts::ast::SourceLocation;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub struct Position {
    pub line: u32, // line num
                   // pub position: u32, // horizontal pos
}

// Convert bytes source location to line & location for easier reference
// This function runs a dichotomy algorithm in order to find the correct line position.
// TODO: caching with check for best match to save steps
pub fn get_line_position(src: &SourceLocation, lines_to_bytes: &Vec<usize>) -> Option<usize> {
    lines_to_bytes.iter().enumerate().find_map(|(l, b)| {
        (src.start.unwrap_or(0) >= *b && src.start.unwrap_or(0) < lines_to_bytes[l + 1])
            .then_some(l)
    }) // unwrap_or(0) should work but not ideal
}

// Scan the file to get the bytes position of each line start
pub fn get_file_lines(mut file: BufReader<File>) -> eyre::Result<Vec<usize>> {
    let mut acc = Vec::new();
    let mut buf = String::new();
    let mut pos = 0;

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
