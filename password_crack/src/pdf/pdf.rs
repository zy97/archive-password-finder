use std::{
    fs::{self},
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};

use crossbeam_channel::Sender;

use crate::Passwords;
pub fn password_check(
    worker_count: usize,
    worker_index: usize,
    pdf_file: &Path,
    passwords: Passwords,
    send_password_found: Sender<String>,
    stop_workers_signal: Arc<AtomicBool>,
    tested_count: Arc<AtomicU64>,
) {
    let batching_dalta = worker_count * 500;
    let first_worker = worker_index == 1;
    let progress_bar_delta: u64 = (batching_dalta * worker_count) as u64;
    let buffer = fs::read(pdf_file).unwrap();

    let mut processed_delta = 0;
    for password in passwords {
        let res = pdf::file::File::from_data_password(&buffer as &[u8], password.as_bytes());
        if res.is_ok() {
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
