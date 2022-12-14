use std::{
    fs::{self},
    path::PathBuf,
};

use indicatif::ParallelProgressIterator;
use rayon::{prelude::ParallelIterator, str::ParallelString};

use crate::{
    password_finder::create_progress_bar,
    password_worker::{password_checker, rar_password_checker, sevenz_password_checker},
    PasswordFinder,
};

pub struct PasswordReader {
    pub total_password_count: usize,
    passwords_lines: String,
}
impl PasswordReader {
    pub fn new(file_path: PathBuf) -> Self {
        let dic = fs::read_to_string(&file_path).unwrap_or_else(|_| {
            panic!(
                "Failed reading the dictionary file: {}",
                file_path.display()
            )
        });
        PasswordReader {
            total_password_count: dic.lines().count(),
            passwords_lines: dic,
        }
    }
}

impl PasswordFinder for PasswordReader {
    fn find_password(&self, compressed_file: PathBuf) -> Option<String> {
        let kind = infer::get_from_path(&compressed_file).unwrap();
        let zip_file = fs::read(&compressed_file)
            .expect(format!("Failed reading the ZIP file: {}", compressed_file.display()).as_str());
        let progress_bar = create_progress_bar(self.total_password_count as u64);
        let pbi = self.passwords_lines.par_lines().progress_with(progress_bar);
        match kind {
            Some(archive) if archive.mime_type() == "application/vnd.rar" => {
                return pbi
                    .find_map_any(|password| {
                        rar_password_checker(password, compressed_file.display().to_string())
                    })
                    .map(|f| f.to_string());
            }
            Some(archive) if archive.mime_type() == "application/zip" => {
                return pbi
                    .find_map_any(|password| password_checker(password, &zip_file))
                    .map(|f| f.to_string());
            }
            Some(archive) if archive.mime_type() == "application/x-7z-compressed" => {
                return pbi
                    .find_map_any(|password| {
                        sevenz_password_checker(password, compressed_file.display().to_string())
                    })
                    .map(|f| f.to_string());
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use indicatif::HumanDuration;
    #[test]
    fn read_dic_test() {
        let start = std::time::Instant::now();
        let dic = super::PasswordReader::new(PathBuf::from("xato-net-10-million-passwords.txt"));
        assert_eq!(dic.total_password_count, 5189454);
        let stop = start.elapsed();
        println!("Duration: {}", HumanDuration(stop));
    }
}
