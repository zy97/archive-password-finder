use std::{
    io::{Cursor, Read},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender};
use indicatif::ProgressBar;
use zip::ZipArchive;

// 使用fs::read 读取文件，并用Curosr包裹buffer，并把cursor传给ZipArchive，速度一下冲26w/s到了380w/s

pub fn password_checker(
    index: usize,
    zip_file: &[u8],
    receive_password: Receiver<String>,
    stop_signal: Arc<AtomicBool>,
    send_password_found: Sender<String>,
    progress_bar: ProgressBar,
) -> JoinHandle<()> {
    let cursor = Cursor::new(zip_file.to_vec());
    thread::Builder::new()
        .name(format!("worker-{}", index))
        .spawn(move || {
            let mut archive = ZipArchive::new(cursor).expect("Archive validated before-hand");

            while !stop_signal.load(Ordering::Relaxed) {
                match receive_password.recv() {
                    Ok(password) => {
                        // 从rust文档了解到，该函数有时会接受错误密码，这是因为zip规范只允许我们检查密码是否有1/256的概率是正确的
                        // 有很多密码也能通过该方法的校验，但是实际上是错误的
                        // 这是ZipCrypto算法的一个缺陷，因为它是相当原始的密码学方法
                        let res = archive.by_index_decrypt(0, password.as_bytes());
                        match res {
                            Ok(Ok(mut zip)) => {
                                // 通过读取zip文件来验证密码，以确保它不仅仅是一个hash碰撞
                                // 不用关心重用缓冲区，因为冲突是极少的
                                let mut buffer = Vec::with_capacity(zip.size() as usize);
                                match zip.read_to_end(&mut buffer) {
                                    Ok(_) => {
                                        send_password_found
                                            .send(password)
                                            .expect("Send found password should not fail");
                                    }
                                    Err(_) => {}
                                }
                            }
                            Ok(Err(_)) => (),
                            Err(e) => panic!("Unexpected error: {}", e),
                        }
                        progress_bar.inc(1);
                    }
                    Err(_) => break,
                }
            }
        })
        .unwrap()
}
