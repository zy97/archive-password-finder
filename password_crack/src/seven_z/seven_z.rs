use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
};

use crossbeam_channel::Sender;

use crate::Passwords;
pub fn password_check(
    worker_count: usize,
    worker_index: usize,
    senven_z_file: &Path,
    passwords: Passwords,
    send_password_found: Sender<String>,
    stop_workers_signal: Arc<AtomicBool>,
    send_progress_info: Sender<u64>,
) {
    let batching_dalta = worker_count * 10;
    let first_worker = worker_index == 1;
    let progress_bar_delta: u64 = (batching_dalta * worker_count) as u64;
    let mut processed_delta = 0;
    for password in passwords {
        let res = sevenz_rust::decompress_file_with_password(
            &senven_z_file,
            "test/",
            password.as_str().into(),
        );
        match res {
            Ok(()) => send_password_found
                .send(password)
                .expect("Send found password should not fail"),
            _ => {}
        }

        processed_delta += 1;
        //do not check internal flags too often
        if processed_delta == batching_dalta {
            if first_worker {
                send_progress_info
                    .send(progress_bar_delta)
                    .expect("Send progress should not fail");
            }
            if stop_workers_signal.load(Ordering::Relaxed) {
                break;
            }
            processed_delta = 0;
        }
    }
}
