use thiserror::Error;
use zip::result::ZipError;
#[derive(Error, Debug)]
pub enum Errors {
    #[error("standard I/O error - {e}")]
    StdIoError { e: std::io::Error },
    #[error("Invalid zip file error - {message}")]
    InvalidZip { message: String },
}
impl Errors {
    pub fn invalid_zip_error(message: String) -> Self {
        Errors::InvalidZip { message }
    }
}
impl std::convert::From<std::io::Error> for Errors {
    fn from(e: std::io::Error) -> Self {
        Errors::StdIoError { e }
    }
}

impl std::convert::From<ZipError> for Errors {
    fn from(e: ZipError) -> Self {
        Errors::InvalidZip {
            message: format!("{}", e),
        }
    }
}
