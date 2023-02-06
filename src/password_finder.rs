use crossbeam_channel::{Receiver, Sender};

use crate::finder_errors::FinderError;
use crate::password_finder::Strategy::{GenPasswords, PasswordFile};
use crate::password_gen::{password_generator_count, PasswordGenWorker};
use crate::password_reader::{password_reader_count, PasswordReader};
use crate::progress_bar::create_progress_bar;
use crate::zip::zip_a::Passwords;
use crate::zip::zip_utils::validate_zip;
use crate::PasswordFinder;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub enum Strategy {
    PasswordFile(PathBuf),
    GenPasswords {
        charsets: Vec<char>,
        min_password_len: usize,
        max_password_len: usize,
    },
}

pub fn password_finder(zip_path: &str, strategy: Strategy) -> Result<Option<String>, FinderError> {
    let zip_path = Path::new(zip_path);

    //停止与线程关闭信号量
    let stop_workers_signal = Arc::new(AtomicBool::new(false));
    let stop_gen_signal = Arc::new(AtomicBool::new(false));

    let total_password_count = match &strategy {
        GenPasswords {
            charsets,
            min_password_len,
            max_password_len,
        } => password_generator_count(charsets, *min_password_len, *max_password_len),
        PasswordFile(password_file_path) => password_reader_count(password_file_path)?,
    };
    // Fail early if the zip file is not valid

    let workers_count = 4;
    let (send_found_password, receive_found_password): (Sender<String>, Receiver<String>) =
        crossbeam_channel::bounded(1);
    let mut worker_handles = Vec::with_capacity(workers_count);
    // let password = password_finder.find_password(zip_path.to_path_buf(), strategy.clone());
    let progress_bar = create_progress_bar(total_password_count as u64);
    let aes_info = validate_zip(zip_path, &progress_bar)?;

    for i in 1..=workers_count {
        let join_handle = crate::password_worker::password_check(
            workers_count,
            i,
            zip_path.to_path_buf(),
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
            progress_bar.abandon();
            println!("Found password: {}", password_found);
        }
        Err(_) => {
            // progress_bar.finish_and_clear();
            progress_bar.finish();
            println!("Password not found");
        }
    }
    Ok(None)
}
