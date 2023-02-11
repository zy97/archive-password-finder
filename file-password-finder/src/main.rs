mod args;
mod cli_error;

use args::{get_args, Arguments};
use cli_error::CLIError;
use password_crack::{password_finder, Strategy};
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
fn main_result() -> Result<(), CLIError> {
    let Arguments {
        input_file,
        charsets,
        workers,
        min_password_len,
        max_password_len,
        password_dictionary,
        custom_chars,
    } = get_args()?;
    let mut charsets = if custom_chars.len() > 0 {
        custom_chars
    } else {
        charsets
    };
    charsets.sort();
    charsets.dedup();
    let strategy = match password_dictionary {
        Some(dict_path) => {
            let path = Path::new(&dict_path);
            Strategy::PasswordFile(path.to_path_buf())
        }
        None => Strategy::GenPasswords {
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
