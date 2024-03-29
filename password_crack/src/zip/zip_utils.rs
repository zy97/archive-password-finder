use std::fs::File;
use std::path::Path;
use zip::result::ZipError::UnsupportedArchive;

use crate::errors::Errors;

#[derive(Clone, Debug)]
pub struct AesInfo {
    pub aes_key_length: usize,
    pub key: Vec<u8>,
    pub derived_key_length: usize,
    pub salt: Vec<u8>,
}

impl AesInfo {
    pub fn new(aes_key_length: usize, key: Vec<u8>, salt: Vec<u8>) -> Self {
        // derive a key from the password and salt
        // the length depends on the aes key length
        let derived_key_length = 2 * aes_key_length + 2;
        AesInfo {
            aes_key_length,
            key,
            derived_key_length,
            salt,
        }
    }
}

// validate that the zip requires a password
pub fn validate_zip(file_path: &Path, show_info: bool) -> Result<Option<AesInfo>, Errors> {
    let file = File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let aes_data = archive.get_aes_key_and_salt(0);
    let zip_result = archive.by_index(0);
    let aes_info = match zip_result {
        Ok(_) => Err(Errors::invalid_zip_error(
            "the archive is not encrypted".to_string(),
        )),
        Err(UnsupportedArchive(msg)) if msg == "Password required to decrypt file" => {
            if let Some((aes_mode, key_, salt_)) = aes_data.expect("Archive validated before-hand")
            {
                let aes_key_length = aes_mode.key_length();
                let key = key_;
                let salt = salt_;
                Ok(Some(AesInfo::new(aes_key_length, key, salt)))
            } else {
                Ok(None)
            }
        }
        Err(e) => Err(Errors::invalid_zip_error(format!("Unexpected error {e:?}"))),
    }?;
    if show_info {
        match &aes_info {
            Some(aes_info) => println!(
                "Archive is encrypted with AES{} - expect a long wait time",
                aes_info.aes_key_length * 8
            ),
            None => {
                println!("Archive is encrypted with ZipCrypto - expect a much faster throughput")
            }
        }
    }

    Ok(aes_info)
}
