use crate::finder_errors::FinderError;
use crate::password_finder::Strategy::{GenPasswords, PasswordFile};
use crate::password_gen::PasswordGenWorker;
use crate::password_reader::PasswordReader;
use crate::PasswordFinder;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};

pub enum Strategy {
    PasswordFile(PathBuf),
    GenPasswords {
        charsets: Vec<char>,
        min_password_len: usize,
        max_password_len: usize,
    },
}

pub fn password_finder(zip_path: &str, strategy: Strategy) -> Result<Option<String>, FinderError> {
    let zip_path = Path::new(zip_path);

    let password_finder: Box<dyn PasswordFinder> = match strategy {
        GenPasswords {
            charsets,
            min_password_len,
            max_password_len,
        } => {
            let password_gen_worker =
                PasswordGenWorker::new(charsets, min_password_len, max_password_len);
            Box::new(password_gen_worker)
        }
        PasswordFile(password_file_path) => {
            let password_reader = PasswordReader::new(password_file_path);
            Box::new(password_reader)
        }
    };

    let password = password_finder.find_password(zip_path.to_path_buf());
    match password {
        Some(password) => {
            println!("Found password: {}", password);
        }
        None => {
            println!("Password not found");
        }
    }

    Ok(None)
}
pub fn create_progress_bar(len: u64) -> ProgressBar {
    //设置进度条 进度条的样式也会影响性能，进度条越简单性能也好，影响比较小
    let progress_bar = ProgressBar::new(len).with_finish(indicatif::ProgressFinish::AndLeave);
    let progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {spinner} {pos:7}/{len:7} throughput:{per_sec} (eta:{eta})")
        .expect("Failed to create progress style");
    progress_bar.set_style(progress_style);
    //每两秒刷新终端，避免闪烁
    // let draw_target = ProgressDrawTarget::stdout_with_hz(2);
    // progress_bar.set_draw_target(draw_target);
    progress_bar
}
