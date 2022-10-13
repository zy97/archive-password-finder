use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender};
use indicatif::ProgressBar;
use zip::ZipArchive;

pub fn password_checker(
    index: usize,
    file_path: &Path,
    receive_password: Receiver<String>,
    stop_signal: Arc<AtomicBool>,
    send_password_found: Sender<String>,
    progress_bar: ProgressBar,
) -> JoinHandle<()> {
    let file = File::open(file_path).expect("File should exist");
    thread::Builder::new()
        .name(format!("worker-{}", index))
        .spawn(move || {
            let mut archive = ZipArchive::new(file).expect("Archive validated before-hand");

            while !stop_signal.load(Ordering::Relaxed) {
                match receive_password.recv() {
                    Ok(password) => {
                        let res = archive.by_index_decrypt(0, password.as_bytes());
                        match res {
                            Ok(Ok(mut zip)) => {
                                let mut buffer = Vec::with_capacity(zip.size() as usize);
                                match zip.read_to_end(&mut buffer) {
                                    Ok(_) => {
                                        send_password_found
                                            .send(password)
                                            .expect("Send found password should not fail");
                                    }
                                    Err(_) => {}
                                }
                            }
                            Ok(Err(_)) => (),
                            Err(e) => panic!("Unexpected error: {}", e),
                        }
                        progress_bar.inc(1);
                    }
                    Err(_) => break,
                }
            }
        })
        .unwrap()
}
