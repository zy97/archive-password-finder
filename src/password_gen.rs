use crate::{
    password_finder::Strategy,
    password_worker::{pdf_password_checker, rar_password_checker, sevenz_password_checker},
    progress_bar::create_progress_bar,
    PasswordFinder,
};
use indicatif::ParallelProgressIterator;
use permutator::{copy::get_cartesian_for, get_cartesian_size};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{fs, path::PathBuf};

pub fn password_generator_count(charset: &Vec<char>, min_size: usize, max_size: usize) -> usize {
    // compute the number of passwords to generate
    let charset_len = charset.len();
    let mut total_password_count = 0;
    for i in min_size..=max_size {
        total_password_count += charset_len.pow(i as u32)
    }
    total_password_count
}
pub struct PasswordGenWorker {
    charset: Vec<char>,
    min_password_len: usize,
    max_password_len: usize,
    pub total_password_count: usize,
    current_password_index: usize,
}
impl Iterator for PasswordGenWorker {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let current_password_index = self.current_password_index;
        let total_password_count = self.total_password_count;
        let mut passwrod_lenth = 0;
        let mut total = 0;
        if current_password_index < total_password_count {
            for i in self.min_password_len..=self.max_password_len {
                total += get_cartesian_size(self.charset.len(), i);
                if current_password_index < total {
                    passwrod_lenth = i;
                    break;
                }
            }
            let current_deep_count = get_cartesian_size(self.charset.len(), passwrod_lenth);
            let current_deep_index = current_password_index - (total - current_deep_count);
            let res = get_cartesian_for(&self.charset, passwrod_lenth, current_deep_index);
            self.current_password_index += 1;
            Some(res.unwrap().iter().collect())
        } else {
            None
        }
    }
}
impl PasswordGenWorker {
    pub fn new(charset: Vec<char>, min_password_len: usize, max_password_len: usize) -> Self {
        let mut total_count = 0;
        let char_count = charset.len();
        for i in min_password_len..=max_password_len {
            total_count += get_cartesian_size(char_count, i)
        }
        PasswordGenWorker {
            charset,
            min_password_len,
            max_password_len,
            total_password_count: total_count,
            current_password_index: 0,
        }
    }
    pub fn total_count(&self) -> usize {
        self.total_password_count
    }
    pub fn get_nth(&self, index: usize) -> Option<String> {
        let current_password_index = index;
        let total_password_count = self.total_password_count;
        let mut passwrod_lenth = 0;
        let mut total = 0;
        if current_password_index < total_password_count {
            for i in self.min_password_len..=self.max_password_len {
                total += get_cartesian_size(self.charset.len(), i);
                if current_password_index < total {
                    passwrod_lenth = i;
                    break;
                }
            }
            let current_deep_count = get_cartesian_size(self.charset.len(), passwrod_lenth);
            let current_deep_index = current_password_index - (total - current_deep_count);
            let res = get_cartesian_for(&self.charset, passwrod_lenth, current_deep_index);
            Some(res.unwrap().iter().collect())
        } else {
            None
        }
    }
}
impl PasswordFinder for PasswordGenWorker {
    fn find_password(&self, compressed_file: PathBuf, strategy: Strategy) -> Option<String> {
        let kind = infer::get_from_path(&compressed_file).unwrap();
        let zip_file = fs::read(&compressed_file)
            .expect(format!("Failed reading the ZIP file: {}", compressed_file.display()).as_str());
        let progress_bar = create_progress_bar(self.total_password_count as u64);
        let total_password_count = self.total_password_count;
        let min_password_len = self.min_password_len;
        let max_password_len = self.max_password_len;
        let charset = self.charset.clone();
        (0..total_password_count)
            .into_par_iter()
            .progress_with(progress_bar)
            .find_map_any(|index| {
                let mut password_len = min_password_len;
                let mut total = 0;
                for _ in min_password_len..=max_password_len {
                    total += get_cartesian_size(charset.len(), password_len);
                    if index < total {
                        break;
                    } else {
                        password_len += 1;
                    }
                }
                let current_deep_count = get_cartesian_size(charset.len(), password_len);
                let current_deep_index = index - (total - current_deep_count);
                let password: String =
                    get_cartesian_for(&charset, password_len, current_deep_index)
                        .unwrap()
                        .iter()
                        .collect();

                match kind {
                    Some(archive) if archive.mime_type() == "application/vnd.rar" => {
                        rar_password_checker(&password, compressed_file.display().to_string())
                            .map(|f| f.to_string())
                    }
                    Some(archive) if archive.mime_type() == "application/x-7z-compressed" => {
                        sevenz_password_checker(&password, compressed_file.display().to_string())
                            .map(|f| f.to_string())
                    }
                    // Some(archive) if archive.mime_type() == "application/zip" => {
                    //     password_checker(&password, &zip_file).map(|f| f.to_string())
                    // }
                    Some(archive) if archive.mime_type() == "application/pdf" => {
                        pdf_password_checker(&password, &zip_file).map(|f| f.to_string())
                    }
                    _ => None,
                }
            })
    }
}
