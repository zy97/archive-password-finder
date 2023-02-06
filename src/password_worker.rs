use crossbeam_channel::Sender;
use hmac::Hmac;
use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::{prelude::ParallelIterator, str::ParallelString};
use sha1::Sha1;
use std::{
    fs::{self, File},
    io::{BufReader, Cursor, Read},
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};
use zip::ZipArchive;

use crate::{
    password_finder::Strategy,
    password_gen::{password_generator_count, password_generator_iter, PasswordGenWorker},
    password_reader::{password_dictionary_reader_iter, password_reader_count, PasswordReader},
    progress_bar::create_progress_bar,
    zip::{
        zip_a::{filter_for_worker_index, Passwords, ZipReader},
        zip_utils::{validate_zip, AesInfo},
    },
};

// 使用fs::read 读取文件，并用Curosr包裹buffer，并把cursor传给ZipArchive，速度一下冲26w/s到了380w/s
// zip_file 使用&[u8]作为参数，性能没有什么变化
pub fn password_checker21<'a>(password: &'a str, zip_file: &[u8]) -> Option<&'a str> {
    let cursor = Cursor::new(zip_file);

    let mut archive = ZipArchive::new(cursor).expect("Archive validated before-hand");
    // 从rust文档了解到，该函数有时会接受错误密码，这是因为zip规范只允许我们检查密码是否有1/256的概率是正确的
    // 有很多密码也能通过该方法的校验，但是实际上是错误的
    // 这是ZipCrypto算法的一个缺陷，因为它是相当原始的密码学方法
    let res = archive.by_index_decrypt(0, password.as_bytes());
    match res {
        Ok(Ok(mut zip)) => {
            // 通过读取zip文件来验证密码，以确保它不仅仅是一个hash碰撞
            // 不用关心重用缓冲区，因为冲突是极少的
            let mut buffer = Vec::with_capacity(zip.size() as usize);
            match zip.read_to_end(&mut buffer) {
                Ok(_) => Some(password),
                Err(_) => None,
            }
        }
        _ => None,
    }
}
pub fn rar_password_checker<'a>(password: &'a str, rar_file_path: String) -> Option<&'a str> {
    let archive = unrar::Archive::with_password(rar_file_path, password.to_string());
    let mut open_archive = archive.test().unwrap();
    if open_archive.process().is_ok() {
        Some(password)
    } else {
        None
    }
}
pub fn sevenz_password_checker<'a>(password: &'a str, rar_file_path: String) -> Option<&'a str> {
    let mut command = Command::new(r".\7z.exe");
    command.arg("t");
    command.arg(rar_file_path);
    command.arg(format!("-p{}", password));
    let output = command.output().unwrap();
    match output.status.code() {
        Some(0) => Some(password),
        _ => None,
    }
}
pub fn pdf_password_checker<'a>(password: &'a str, pdf_file: &[u8]) -> Option<&'a str> {
    let res = pdf::file::File::from_data_password(pdf_file, password.as_bytes());
    match res {
        Ok(_) => Some(password),
        _ => None,
    }
}

