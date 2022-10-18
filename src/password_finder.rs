use crate::password_gen::start_password_generation;
use crate::{
    finder_errors::FinderError, password_reader::start_password_reader,
    password_worker::password_checker,
};
use crate::{GenPasswords, PasswordFile};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::prelude::{IntoParallelRefIterator, IndexedParallelIterator};
use std::fs;
use std::io::{BufRead, BufReader, Cursor};
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use zip::result::ZipError::UnsupportedArchive;

pub enum Strategy {
    PasswordFile(PathBuf),
    GenPasswords {
        charset_choice: Vec<String>,
        min_password_len: usize,
        max_password_len: usize,
    },
}

pub fn password_finder(
    zip_path: &str,
    workers: usize,
    strategy: Strategy,
) -> Result<Option<String>, FinderError> {
    let vvv = vec![1, 2, 3];
    let zip_path = Path::new(zip_path);
    let zip_file = fs::read(zip_path)
        .expect(format!("Failed reading the ZIP file: {}", zip_path.display()).as_str());
    validate_zip(&zip_file)?;

    //设置进度条
    let progress_bar = ProgressBar::new(0);
    let progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {wide_bar} {pos}/{len} throughput:{per_sec} (eta:{eta})")
        .expect("Failed to create progress style");
    progress_bar.set_style(progress_style);
    //每两秒刷新终端，避免闪烁
    let draw_target = ProgressDrawTarget::stdout_with_hz(2);
    progress_bar.set_draw_target(draw_target);

    let (send_password, receive_password) = crossbeam_channel::bounded(workers * 10_000);

    let (send_password_found, receive_password_found) = crossbeam_channel::bounded(1);

    let stop_workers_signal = Arc::new(AtomicBool::new(false));
    let stop_gen_signal = Arc::new(AtomicBool::new(false));

    let (total_password_count, password_gen_handle) = match strategy {
        GenPasswords {
            charset_choice,
            min_password_len,
            max_password_len,
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
            let charset = charset_choice
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

            // let charset = match charset_choice {
            //     CharsetChoice::Easy => vec![charset_letters, charset_uppercase_letters].concat(),
            //     CharsetChoice::Medium => {
            //         vec![charset_letters, charset_uppercase_letters, charset_digits].concat()
            //     }
            //     CharsetChoice::Hard => vec![
            //         charset_letters,
            //         charset_uppercase_letters,
            //         charset_digits,
            //         charset_punctuations,
            //     ]
            //     .concat(),
            // };

            let mut total_password_count = 0;
            let charset_len = charset.len();
            for i in min_password_len..=max_password_len {
                total_password_count += charset_len.pow(i as u32);
            }
            (
                total_password_count,
                start_password_generation(
                    charset,
                    min_password_len,
                    max_password_len,
                    send_password,
                    stop_gen_signal.clone(),
                    progress_bar.clone(),
                ),
            )
        }
        PasswordFile(password_file_path) => {
            let file =
                BufReader::new(File::open(&password_file_path).expect("Unable to open file"));
            let mut total_password_count = 0;
            for _ in file.lines() {
                total_password_count += 1;
            }
            progress_bar.println(format!(
                "Using passwords file reader {:?} with {} lines",
                password_file_path, total_password_count
            ));
            (
                total_password_count,
                start_password_reader(password_file_path, send_password, stop_gen_signal.clone()),
            )
        }
    };

    progress_bar.set_length(total_password_count as u64);

    let mut worker_handles = Vec::with_capacity(workers);
    progress_bar.println(format!("Using {} workers to test passwords", workers));
    for i in 1..=workers {
        let join_handle = password_checker(
            i,
            &zip_file,
            receive_password.clone(),
            stop_workers_signal.clone(),
            send_password_found.clone(),
            progress_bar.clone(),
        );
        worker_handles.push(join_handle);
    }

    drop(send_password_found);

    match receive_password_found.recv() {
        Ok(password_found) => {
            progress_bar.println(format!("Password found '{}'", password_found));
            stop_gen_signal.store(true, Ordering::Relaxed);
            password_gen_handle.join().unwrap();
            stop_workers_signal.store(true, Ordering::Relaxed);
            for handle in worker_handles {
                handle.join().unwrap();
            }
            progress_bar.finish_and_clear();
            Ok(Some(password_found))
        }
        Err(_) => {
            progress_bar.println("Password not found :(");
            progress_bar.finish_and_clear();
            Ok(None)
        }
    }
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
