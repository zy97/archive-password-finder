mod charsets;
mod errors;
mod password_finder;
mod password_gen;
mod password_reader;
mod password_worker;
#[cfg(feature = "pdf")]
mod pdf;
#[cfg(feature = "rar")]
mod rar;
#[cfg(feature = "7z")]
mod seven_z;
mod zip;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub use charsets::{charset_lowercase_letters, CharsetChoice};
pub use errors::Errors;
pub use password_finder::Strategy;
use password_finder::{get_password_count, password_finder};
pub use password_gen::PasswordGenerator;

type Passwords = Box<dyn Iterator<Item = String> + Send>;
fn filter_for_worker_index(
    passwords: Passwords,
    worker_count: usize,
    worker_index: usize,
) -> Passwords {
    if worker_count > 1 {
        Box::new(passwords.enumerate().filter_map(move |(index, password)| {
            if index % worker_count == worker_index - 1 {
                Some(password)
            } else {
                None
            }
        }))
    } else {
        passwords
    }
}
#[derive(Clone)]
pub struct Cracker {
    file_path: String,
    workers: usize,
    strategy: Strategy,
    total_count: Option<usize>,
    tested_count: Arc<AtomicU64>,
}
impl Cracker {
    pub fn new(file_path: String, workers: usize, strategy: Strategy) -> Self {
        Cracker {
            file_path,
            workers,
            strategy,
            total_count: None,
            tested_count: Arc::new(AtomicU64::new(0)),
        }
    }
    pub fn start(self: &Self) -> Result<Option<String>, Errors> {
        password_finder(
            &self.file_path,
            self.workers,
            self.strategy.clone(),
            self.tested_count.clone(),
        )
    }
    pub fn count(self: &Self) -> Result<usize, Errors> {
        match self.total_count {
            Some(c) => Ok(c),
            None => get_password_count(&self.strategy),
        }
    }
    pub fn tested_count(self: &Self) -> u64 {
        self.tested_count.load(Ordering::SeqCst)
    }
}
