use std::{
    fs::{self, File},
    io::{BufReader, Cursor, Read, Seek},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crossbeam_channel::Sender;
use hmac::Hmac;
use indicatif::ProgressBar;
use sha1::Sha1;
use zip::ZipArchive;

use crate::{password_finder::Strategy, password_reader::password_dictionary_reader_iter};

use super::zip_utils::AesInfo;
type Passwords = Box<dyn Iterator<Item = String>>;

pub fn filter_for_worker_index(
    passwords: Passwords,
    worker_count: usize,
    worker_index: usize,
) -> Passwords {
    if worker_count > 1 {
        Box::new(passwords.enumerate().filter_map(move |(index, password)| {
            if index % worker_count == worker_index {
                Some(password)
            } else {
                None
            }
        }))
    } else {
        passwords
    }
}

pub trait ZipReader: Read + Seek {}
impl ZipReader for Cursor<Vec<u8>> {}
impl ZipReader for BufReader<fs::File> {}
pub fn password_check1(
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
            let mut password_iter: Box<dyn Iterator<Item = String>> = match stragegy {
                Strategy::GenPasswords {
                    charsets,
                    min_password_len,
                    max_password_len,
                } => {
                    let pb = if first_worker {
                        progress_bar.clone()
                    } else {
                        ProgressBar::hidden()
                    };
                    todo!()
                }
                Strategy::PasswordFile(dictionary_path) => {
                    let iterator = password_dictionary_reader_iter(dictionary_path);
                    Box::new(iterator)
                }
            };
            password_iter = filter_for_worker_index(password_iter, worker_count, worker_index);

            // AES info bindings
            let mut derived_key_len = 0;
            let mut derived_key: Vec<u8> = Vec::new();
            let mut salt: Vec<u8> = Vec::new();
            let mut key: Vec<u8> = Vec::new();

            // setup file reader depending on the encryption method
            let reader: Box<dyn ZipReader> = if let Some(aes_info) = aes_info {
                salt = aes_info.salt;
                key = aes_info.key;
                derived_key_len = aes_info.derived_key_length;
                derived_key = vec![0; derived_key_len];
                let file = File::open(zip_file).expect("File should exist");
                // in case of AES we do not need to access the archive often, a buffer reader is enough
                Box::new(BufReader::new(file))
            } else {
                let zip_file = fs::read(zip_file).expect("File should exist");
                // in case of ZipCrypto, we load the file in memory as it will be access on each password
                Box::new(Cursor::new(zip_file))
            };

            // zip archive
            let mut archive = ZipArchive::new(reader).expect("Archive validated before-hand");
            let mut extraction_buffer = Vec::new();

            let mut processed_delta = 0;
            for password in password_iter {
                let password_bytes = password.as_bytes();
                let mut potential_match = true;

                // process AES KEY
                if derived_key_len != 0 {
                    // use PBKDF2 with HMAC-Sha1 to derive the key
                    pbkdf2::pbkdf2::<Hmac<Sha1>>(password_bytes, &salt, 1000, &mut derived_key);
                    let pwd_verify = &derived_key[derived_key_len - 2..];
                    // the last 2 bytes should equal the password verification value
                    potential_match = key == pwd_verify;
                }

                // ZipCrypto falls back directly here and will recompute its key for each password
                if potential_match {
                    // From the Rust doc:
                    // This function sometimes accepts wrong password. This is because the ZIP spec only allows us to check for a 1/256 chance that the password is correct.
                    // There are many passwords out there that will also pass the validity checks we are able to perform.
                    // This is a weakness of the ZipCrypto algorithm, due to its fairly primitive approach to cryptography.
                    let res = archive.by_index_decrypt(0, password_bytes);
                    match res {
                        Ok(Err(_)) => (), // invalid password
                        Ok(Ok(mut zip)) => {
                            // Validate password by reading the zip file to make sure it is not merely a hash collision.
                            extraction_buffer.reserve(zip.size() as usize);
                            match zip.read_to_end(&mut extraction_buffer) {
                                Err(_) => (), // password collision - continue
                                Ok(_) => {
                                    // Send password and continue processing while waiting for signal
                                    send_password_found
                                        .send(password)
                                        .expect("Send found password should not fail");
                                }
                            }
                            extraction_buffer.clear();
                        }
                        Err(e) => panic!("Unexpected error {e:?}"),
                    }
                }
                processed_delta + 1;
                //do not check internal flags too often
                if processed_delta == batching_dalta {
                    if first_worker {
                        progress_bar.inc(progress_bar_delta);
                    }
                    if stop_workers_signal.load(Ordering::Relaxed) {
                        break;
                    }
                    processed_delta = 0;
                }
            }
        })
        .unwrap()
}
