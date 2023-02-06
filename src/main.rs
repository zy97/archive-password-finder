mod args;
mod charsets;
mod finder_errors;
mod password_finder;
mod password_gen;
mod password_reader;
mod password_worker;
mod pdf;
mod progress_bar;
mod rar;
mod seven_z;
mod zip;

use crate::password_finder::password_finder;
use crate::password_finder::Strategy::{GenPasswords, PasswordFile};
use args::{get_args, Arguments};
use finder_errors::FinderError;
use password_finder::Strategy;
use std::path::PathBuf;
use std::{path::Path, process::exit};
fn main() {
    let result = main_result();
    exit(match result {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    })
}
fn main_result() -> Result<(), FinderError> {
    let Arguments {
        input_file,
        charsets,
        workers,
        min_password_len,
        max_password_len,
        password_dictionary,
        custom_chars,
    } = get_args()?;
    let mut charsets = vec![charsets, custom_chars].concat();
    charsets.sort();
    charsets.dedup();
    let strategy = match password_dictionary {
        Some(dict_path) => {
            let path = Path::new(&dict_path);
            PasswordFile(path.to_path_buf())
        }
        None => GenPasswords {
            charsets,
            min_password_len,
            max_password_len,
        },
    };
    password_finder(
        &input_file,
        workers.unwrap_or_else(num_cpus::get_physical),
        strategy,
    )?;
    Ok(())
}
pub trait PasswordFinder {
    fn find_password(&self, compressed_file: PathBuf, strategy: Strategy) -> Option<String>;
}
type Passwords = Box<dyn Iterator<Item = String>>;
fn filter_for_worker_index(
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
