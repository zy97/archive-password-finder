use std::{
    cmp::max,
    env,
    fs::File,
    io::{stdin, BufRead, BufReader, Read},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, available_parallelism, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use zip::ZipArchive;

fn main() {
    let zip_path = env::args().nth(1).expect("No zip file provided");
    let dictionary_path = "xato-net-10-million-passwords.txt";
    let num_cores = available_parallelism().unwrap().get();
    println!("Using {} cores", num_cores);
    let workers = max(1, num_cores - 1);
    match password_finder(&zip_path, dictionary_path, workers) {
        Some(password) => println!("Password found: {}", password),
        None => println!("Password not found"),
    }
    println!("按回车键退出...");
    stdin().read(&mut [0]).unwrap();
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

pub fn password_finder(zip_path: &str, password_list_path: &str, workers: usize) -> Option<String> {
    let zip_file_path = Path::new(zip_path);
    let password_list_file_path = Path::new(password_list_path);

    //设置进度条
    let progress_bar = ProgressBar::new(0);
    let progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {wide_bar} {pos}/{len} throughput:{per_sec} (eta:{eta})")
        .expect("Failed to create progress style");
    progress_bar.set_style(progress_style);
    //每两秒刷新终端，避免闪烁
    let draw_target = ProgressDrawTarget::stdout_with_hz(2);
    progress_bar.set_draw_target(draw_target);
    //设置进度条长度为字典大小
    let file = BufReader::new(File::open(password_list_file_path).expect("Unable to open file"));
    let total_password_count = file.lines().count();
    progress_bar.set_length(total_password_count as u64);

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
            progress_bar.clone(),
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
            progress_bar.finish_and_clear();
            Some(password_found)
        }
        Err(_) => None,
    }
}
