use password_crack::Errors;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum CLIError {
    #[error("CLI argument error - {message:?}")]
    CliArgumentError { message: String },
    #[error("CLI argument error ({e})")]
    ClapError { e: clap::Error },
    #[error("CLI argument match error ({message})")]
    ClapMatchError { message: String },
    #[error("CLI argument match error ({e})")]
    PasswordCrachError { e: Errors },
}

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
        CLIError::PasswordCrachError { e: e }
    }
}
