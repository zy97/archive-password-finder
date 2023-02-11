mod args;
mod cli_error;

use args::{get_args, Arguments};
use cli_error::CLIError;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use password_crack::{get_password_count, password_finder, Strategy};
use std::thread;
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
    let total_passwords = get_password_count(&strategy)?;
    let progress_bar = create_progress_bar(total_passwords as u64);
    let workers = workers.unwrap_or_else(num_cpus::get_physical);
    println!("Starting {} workers to test passwords", workers);
    let (send_progress_info, receive_progress_info) = crossbeam_channel::unbounded();
    thread::spawn(move || loop {
        match receive_progress_info.recv() {
            Ok(info) => progress_bar.inc(info),
            Err(e) => {
                println!("{:?}", e)
            }
        }
    });
    match password_finder(&input_file, workers, strategy, send_progress_info.clone()) {
        Ok(Some(password)) => {
            println!("Found password: {}", password);
        }
        Ok(None) => {
            println!("Password not found");
        }
        Err(_) => {}
    };
    drop(send_progress_info);
    Ok(())
}

pub fn create_progress_bar(len: u64) -> ProgressBar {
    //设置进度条 进度条的样式也会影响性能，进度条越简单性能也好，影响比较小
    let progress_bar = ProgressBar::new(len).with_finish(indicatif::ProgressFinish::Abandon);
    let progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar} {pos}/{len} throughput:{per_sec} (eta:{eta})")
        .expect("Failed to create progress style");
    progress_bar.set_style(progress_style);
    //每两秒刷新终端，避免闪烁
    let draw_target = ProgressDrawTarget::stdout_with_hz(2);
    progress_bar.set_draw_target(draw_target);
    progress_bar
}
