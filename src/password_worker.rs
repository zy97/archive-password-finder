use crossbeam_channel::Sender;

use indicatif::ProgressBar;
use infer::Type;

use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
};

use crate::{
    finder_errors::FinderError, password_finder::Strategy, password_gen::PasswordGenWorker,
    password_reader::PasswordReader, zip::zip_utils::validate_zip, Passwords,
};

pub fn password_check(
    worker_count: usize,
    file_path: PathBuf,
    strategy: Strategy,
    send_password_found: Sender<String>,
    stop_workers_signal: Arc<AtomicBool>,
    progress_bar: ProgressBar,
    file_type: Option<Type>,
) -> Result<Vec<JoinHandle<()>>, FinderError> {
    let mut worker_handles = Vec::with_capacity(worker_count);

    for i in 1..=worker_count {
        let strategy = strategy.clone();
        let file_path = file_path.clone();
        // let aes_info = aes_info.clone();
        let send_password_found = send_password_found.clone();
        let stop_workers_signal = stop_workers_signal.clone();
        let progress_bar = progress_bar.clone();
        let join_handle = thread::Builder::new()
            .name(format!("worker-{}", i))
            .spawn(move || {
                let passwords: Passwords = match &strategy {
                    Strategy::GenPasswords {
                        charsets,
                        min_password_len,
                        max_password_len,
                    } => {
                        let c = charsets.clone();
                        let password_gen_worker =
                            PasswordGenWorker::new(c, *min_password_len, *max_password_len);

                        Box::new(password_gen_worker)
                    }
                    Strategy::PasswordFile(password_file_path) => {
                        let password_reader = PasswordReader::new(password_file_path.clone());
                        Box::new(password_reader)
                    }
                };
                match file_type {
                    #[cfg(feature = "rar")]
                    Some(file) if file.mime_type() == "application/vnd.rar" => {
                        crate::rar::password_check(
                            worker_count,
                            i,
                            file_path,
                            passwords,
                            send_password_found,
                            stop_workers_signal,
                            progress_bar,
                        )
                    }
                    Some(file) if file.mime_type() == "application/zip" => {
                        let aes_info = validate_zip(&file_path, &progress_bar).unwrap();
                        crate::zip::password_check(
                            worker_count,
                            i,
                            file_path,
                            aes_info,
                            passwords,
                            send_password_found,
                            stop_workers_signal,
                            progress_bar,
                        )
                    }
                    #[cfg(feature = "7z")]
                    Some(file) if file.mime_type() == "application/x-7z-compressed" => {
                        crate::seven_z::password_check(
                            worker_count,
                            i,
                            file_path,
                            passwords,
                            send_password_found,
                            stop_workers_signal,
                            progress_bar,
                        )
                    }
                    #[cfg(feature = "pdf")]
                    Some(file) if file.mime_type() == "application/pdf" => {
                        crate::pdf::password_check(
                            worker_count,
                            i,
                            file_path,
                            passwords,
                            send_password_found,
                            stop_workers_signal,
                            progress_bar,
                        )
                    }
                    _ => {
                        println!("7777777777");
                        progress_bar.abandon_with_message(format!(
                            " {} is not supported",
                            file_path.display()
                        ));
                    }
                }
            })
            .unwrap();
        worker_handles.push(join_handle);
    }
    Ok(worker_handles)
}
#[cfg(test)]
mod test {
    use std::{process::Command, time::Instant};

    #[test]
    fn test() {
        let now = Instant::now();
        let zip = "test1.7z";
        let mut command = Command::new(r".\7z.exe");
        command.arg("t");
        command.arg(zip);
        // command.arg(format!("-p{}", "123456"));
        let output = command.output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        println!("stdout: {}", stdout);
        println!("code : {}", output.status.code().unwrap());
        println!("time: {:?}", now.elapsed());
    }
}
