use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender};
use zip::ZipArchive;

fn main() {
    let zip_path = env::args().nth(1).expect("No zip file provided");
    let dictionary_path = "xato-net-10-million-passwords.txt";
    let workers = 3;
    match password_finder(&zip_path, dictionary_path, workers) {
        Some(password) => println!("Password found: {}", password),
        None => println!("Password not found"),
    }
}

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

pub fn password_checker(
    index: usize,
    file_path: &Path,
    receive_password: Receiver<String>,
    stop_signal: Arc<AtomicBool>,
    send_password_found: Sender<String>,
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
                    }
                    Err(_) => break,
                }
            }
        })
        .unwrap()
}

pub fn password_finder(zip_path: &str, password_list_path: &str, workers: usize) -> Option<String> {
    let zip_file_path = Path::new(zip_path);
    let password_list_file_path = Path::new(password_list_path);

    let (send_password, receive_password) = crossbeam_channel::bounded(workers * 10_000);

    let (send_password_found, receive_password_found) = crossbeam_channel::bounded(1);

    let stop_workers_signal = Arc::new(AtomicBool::new(false));
    let stop_gen_signal = Arc::new(AtomicBool::new(false));

    let password_gen_handle = start_password_reader(
        password_list_file_path.to_path_buf(),
        send_password,
        stop_gen_signal.clone(),
    );

    let mut worker_handles = Vec::with_capacity(workers);
    for i in 0..=workers {
        let join_handle = password_checker(
            i,
            zip_file_path,
            receive_password.clone(),
            stop_workers_signal.clone(),
            send_password_found.clone(),
        );
        worker_handles.push(join_handle);
    }

    drop(send_password_found);

    match receive_password_found.recv() {
        Ok(password_found) => {
            stop_gen_signal.store(true, Ordering::Relaxed);
            password_gen_handle.join().unwrap();
            stop_workers_signal.store(true, Ordering::Relaxed);
            for handle in worker_handles {
                handle.join().unwrap();
            }
            Some(password_found)
        }
        Err(_) => None,
    }
}
