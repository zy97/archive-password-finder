use crate::{
    password_finder::create_progress_bar, password_worker::password_checker, ZipPasswordFinder,
};
use crossbeam_channel::bounded;
use indicatif::MultiProgress;
use permutator::{copy::get_cartesian_for, get_cartesian_size};
use rayon::{join, ThreadPool};
use std::{
    thread::{self},
    time::Duration,
};

pub struct PasswordGenWorker {
    charset: Vec<char>,
    min_password_len: usize,
    max_password_len: usize,
    total_password_count: usize,
    current_password_index: usize,
}
impl Iterator for PasswordGenWorker {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let current_password_index = self.current_password_index;
        let total_password_count = self.total_password_count;
        let mut passwrod_lenth = 0;
        let mut total = 0;
        if current_password_index < total_password_count {
            for i in self.min_password_len..=self.max_password_len {
                total += get_cartesian_size(self.charset.len(), i);
                if current_password_index < total {
                    passwrod_lenth = i;
                    break;
                }
            }
            let current_deep_count = get_cartesian_size(self.charset.len(), passwrod_lenth);
            let current_deep_index = current_password_index - (total - current_deep_count);
            let res = get_cartesian_for(&self.charset, passwrod_lenth, current_deep_index);
            self.current_password_index += 1;
            Some(res.unwrap().iter().collect())
        } else {
            None
        }
    }
}
impl PasswordGenWorker {
    pub fn new(charset: Vec<char>, min_password_len: usize, max_password_len: usize) -> Self {
        let mut total_count = 0;
        for i in min_password_len..=max_password_len {
            total_count += get_cartesian_size(charset.len(), i)
        }
        PasswordGenWorker {
            charset,
            min_password_len,
            max_password_len,
            total_password_count: total_count,
            current_password_index: 0,
        }
    }
}
impl ZipPasswordFinder for PasswordGenWorker {
    fn find_password(&self, zip_file: &[u8]) -> Option<String> {
        let file = zip_file.to_vec();
        let multiProgress = MultiProgress::new();
        let total_password_count = self.total_password_count as u64;
        let generate_password_bar = multiProgress.add(create_progress_bar(total_password_count));
        let check_password_bar = multiProgress.add(create_progress_bar(total_password_count));
        // let mm = multiProgress.clone();
        let min_password_len = self.min_password_len;
        let max_password_len = self.max_password_len;
        let charset = self.charset.clone();
        let (tx, rx) = bounded(1000);
        thread::spawn(move || {
            let mut current_password_index: usize = 0;
            let total_password_count: usize = total_password_count as usize;
            let mut passwrod_lenth = 0;
            let mut total = 0;
            for i in min_password_len..=max_password_len {
                total += get_cartesian_size(charset.len(), i);
            }
            loop {
                if current_password_index < total_password_count {
                    for i in min_password_len..=max_password_len {
                        if current_password_index < total {
                            passwrod_lenth = i;
                            break;
                        }
                    }
                    let current_deep_count = get_cartesian_size(charset.len(), passwrod_lenth);
                    let current_deep_index = current_password_index - (total - current_deep_count);
                    let res = get_cartesian_for(&charset, passwrod_lenth, current_deep_index);
                    current_password_index += 1;
                    match res {
                        Ok(s) => {
                            let pwd = s.iter().collect::<String>();
                            match tx.send(pwd) {
                                Ok(_) => generate_password_bar.inc(1),
                                Err(_) => println!("Error!!!!"),
                            }
                            generate_password_bar.inc(1);
                        }
                        Err(e) => panic!("{}", e),
                    }
                } else {
                    break;
                }
            }
        });

        loop {
            match rx.recv() {
                Ok(password) => {
                    check_password_bar.inc(1);
                    let password = password_checker(&password, &file);
                    match password {
                        Some(p) => return Some(p.to_string()),
                        _ => {}
                    }
                }
                Err(e) => return None,
            }
        }
        Some("".to_string())
    }
}
#[cfg(test)]
mod tests {

    use super::PasswordGenWorker;
    use indicatif::HumanDuration;
    use permutator::{
        cartesian_product,
        copy::{get_cartesian_for, Combination},
        get_cartesian_size, get_permutation_for, CartesianProduct, CartesianProductIterator,
        LargeCombinationIterator, Permutation,
    };
    use rayon::prelude::{ParallelBridge, ParallelIterator};

    #[test]
    fn test_password_gen() {
        let start = std::time::Instant::now();
        let charset_letters = vec![
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ];
        let charset_uppercase_letters = vec![
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ];
        let charset_digits = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let charset_punctuations = vec![
            ' ', '-', '=', '!', '@', '#', '$', '%', '^', '&', '*', '_', '+', '<', '>', '/', '?',
            '.', ';', ':', '{', '}',
        ];
        let charset = vec![
            charset_letters,
            charset_uppercase_letters,
            charset_digits,
            charset_punctuations,
        ]
        .concat();
        let num = get_cartesian_size(charset.len(), 10);
        println!("num:{}", num);
        let stop = start.elapsed();
        let sds = get_cartesian_for(&charset, 10, num - 1651981111);
        println!("sds:{:?}", sds);
        println!("Duration: {}", HumanDuration(stop));
    }
    #[test]
    fn test_password_gen1() {
        let start = std::time::Instant::now();
        let charset_letters = vec![
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ];
        let charset_uppercase_letters = vec![
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ];
        let charset_digits = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let charset_punctuations = vec![
            ' ', '-', '=', '!', '@', '#', '$', '%', '^', '&', '*', '_', '+', '<', '>', '/', '?',
            '.', ';', ':', '{', '}',
        ];
        let charset = vec![
            // charset_letters,
            // charset_uppercase_letters,
            charset_digits,
            // charset_punctuations,
        ]
        .concat();
        let a = charset.as_slice();
        let sdfsdfasdf = vec![a; 7];
        let a = sdfsdfasdf.as_slice();
        let combine = CartesianProductIterator::new(a);
        let sdf = combine.par_bridge().collect::<Vec<_>>().len();

        // combine.par_bridge().for_each(|op| {
        //     println!("{:?}", op);
        // });
        println!("sdf:{}", sdf);
        let stop = start.elapsed();
        println!("Duration: {}", HumanDuration(stop));
    }
    #[test]
    fn test_password_gen2() {
        let start = std::time::Instant::now();
        let charset_letters = vec![
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ];
        let charset_uppercase_letters = vec![
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ];
        let charset_digits = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let charset_punctuations = vec![
            ' ', '-', '=', '!', '@', '#', '$', '%', '^', '&', '*', '_', '+', '<', '>', '/', '?',
            '.', ';', ':', '{', '}',
        ];
        let charset = vec![
            // charset_letters,
            // charset_uppercase_letters,
            charset_digits,
            // charset_punctuations,
        ]
        .concat();
        let mut generator = PasswordGenWorker::new(charset, 7, 7);
        let res = generator.collect::<Vec<_>>();
        println!("res:{:?}", res.len());
        // println!("res:{:?}", res.last());
        let stop = start.elapsed();
        println!("Duration: {}", HumanDuration(stop));
    }
}
