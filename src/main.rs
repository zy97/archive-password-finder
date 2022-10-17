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
        workers,
        input_file,
        charset,
        min_password_len,
        max_password_len,
        password_dictionary,
    } = get_args()?;
    let strategy = match password_dictionary {
        Some(dict_path) => {
            let path = Path::new(&dict_path);
            PasswordFile(path.to_path_buf())
        }
        None => GenPasswords {
            charset_choice: charset,
            min_password_len,
            max_password_len,
        },
    };
    let workers = workers.unwrap_or_else(num_cpus::get_physical);
    password_finder(&input_file, workers, strategy)?;
    Ok(())
}
