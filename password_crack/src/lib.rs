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

pub use charsets::CharsetChoice;
pub use errors::Errors;
pub use password_finder::Strategy;
pub use password_finder::{get_password_count, password_finder};

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
