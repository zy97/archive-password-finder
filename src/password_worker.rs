use std::io::{Cursor, Read};
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

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        let archive = unrar::Archive::with_password("test.rar".into(), "123456".to_string());
        // let archive = unrar::Archive::new("test.rar".into());
        // let mut sd = archive
        //     .open(
        //         unrar::archive::OpenMode::Extract,
        //         Some("()".to_string()),
        //         unrar::archive::Operation::Extract,
        //     )
        //     .unwrap();
        let mut sd = archive.test().unwrap();
        sd.process().unwrap();
        println!("111:{:?}", sd);
        // for entry in archive.list().unwrap() {
        //     println!("{}", entry.unwrap());
        // }
    }
}
