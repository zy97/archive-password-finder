use crate::finder_errors::FinderError;
use crate::password_finder::Strategy::{GenPasswords, PasswordFile};
use crate::password_gen::PasswordGenWorker;
use crate::password_reader::PasswordReader;
use crate::PasswordFinder;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
#[derive(Clone, Debug)]
pub enum Strategy {
    PasswordFile(PathBuf),
    GenPasswords {
        charsets: Vec<char>,
        min_password_len: usize,
        max_password_len: usize,
    },
}

pub fn password_finder(zip_path: &str, strategy: Strategy) -> Result<Option<String>, FinderError> {
    let zip_path = Path::new(zip_path);

    //停止与线程关闭信号量
    let stop_workers_signal = Arc::new(AtomicBool::new(false));
    let stop_gen_signal = Arc::new(AtomicBool::new(false));

    let password_finder: Box<dyn PasswordFinder> = match &strategy {
        GenPasswords {
            charsets,
            min_password_len,
            max_password_len,
        } => {
            let c = charsets.clone();
            let password_gen_worker =
                PasswordGenWorker::new(c, *min_password_len, *max_password_len);
            Box::new(password_gen_worker)
        }
        PasswordFile(password_file_path) => {
            let password_reader = PasswordReader::new(password_file_path.clone());
            Box::new(password_reader)
        }
    };

    let password = password_finder.find_password(zip_path.to_path_buf(), strategy.clone());
    match password {
        Some(password) => {
            println!("Found password: {}", password);
        }
        None => {
            println!("Password not found");
        }
    }

    Ok(None)
}
