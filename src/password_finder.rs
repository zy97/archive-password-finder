use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::{password_reader::start_password_reader, password_worker::password_checker};

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
