use std::{
    fs::{self},
    path::PathBuf,
};

use rayon::{prelude::ParallelIterator, str::ParallelString};

pub struct PasswordReader {
    pub lines: Vec<String>,
}
impl PasswordReader {
    pub fn new(file_path: PathBuf) -> Self {
        let dic = fs::read_to_string(&file_path).unwrap_or_else(|_| {
            panic!(
                "Failed reading the dictionary file: {}",
                file_path.display()
            )
        });
        let lines = dic.par_lines().map(|f| f.to_string()).collect::<Vec<_>>();
        PasswordReader { lines }
    }
    pub fn len(&self) -> usize {
        self.lines.len()
    }
}
// impl Iterator for PasswordReader<'_> {
//     type Item = String;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.lines.next().map(|s| s.to_string())
//     }
// }
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use indicatif::HumanDuration;
    #[test]
    fn read_dic_test() {
        let start = std::time::Instant::now();
        let dic = super::PasswordReader::new(PathBuf::from("xato-net-10-million-passwords.txt"));
        assert_eq!(dic.len(), 5189454);
        let stop = start.elapsed();
        println!("Duration: {}", HumanDuration(stop));
    }
}
