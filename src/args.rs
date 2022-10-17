use crate::finder_errors::FinderError;
use crate::finder_errors::FinderError::CliArgumentError;
use crate::password_finder::CharsetChoice;
use clap::builder::EnumValueParser;
use clap::{crate_authors, crate_description, crate_name, crate_version, value_parser};
use clap::{Arg, Command};
use std::path::Path;

fn command() -> clap::Command {
    Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::new("inputFile")
                .help("path to zip input file")
                .long("inputFile")
                .short('i')
                .num_args(1)
                .required(true),
        )
        .arg(
            Arg::new("workers")
                .value_parser(value_parser!(usize))
                .help("number of workers")
                .long("workers")
                .short('w')
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("passwordDictionary")
                .help("path to a password dictionary file")
                .long("passwordDictionary")
                .short('p')
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("charset")
                .help("charset to use to generate password")
                .long("charset")
                .short('c')
                .value_parser(EnumValueParser::<CharsetChoice>::new())
                .default_value("medium")
                .required(false),
        )
        .arg(
            Arg::new("minPasswordLen")
                .value_parser(value_parser!(usize))
                .help("minimum password length")
                .long("minPasswordLen")
                .num_args(1)
                .default_value("1")
                .required(false),
        )
        .arg(
            Arg::new("maxPasswordLen")
                .value_parser(value_parser!(usize))
                .help("maximum password length")
                .long("maxPasswordLen")
                .num_args(1)
                .default_value("10")
                .required(false),
        )
}

pub struct Arguments {
    pub input_file: String,
    pub workers: Option<usize>,
    pub charset: CharsetChoice,
    pub min_password_len: usize,
    pub max_password_len: usize,
    pub password_dictionary: Option<String>,
}

pub fn get_args() -> Result<Arguments, FinderError> {
    let command = command();
    let matches = command.get_matches();

    let input_file: &String = matches.get_one("inputFile").expect("impossible");
    if !Path::new(input_file).is_file() {
        return Err(CliArgumentError {
            message: "'inputFile' does not exist".to_string(),
        });
    }

    let password_dictionary = matches.try_get_one("passwordDictionary")?;
    if let Some(dict_path) = password_dictionary {
        if !Path::new(dict_path).is_file() {
            return Err(CliArgumentError {
                message: "'passwordDictionary' does not exist".to_string(),
            });
        }
    }

    let charset: CharsetChoice = *matches.get_one("charset").expect("impossible");

    let workers = matches.try_get_one("workers")?;
    if workers == Some(&0) {
        return Err(CliArgumentError {
            message: "'workers' must be positive".to_string(),
        });
    }

    let min_password_len = matches.get_one("minPasswordLen").expect("impossible");
    if *min_password_len == 0 {
        return Err(CliArgumentError {
            message: "'minPasswordLen' must be positive".to_string(),
        });
    }

    let max_password_len = matches.get_one("maxPasswordLen").expect("impossible");
    if *max_password_len == 0 {
        return Err(CliArgumentError {
            message: "'maxPasswordLen' must be positive".to_string(),
        });
    }

    if min_password_len > max_password_len {
        return Err(CliArgumentError {
            message: "'minPasswordLen' must be less than 'maxPasswordLen'".to_string(),
        });
    }

    Ok(Arguments {
        input_file: input_file.clone(),
        workers: workers.cloned(),
        charset,
        min_password_len: *min_password_len,
        max_password_len: *max_password_len,
        password_dictionary: password_dictionary.cloned(),
    })
}

#[cfg(test)]
mod argr_tests {
    use crate::args::command;

    #[test]
    fn virify_command() {
        command().debug_assert();
    }
}
