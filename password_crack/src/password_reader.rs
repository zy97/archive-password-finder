use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use crate::errors::Errors;

pub struct PasswordReader {
    reader: BufReader<File>,
    line_buffer: String,
}
// pub fn password_reader_count(dictionary_path: &PathBuf) -> Result<usize, Errors> {
//     let file = File::open(dictionary_path).expect("Unable to open file");
//     let mut reader = BufReader::new(file);
//     let mut total_count = 0;
//     let mut line_buffer = Vec::with_capacity(1024);
//     loop {
//         // count line number without reallocating each line
//         // read_until to avoid UTF-8 validation (unlike read_line which produce a String)
//         let res = reader.read_until(b'\n', &mut line_buffer)?;
//         if res == 0 {
//             break;
//         }
//         line_buffer.clear();
//         total_count += 1;
//     }
//     Ok(total_count)
// }
pub fn password_reader_count(dictionary_path: &PathBuf) -> Result<usize, Errors> {
    let file = File::open(dictionary_path).expect("Unable to open file");
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 4096];
    let mut line_count = 0;
    let mut byte_count = 0;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        byte_count += bytes_read;
        for i in 0..bytes_read {
            if buffer[i] == b'\n' {
                line_count += 1;
            }
        }
    }

    // Check if last line doesn't end with a newline character
    if buffer[byte_count % 4096 - 1] != b'\n' {
        line_count += 1;
    }
    Ok(line_count)
}

impl PasswordReader {
    pub fn new(dictionary_path: &Path) -> Self {
        let file = File::open(dictionary_path).expect("Unable to open file");
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
#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Instant};

    use crate::password_reader::password_reader_count;

    #[test]
    fn test1() {
        let path =
            PathBuf::from(r"C:\repo\archive-password-finder\xato-net-10-million-passwords.txt");

        let now = Instant::now();
        let count = password_reader_count(&path).unwrap();
        println!("{:<10}ms,{}", now.elapsed().as_millis(), count);
    }
}
