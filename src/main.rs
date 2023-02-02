mod args;
mod charsets;
mod finder_errors;
mod password_finder;
mod password_gen;
mod password_reader;
mod password_worker;
use crate::password_finder::password_finder;
use crate::password_finder::Strategy::{GenPasswords, PasswordFile};
use args::{get_args, Arguments};
use finder_errors::FinderError;
use itertools::Itertools;
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
    // println!("按回车键退出...");
    // stdin().read(&mut [0]).unwrap();
}
fn main_result() -> Result<(), FinderError> {
    let Arguments {
        input_file,
        charsets,
        min_password_len,
        max_password_len,
        password_dictionary,
        custom_chars,
    } = get_args()?;
    let mut charsets = vec![charsets, custom_chars].concat();
    charsets.sort();
    charsets.dedup();
    println!("{:?}", charsets);
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
    password_finder(&input_file, strategy)?;
    Ok(())
}
pub trait PasswordFinder {
    fn find_password(&self, compressed_file: PathBuf) -> Option<String>;
}
