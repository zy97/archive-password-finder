use password_crack::Errors;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum CLIError {
    #[error("standard I/O error - {e}")]
    StdIoError { e: std::io::Error },
    #[error("Invalid zip file error - {message}")]
    InvalidZip { message: String },
    #[error("CLI argument error - {message:?}")]
    CliArgumentError { message: String },
    #[error("CLI argument error ({e})")]
    ClapError { e: clap::Error },
    #[error("CLI argument match error ({message})")]
    ClapMatchError { message: String },
    #[error("CLI argument match error ({e})")]
    Te { e: Errors },
}
impl CLIError {
    pub fn invalid_zip_error(message: String) -> Self {
        CLIError::InvalidZip { message }
    }
}
impl std::convert::From<std::io::Error> for CLIError {
    fn from(e: std::io::Error) -> Self {
        CLIError::StdIoError { e }
    }
}

// impl std::convert::From<ZipError> for FinderError {
//     fn from(e: ZipError) -> Self {
//         FinderError::InvalidZip {
//             message: format!("{}", e),
//         }
//     }
// }

impl std::convert::From<clap::Error> for CLIError {
    fn from(e: clap::Error) -> Self {
        CLIError::ClapError { e }
    }
}

impl std::convert::From<clap::parser::MatchesError> for CLIError {
    fn from(e: clap::parser::MatchesError) -> Self {
        CLIError::ClapMatchError {
            message: format!("{}", e),
        }
    }
}
impl std::convert::From<Errors> for CLIError {
    fn from(e: Errors) -> Self {
        CLIError::Te { e: e }
    }
}
