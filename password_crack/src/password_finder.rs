use crossbeam_channel::{Receiver, Sender};

use crate::errors::Errors;
use crate::password_finder::Strategy::{GenPasswords, PasswordFile};
use crate::password_gen::password_generator_count;
use crate::password_reader::password_reader_count;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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

pub fn password_finder(
    file_path: &str,
    workers: usize,
    strategy: Strategy,
    t: Arc<AtomicU64>,
) -> Result<Option<String>, Errors>
where
{
    let file_path = Path::new(file_path);
    let file_type = infer::get_from_path(&file_path)?;
    //停止与线程关闭信号量
    let stop_workers_signal = Arc::new(AtomicBool::new(false));
    let stop_gen_signal = Arc::new(AtomicBool::new(false));
    let (send_found_password, receive_found_password): (Sender<String>, Receiver<String>) =
        crossbeam_channel::bounded(1);
    let worker_handles = crate::password_worker::password_check(
        workers,
        file_path,
        strategy,
        send_found_password.clone(),
        stop_workers_signal.clone(),
        file_type,
        t,
    )?;
    // drop reference in `main` so that it disappears completely with workers for a clean shutdown
    drop(send_found_password);

    let res = match receive_found_password.recv() {
        Ok(password_found) => {
            // stop generating values first to avoid deadlock on channel
            stop_gen_signal.store(true, Ordering::Relaxed);
            // stop workers
            stop_workers_signal.store(true, Ordering::Relaxed);
            for h in worker_handles {
                h.join().unwrap();
            }
            Some(password_found)
        }
        Err(e) => {
            println!("66666{}", e);
            None
        }
    };
    // drop(send_progress_info);
    Ok(res)
}
pub fn get_password_count(strategy: &Strategy) -> Result<usize, Errors> {
    let total_password_count = match &strategy {
        GenPasswords {
            charsets,
            min_password_len,
            max_password_len,
        } => password_generator_count(charsets, *min_password_len, *max_password_len),
        PasswordFile(password_file_path) => password_reader_count(password_file_path),
    };
    total_password_count
}
