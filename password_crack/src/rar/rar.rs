use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};

use crossbeam_channel::Sender;
use unrar::Archive;

use crate::Passwords;
pub fn password_check(
    worker_count: usize,
    worker_index: usize,
    rar_file: &Path,
    passwords: Passwords,
    send_password_found: Sender<String>,
    stop_workers_signal: Arc<AtomicBool>,
    tested_count: Arc<AtomicU64>,
) {
    let batching_dalta = worker_count * 500;
    let first_worker = worker_index == 1;
    let progress_bar_delta: u64 = (batching_dalta * worker_count) as u64;

    let mut processed_delta = 0;
    for password in passwords {
        let archive = Archive::with_password(rar_file.display().to_string(), password.to_string());
        let mut open_archive = archive.test().unwrap();
        if open_archive.process().is_ok() {
            send_password_found
                .send(password)
                .expect("Send found password should not fail");
        }

        processed_delta += 1;
        //do not check internal flags too often
        if processed_delta == batching_dalta {
            if first_worker {
                tested_count.fetch_add(progress_bar_delta, Ordering::SeqCst);
            }
            if stop_workers_signal.load(Ordering::Relaxed) {
                break;
            }
            processed_delta = 0;
        }
    }
}
