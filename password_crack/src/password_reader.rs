use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use crate::errors::Errors;

pub struct PasswordReader {
    reader: BufReader<File>,
    line_buffer: String,
}
pub fn password_reader_count(dictionary_path: &PathBuf) -> Result<usize, Errors> {
    let file = File::open(dictionary_path).expect("Unable to open file");
    let mut reader = BufReader::new(file);
    let mut total_count = 0;
    let mut line_buffer = vec![];
    loop {
        // count line number without reallocating each line
        // read_until to avoid UTF-8 validation (unlike read_line which produce a String)
        let res = reader
            .read_until(b'\n', &mut line_buffer)
            .expect("Unable to read file");
        if res == 0 {
            break;
        }
        line_buffer.clear();
        total_count += 1;
    }
    Ok(total_count)
}
impl PasswordReader {
    pub fn new(dictionary_path: PathBuf) -> Self {
        let file = File::open(&dictionary_path).expect("Unable to open file");
        let reader = BufReader::new(file);
        PasswordReader {
            reader,
            line_buffer: String::new(),
        }
    }
}

impl Iterator for PasswordReader {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.line_buffer.clear();
            let res = self.reader.read_line(&mut self.line_buffer);
            match res {
                Ok(0) => return None,
                Ok(_) => {
                    if self.line_buffer.ends_with('\n') {
                        self.line_buffer.pop();
                        if self.line_buffer.ends_with('\r') {
                            self.line_buffer.pop();
                        }
                    }
                    return Some(self.line_buffer.clone());
                }
                Err(_) => continue,
            }
        }
    }
}
