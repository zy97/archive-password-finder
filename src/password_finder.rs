use crate::finder_errors::FinderError;
use crate::password_finder::Strategy::{GenPasswords, PasswordFile};
use crate::password_gen::PasswordGenWorker;
use crate::password_reader::PasswordReader;

use crate::PasswordFinder;
use indicatif::{ProgressBar, ProgressStyle};

use std::collections::HashSet;
use std::fs::{self};
use std::io::Cursor;
use std::path::{Path, PathBuf};

use zip::result::ZipError::UnsupportedArchive;
pub enum Strategy {
    PasswordFile(PathBuf),
    GenPasswords {
        charset_choice: Vec<String>,
        min_password_len: usize,
        max_password_len: usize,
        custom_chars: Vec<char>,
    },
}

pub fn password_finder(zip_path: &str, strategy: Strategy) -> Result<Option<String>, FinderError> {
    let zip_path = Path::new(zip_path);
    let zip_file = fs::read(zip_path)
        .expect(format!("Failed reading the ZIP file: {}", zip_path.display()).as_str());
    // validate_zip(&zip_file)?;

    let password_finder: Box<dyn PasswordFinder> = match strategy {
        GenPasswords {
            charset_choice,
            min_password_len,
            max_password_len,
            custom_chars,
        } => {
            let charset_letters = vec![
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            ];
            let charset_uppercase_letters = vec![
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
                'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            ];
            let charset_digits = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
            let charset_punctuations = vec![
                ' ', '-', '=', '!', '@', '#', '$', '%', '^', '&', '*', '_', '+', '<', '>', '/',
                '?', '.', ';', ':', '{', '}',
            ];
            let mut charset = charset_choice
                .into_iter()
                .map(|c| {
                    if c == "number" {
                        charset_digits.clone()
                    } else if c == "upper" {
                        charset_uppercase_letters.clone()
                    } else if c == "lower" {
                        charset_letters.clone()
                    } else if c == "special" {
                        charset_punctuations.clone()
                    } else {
                        panic!("Invalid charset choice")
                    }
                })
                .flatten()
                .collect::<Vec<char>>();
            for c in custom_chars {
                if !charset.contains(&c) {
                    charset.push(c);
                }
            }
            // let mut total_password_count = 0;
            // let charset_len = charset.len();
            // for i in min_password_len..=max_password_len {
            //     total_password_count += charset_len.pow(i as u32);
            // }
            let password_gen_worker =
                PasswordGenWorker::new(charset, min_password_len, max_password_len);
            Box::new(password_gen_worker)
        }
        PasswordFile(password_file_path) => {
            let password_reader = PasswordReader::new(password_file_path);
            Box::new(password_reader)
        }
    };

    let password = password_finder.find_password(zip_path.to_path_buf());
    match password {
        Some(password) => {
            println!("Found password: {}", password);
        }
        None => {
            println!("Password not found");
        }
    }

    Ok(None)
}
pub fn create_progress_bar(len: u64) -> ProgressBar {
    //设置进度条 进度条的样式也会影响性能，进度条越简单性能也好，影响比较小
    let progress_bar = ProgressBar::new(len).with_finish(indicatif::ProgressFinish::AndLeave);
    let progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {spinner} {pos:7}/{len:7} throughput:{per_sec} (eta:{eta})")
        .expect("Failed to create progress style");
    progress_bar.set_style(progress_style);
    //每两秒刷新终端，避免闪烁
    // let draw_target = ProgressDrawTarget::stdout_with_hz(2);
    // progress_bar.set_draw_target(draw_target);
    progress_bar
}

fn validate_zip(zip_file: &[u8]) -> Result<(), FinderError> {
    let cursor = Cursor::new(zip_file);
    let mut archive = zip::ZipArchive::new(cursor)?;
    let zip_result = archive.by_index(0);
    match zip_result {
        Ok(_) => Err(FinderError::invalid_zip_error(
            "the archive is not encrypted".to_string(),
        )),
        Err(UnsupportedArchive(msg)) if msg == "Password required to decrypt file" => Ok(()),
        Err(e) => Err(FinderError::invalid_zip_error(format!(
            "Unexcepted error: {:?}",
            e
        ))),
    }
}
