mod args;
mod finder_errors;
mod password_finder;
mod password_gen;
mod password_reader;
mod password_worker;
use crate::password_finder::password_finder;
use crate::password_finder::Strategy::{GenPasswords, PasswordFile};
use args::{get_args, Arguments};
use finder_errors::FinderError;
use indicatif::ProgressBar;
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
        charset,
        min_password_len,
        max_password_len,
        password_dictionary,
        custom_chars,
    } = get_args()?;
    let mut charset = charset;
    charset.retain(|f| vec!["number", "upper", "lower", "special"].contains(&f.as_str()));
    charset.sort();
    charset.dedup();
    if charset.len() == 0 {
        charset.push("number".to_string());
    }
    let strategy = match password_dictionary {
        Some(dict_path) => {
            let path = Path::new(&dict_path);
            PasswordFile(path.to_path_buf())
        }
        None => GenPasswords {
            charset_choice: charset,
            min_password_len,
            max_password_len,
            custom_chars,
        },
    };
    password_finder(&input_file, strategy)?;
    Ok(())
}
pub trait ZipPasswordFinder {
    fn find_password(&self, zip_file: &[u8]) -> Option<String>;
}