pub fn password_check(
    worker_count: usize,
    worker_index: usize,
    zip_file: PathBuf,
    aes_info: Option<AesInfo>,
    stragegy: Strategy,
    send_password_found: Sender<String>,
    stop_workers_signal: Arc<AtomicBool>,
    progress_bar: ProgressBar,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name(format!("worker-{}", worker_index))
        .spawn(move || {
            let batching_dalta = worker_count * 500;
            let first_worker = worker_index == 1;
            let progress_bar_delta: u64 = (batching_dalta * worker_count) as u64;
            let (total_password_count, passwords): (usize, Passwords) = match stragegy {
                Strategy::GenPasswords {
                    charsets,
                    min_password_len,
                    max_password_len,
                } => {
                    let c = charsets.clone();
                    let password_gen_worker =
                        PasswordGenWorker::new(c, min_password_len, max_password_len);
                    (
                        password_generator_count(&charsets, min_password_len, max_password_len),
                        Box::new(password_gen_worker),
                    )
                }
                Strategy::PasswordFile(password_file_path) => {
                    let password_reader = PasswordReader::new(password_file_path.clone());
                    (
                        password_reader_count(&password_file_path).unwrap(),
                        Box::new(password_reader),
                    )
                }
            };
            // password_iter = filter_for_worker_index(password_iter, worker_count, worker_index);

            // // AES info bindings
            // let mut derived_key_len = 0;
            // let mut derived_key: Vec<u8> = Vec::new();
            // let mut salt: Vec<u8> = Vec::new();
            // let mut key: Vec<u8> = Vec::new();

            // // setup file reader depending on the encryption method
            // let reader: Box<dyn ZipReader> = if let Some(aes_info) = aes_info {
            //     salt = aes_info.salt;
            //     key = aes_info.key;
            //     derived_key_len = aes_info.derived_key_length;
            //     derived_key = vec![0; derived_key_len];
            //     let file = File::open(zip_file).expect("File should exist");
            //     // in case of AES we do not need to access the archive often, a buffer reader is enough
            //     Box::new(BufReader::new(file))
            // } else {
            //     let zip_file = fs::read(zip_file).expect("File should exist");
            //     // in case of ZipCrypto, we load the file in memory as it will be access on each password
            //     Box::new(Cursor::new(zip_file))
            // };

            // // zip archive
            // let mut archive = ZipArchive::new(reader).expect("Archive validated before-hand");
            // let mut extraction_buffer = Vec::new();

            // let mut processed_delta = 0;
            // for password in password_iter {
            //     let password_bytes = password.as_bytes();
            //     let mut potential_match = true;

            //     // process AES KEY
            //     if derived_key_len != 0 {
            //         // use PBKDF2 with HMAC-Sha1 to derive the key
            //         pbkdf2::pbkdf2::<Hmac<Sha1>>(password_bytes, &salt, 1000, &mut derived_key);
            //         let pwd_verify = &derived_key[derived_key_len - 2..];
            //         // the last 2 bytes should equal the password verification value
            //         potential_match = key == pwd_verify;
            //     }

            //     // ZipCrypto falls back directly here and will recompute its key for each password
            //     if potential_match {
            //         // From the Rust doc:
            //         // This function sometimes accepts wrong password. This is because the ZIP spec only allows us to check for a 1/256 chance that the password is correct.
            //         // There are many passwords out there that will also pass the validity checks we are able to perform.
            //         // This is a weakness of the ZipCrypto algorithm, due to its fairly primitive approach to cryptography.
            //         let res = archive.by_index_decrypt(0, password_bytes);
            //         match res {
            //             Ok(Err(_)) => (), // invalid password
            //             Ok(Ok(mut zip)) => {
            //                 // Validate password by reading the zip file to make sure it is not merely a hash collision.
            //                 extraction_buffer.reserve(zip.size() as usize);
            //                 match zip.read_to_end(&mut extraction_buffer) {
            //                     Err(_) => (), // password collision - continue
            //                     Ok(_) => {
            //                         // Send password and continue processing while waiting for signal
            //                         send_password_found
            //                             .send(password)
            //                             .expect("Send found password should not fail");
            //                     }
            //                 }
            //                 extraction_buffer.clear();
            //             }
            //             Err(e) => panic!("Unexpected error {e:?}"),
            //         }
            //     }
            //     processed_delta += 1;
            //     //do not check internal flags too often
            //     if processed_delta == batching_dalta {
            //         if first_worker {
            //             progress_bar.inc(progress_bar_delta);
            //         }
            //         if stop_workers_signal.load(Ordering::Relaxed) {
            //             break;
            //         }
            //         processed_delta = 0;
            //     }
            // }
            crate::zip::zip_a::password_check(
                worker_count,
                worker_index,
                zip_file,
                aes_info,
                passwords,
                send_password_found,
                stop_workers_signal,
                progress_bar,
            )
        })
        .unwrap()
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
