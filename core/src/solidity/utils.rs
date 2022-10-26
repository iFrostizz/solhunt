use ethers_solc::artifacts::ast::SourceLocation;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
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

// Convert bytes source location to line & location for easier reference
pub fn get_line_position(src: &SourceLocation, lines_to_bytes: &[u32]) -> usize {
    /*dbg!(src.start);
    dbg!(lines_to_bytes);
    dbg!(lines_to_bytes.binary_search(&(src.start.unwrap_or(0) as u32)));*/
    into_ok_or_err!(lines_to_bytes.binary_search(&(src.start.unwrap_or(0) as u32)))
}

// Scan the file to get the bytes position of each line start
pub fn get_file_lines(mut file: BufReader<File>) -> Result<Vec<u32>, std::io::Error> {
    let mut acc = Vec::new();
    let mut buf = String::new();
    let mut pos: u32 = 0;

    dbg!("test");

    // dbg!(&file.read_line(&mut buf));
    /*let mut a = String::new();
    file.read_to_string(&mut a).unwrap();
    dbg!(&a);*/

    loop {
        match file.read_line(&mut buf)? {
            0 => break,
            n => {
                acc.push(pos);
                pos += n as u32;
                // dbg!(&pos);
            }
        }
        buf.clear();
    }

    Ok(acc)
}
