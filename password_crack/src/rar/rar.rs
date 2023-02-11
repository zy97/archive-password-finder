use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossbeam_channel::Sender;
use indicatif::ProgressBar;
use unrar::Archive;

use crate::{filter_for_worker_index, Passwords};
pub fn password_check(
    worker_count: usize,
    worker_index: usize,
    rar_file: PathBuf,
    passwords: Passwords,
    send_password_found: Sender<String>,
    stop_workers_signal: Arc<AtomicBool>,
    progress_bar: ProgressBar,
) {
    let batching_dalta = worker_count * 500;
    let first_worker = worker_index == 1;
    let progress_bar_delta: u64 = (batching_dalta * worker_count) as u64;
    let passwords = filter_for_worker_index(passwords, worker_count, worker_index);

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
                progress_bar.inc(progress_bar_delta);
            }
            if stop_workers_signal.load(Ordering::Relaxed) {
                break;
            }
            processed_delta = 0;
        }
    }
}
