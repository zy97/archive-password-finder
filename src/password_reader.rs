use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crossbeam_channel::Sender;

pub fn start_password_reader(
    file_path: PathBuf,
    send_password: Sender<String>,
    stop_signal: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("password_reader".to_string())
        .spawn(move || {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if stop_signal.load(Ordering::Relaxed) {
                    break;
                } else {
                    match send_password.send(line.unwrap()) {
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            }
        })
        .unwrap()
}
