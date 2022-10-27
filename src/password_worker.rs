use std::{
    io::{Cursor, Read},
    process::Command,
};
use zip::ZipArchive;

// 使用fs::read 读取文件，并用Curosr包裹buffer，并把cursor传给ZipArchive，速度一下冲26w/s到了380w/s
// zip_file 使用&[u8]作为参数，性能没有什么变化
pub fn password_checker<'a>(password: &'a str, zip_file: &[u8]) -> Option<&'a str> {
    let cursor = Cursor::new(zip_file);

    let mut archive = ZipArchive::new(cursor).expect("Archive validated before-hand");
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
                Ok(_) => Some(password),
                Err(_) => None,
            }
        }
        _ => None,
    }
}
pub fn rar_password_checker<'a>(password: &'a str, rar_file_path: String) -> Option<&'a str> {
    let archive = unrar::Archive::with_password(rar_file_path, password.to_string());
    let mut open_archive = archive.test().unwrap();
    if open_archive.process().is_ok() {
        Some(password)
    } else {
        None
    }
}
pub fn sevenz_password_checker<'a>(password: &'a str, rar_file_path: String) -> Option<&'a str> {
    let mut command = Command::new(r".\7z.exe");
    command.arg("t");
    command.arg(rar_file_path);
    command.arg(format!("-p{}", password));
    let output = command.output().unwrap();
    match output.status.code() {
        Some(0) => Some(password),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use std::{path::Path, process::Command, time::Instant};

    #[test]
    fn test() {
        let now = Instant::now();
        let zip = "test1.7z";
        let mut command = Command::new(r".\7z.exe");
        command.arg("t");
        command.arg(zip);
        // command.arg(format!("-p{}", "123456"));
        let output = command.output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        println!("stdout: {}", stdout);
        println!("code : {}", output.status.code().unwrap());
        println!("time: {:?}", now.elapsed());
    }
}
