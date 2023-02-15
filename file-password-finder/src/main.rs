mod args;
mod cli_error;

use args::{get_args, Arguments};
use cli_error::CLIError;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use password_crack::{Cracker, Strategy};

use std::sync::Arc;
use std::thread;
use std::time::Duration;
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
}
fn main_result() -> Result<(), CLIError> {
    let Arguments {
        input_file,
        charsets,
        workers,
        min_password_len,
        max_password_len,
        password_dictionary,
        custom_chars,
    } = get_args()?;
    let mut charsets = if custom_chars.len() > 0 {
        custom_chars
    } else {
        charsets
    };
    charsets.sort();
    charsets.dedup();
    let strategy = match password_dictionary {
        Some(dict_path) => {
            let path = Path::new(&dict_path);
            Strategy::PasswordFile(path.to_path_buf())
        }
        None => Strategy::GenPasswords {
            charsets,
            min_password_len,
            max_password_len,
        },
    };

    let workers = workers.unwrap_or_else(num_cpus::get_physical);
    println!("Starting {} workers to test passwords", workers);

    let crack = Arc::new(Cracker::new(input_file, workers, strategy));
    let count = crack.count()?;
    let progress_bar = Arc::new(create_progress_bar(count as u64));
    let progress_bar1 = Arc::clone(&progress_bar);
    let crack1 = Arc::clone(&crack);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(500));
        progress_bar1.set_position(crack1.tested_count());
    });
    match crack.start() {
        Ok(Some(password)) => {
            println!("Found password: {}", password);
        }
        Ok(None) => {
            println!("Password not found");
        }
        Err(_) => {}
    };
    Ok(())
}

pub fn create_progress_bar(len: u64) -> ProgressBar {
    //设置进度条 进度条的样式也会影响性能，进度条越简单性能也好，影响比较小
    let progress_bar = ProgressBar::new(len);
    let progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar} {pos}/{len} throughput:{per_sec} (eta:{eta})")
        .expect("Failed to create progress style");
    progress_bar.set_style(progress_style);
    //每两秒刷新终端，避免闪烁
    let draw_target = ProgressDrawTarget::stdout_with_hz(2);
    progress_bar.set_draw_target(draw_target);
    progress_bar
}
