use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam_channel::{Receiver, Sender};
use indicatif::ParallelProgressIterator;
use rayon::{prelude::ParallelIterator, str::ParallelString};

use crate::{
    password_finder::Strategy,
    password_worker::{pdf_password_checker, rar_password_checker, sevenz_password_checker},
    progress_bar::create_progress_bar,
    zip::zip_utils::validate_zip,
    PasswordFinder,
};

pub struct PasswordReader {
    reader: BufReader<File>,
    line_buffer: String,
}
pub fn password_dictionary_reader_iter(dictionary_path: PathBuf) -> impl Iterator<Item = String> {
    PasswordReader::new(dictionary_path)
}
pub fn password_reader_count(dictionary_path: PathBuf) -> Result<usize, std::io::Error> {
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

impl PasswordFinder for PasswordReader {
    fn find_password(self: &Self, file_path: PathBuf, strategy: Strategy) -> Option<String> {
        let kind = infer::get_from_path(&file_path).unwrap();
        let zip_file = fs::read(&file_path)
            .expect(format!("Failed reading the ZIP file: {}", file_path.display()).as_str());
        let password_count = 1;
        let progress_bar = create_progress_bar(password_count as u64);
        // let pbi = self.par_lines().progress_with(progress_bar);
        let sdf = String::from("value");
        let pbi = sdf.par_lines().progress_with(progress_bar.clone());
        match kind {
            Some(archive) if archive.mime_type() == "application/vnd.rar" => {
                return pbi
                    .find_map_any(|password| {
                        rar_password_checker(&password, file_path.display().to_string())
                    })
                    .map(|f| f.to_string());
            }
            Some(archive) if archive.mime_type() == "application/zip" => {
                let aes_info = validate_zip(&file_path.as_path()).unwrap();
                let workers = 4;
                let mut worker_handles = Vec::with_capacity(workers);
                // stop signals to shutdown threads
                let stop_workers_signal = Arc::new(AtomicBool::new(false));
                let stop_gen_signal = Arc::new(AtomicBool::new(false));
                let (send_found_password, receive_found_password): (
                    Sender<String>,
                    Receiver<String>,
                ) = crossbeam_channel::bounded(1);
                let total_password_count = match &strategy {
                    Strategy::GenPasswords {
                        charsets,
                        min_password_len,
                        max_password_len,
                    } => todo!(),
                    Strategy::PasswordFile(password_list_path) => {
                        let total =
                            password_reader_count(password_list_path.to_path_buf()).unwrap();
                        progress_bar.println(format!(
                            "Using passwords dictionary {password_list_path:?} with {total} candidates."
                        ));
                        total
                    }
                };
                progress_bar.set_length(total_password_count as u64);
                // progress_bar.println(format!("Starting {workers} workers to test passwords"));
                for i in 1..=workers {
                    let join_handle = crate::password_worker::password_check(
                        workers,
                        i,
                        file_path.clone(),
                        aes_info.clone(),
                        strategy.clone(),
                        send_found_password.clone(),
                        stop_workers_signal.clone(),
                        progress_bar.clone(),
                    );
                    worker_handles.push(join_handle);
                }
                // drop reference in `main` so that it disappears completely with workers for a clean shutdown
                drop(send_found_password);

                match receive_found_password.recv() {
                    Ok(password_found) => {
                        // stop generating values first to avoid deadlock on channel
                        stop_gen_signal.store(true, Ordering::Relaxed);
                        // stop workers
                        stop_workers_signal.store(true, Ordering::Relaxed);
                        for h in worker_handles {
                            h.join().unwrap();
                        }
                        // progress_bar.finish_and_clear();
                        Some(password_found)
                    }
                    Err(_) => {
                        // progress_bar.finish_and_clear();
                        None
                    }
                }
            }
            Some(archive) if archive.mime_type() == "application/x-7z-compressed" => {
                return pbi
                    .find_map_any(|password| {
                        sevenz_password_checker(&password, file_path.display().to_string())
                    })
                    .map(|f| f.to_string());
            }
            Some(archive) if archive.mime_type() == "application/pdf" => {
                return pbi
                    .find_map_any(|password| pdf_password_checker(&password, &zip_file))
                    .map(|f| f.to_string());
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::PathBuf};

    use crate::password_reader::password_reader_count;

    #[test]
    fn read_dic_test() {
        let path = PathBuf::from("xato-net-10-million-passwords.txt");
        let start = std::time::Instant::now();
        let mut content = String::new();
        File::open(path.clone())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(5189454, content.lines().count());
        let first = start.elapsed();

        let count = password_reader_count(path).unwrap();
        assert_eq!(5189454, count);
        let second = start.elapsed();

        println!("first: {:?},second: {:?}", first, second - first);
    }
}
