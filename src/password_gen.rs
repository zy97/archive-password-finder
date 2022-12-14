use crate::{
    password_finder::create_progress_bar,
    password_worker::{password_checker, rar_password_checker},
    PasswordFinder,
};
use indicatif::ParallelProgressIterator;
use permutator::{copy::get_cartesian_for, get_cartesian_size};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{fs, path::PathBuf};

pub struct PasswordGenWorker {
    charset: Vec<char>,
    min_password_len: usize,
    max_password_len: usize,
    total_password_count: usize,
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
        for i in min_password_len..=max_password_len {
            total_count += get_cartesian_size(charset.len(), i)
        }
        PasswordGenWorker {
            charset,
            min_password_len,
            max_password_len,
            total_password_count: total_count,
            current_password_index: 0,
        }
    }
}
impl PasswordFinder for PasswordGenWorker {
    fn find_password(&self, compressed_file: PathBuf) -> Option<String> {
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
                    Some(archive) if archive.mime_type() == "application/zip" => {
                        password_checker(&password, &zip_file).map(|f| f.to_string())
                    }
                    _ => None,
                }
            })
    }
}
