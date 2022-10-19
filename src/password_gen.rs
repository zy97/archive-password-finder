use std::{
    iter::repeat,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crossbeam_channel::Sender;
use indicatif::ProgressBar;
use permutator::{copy::get_cartesian_for, get_cartesian_size};

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

pub fn start_password_generation(
    charset: Vec<char>,
    min_size: usize,
    max_size: usize,
    send_password: Sender<String>,
    stop_signal: Arc<AtomicBool>,
    progress_bar: ProgressBar,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("password-gen".to_string())
        .spawn(move || {
            let charset_len = charset.len();
            progress_bar.println(format!(
                "Generating passwords with length from {} to {} for charset with length {}\n{:?}",
                min_size, max_size, charset_len, charset,
            ));
            let charset_first = charset.first().expect("charset non empty");
            let charset_last = charset.last().expect("charset non empty");

            let mut password = if min_size == 1 {
                progress_bar.println(format!(
                    "Starting search space for password length {} ({} possibilities)",
                    min_size, charset_len
                ));
                vec![charset_first; 1]
            } else {
                vec![charset_last; min_size - 1]
            };
            let mut current_len = password.len();
            let mut current_index = current_len - 1;
            let mut generated_count = 0;

            while password.len() < max_size + 1 && !stop_signal.load(Ordering::Relaxed) {
                if current_len == current_index + 1 && !password.iter().any(|&c| c != charset_last)
                {
                    // 增加长度并重置字母
                    current_index += 1;
                    current_len += 1;
                    password = Vec::from_iter(repeat(charset_first).take(current_len));

                    let possibilities = charset_len.pow(current_len as u32);
                    progress_bar.println(format!( "Starting search space for password length {} ({} possibilities) ({} passwords generated so far)",
                    current_len, possibilities, generated_count));
                }
                else{
                    let current_char = *password.get(current_index).unwrap();
                    if current_char == charset_last{
                        // 当前字符到达字符集的末尾，重置当前字符并碰撞前面的字符
                        let at_prev = password.iter()
                        .rposition(|&c| c != charset_last)
                        .unwrap_or_else(|| panic!("Must find something else than {} in {:?}", charset_last, password));
                        let next_prev = if at_prev == charset_len - 1 {
                            charset.get(charset_len - 1).unwrap()
                        } else {
                            let prev_char = *password.get(at_prev).unwrap();
                            let prev_index_charset =
                                charset.iter().position(|c| c == prev_char).unwrap();
                            charset.get(prev_index_charset + 1).unwrap()
                        };
                        let mut tmp = Vec::with_capacity(current_len);
                        for (i, x) in password.into_iter().enumerate() {
                            if i == current_index {
                                tmp.push(charset_first)
                            } else if i == at_prev {
                                tmp.push(next_prev)
                            } else if x == charset_last && i > at_prev {
                                tmp.push(charset_first)
                            } else {
                                tmp.push(x);
                            }
                        }
                        password = tmp;
                    }
                    else{
                        // 增加当前字符
                        let at = charset.iter().position(|c| c == current_char).unwrap();
                        let next = if at == charset_len - 1 {
                            charset_first
                        } else {
                            charset.get(at + 1).unwrap()
                        };

                        //println!("in-place char:{}, index in charset:{}", current_char, at);

                        let mut tmp = Vec::with_capacity(current_len);
                        for (i, x) in password.iter().enumerate() {
                            if i == current_index {
                                tmp.push(next)
                            } else {
                                tmp.push(*x);
                            }
                        }
                        password = tmp;
                    }
                }
                let to_push = password.iter().cloned().collect::<String>();
                generated_count +1;
                send_password.send(to_push).unwrap();
            }
        })
        .unwrap()
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
