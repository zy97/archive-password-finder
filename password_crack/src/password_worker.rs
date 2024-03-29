use crossbeam_channel::Sender;

use infer::Type;

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crate::{
    errors::Errors, filter_for_worker_index, password_finder::Strategy,
    password_gen::PasswordGenerator, password_reader::PasswordReader, Passwords,
};

pub fn password_check(
    worker_count: usize,
    file_path: &Path,
    strategy: Strategy,
    send_password_found: Sender<String>,
    stop_workers_signal: Arc<AtomicBool>,
    file_type: Option<Type>,
    tested_count: Arc<AtomicU64>,
) -> Result<Vec<JoinHandle<()>>, Errors> {
    let mut worker_handles = Vec::with_capacity(worker_count);

    for i in 1..=worker_count {
        let file_path = file_path.clone().to_path_buf();
        let send_password_found = send_password_found.clone();
        let stop_workers_signal = stop_workers_signal.clone();
        // let send_progress_info = send_progress_info.clone();
        let mut passwords: Passwords = match &strategy {
            Strategy::GenPasswords {
                charsets,
                min_password_len,
                max_password_len,
            } => {
                let c = charsets.clone();
                let password_gen_worker =
                    PasswordGenerator::new(c, *min_password_len, *max_password_len);

                Box::new(password_gen_worker)
            }
            Strategy::PasswordFile(password_file_path) => {
                let password_reader = PasswordReader::new(password_file_path);
                Box::new(password_reader)
            }
        };
        passwords = filter_for_worker_index(passwords, worker_count, i);

        let tested_count = Arc::clone(&tested_count);
        let join_handle = thread::Builder::new()
            .name(format!("worker-{}", i))
            .spawn(move || {
                match file_type {
                    #[cfg(feature = "rar")]
                    Some(file) if file.mime_type() == "application/vnd.rar" => {
                        crate::rar::password_check(
                            worker_count,
                            i,
                            &file_path,
                            passwords,
                            send_password_found,
                            stop_workers_signal,
                            tested_count,
                        )
                    }
                    Some(file) if file.mime_type() == "application/zip" => {
                        crate::zip::password_check(
                            worker_count,
                            i,
                            &file_path,
                            passwords,
                            send_password_found,
                            stop_workers_signal,
                            tested_count,
                        )
                    }
                    #[cfg(feature = "7z")]
                    Some(file) if file.mime_type() == "application/x-7z-compressed" => {
                        crate::seven_z::password_check(
                            worker_count,
                            i,
                            &file_path,
                            passwords,
                            send_password_found,
                            stop_workers_signal,
                            tested_count,
                        )
                    }
                    #[cfg(feature = "pdf")]
                    Some(file) if file.mime_type() == "application/pdf" => {
                        crate::pdf::password_check(
                            worker_count,
                            i,
                            &file_path,
                            passwords,
                            send_password_found,
                            stop_workers_signal,
                            tested_count,
                        )
                    }
                    _ => {
                        // progress_bar.abandon_with_message(format!(
                        //     " {} is not supported",
                        //     file_path.display()
                        // ));
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
