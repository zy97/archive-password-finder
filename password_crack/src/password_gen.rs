use std::{collections::HashMap, iter::repeat};

use ahash::AHashMap;

use crate::Errors;
pub fn password_generator_count(
    charset: &Vec<char>,
    min_size: usize,
    max_size: usize,
) -> Result<usize, Errors> {
    // compute the number of passwords to generate
    let charset_len = charset.len();
    let mut total_password_count: usize = 0;
    for i in min_size..=max_size {
        let a = charset_len.checked_pow(i as u32).ok_or(Errors::MathError {
            message: String::from("算术溢出"),
        })?;
        total_password_count = total_password_count
            .checked_add(a)
            .ok_or(Errors::MathError {
                message: String::from("算术溢出"),
            })?;
    }
    Ok(total_password_count)
}
pub struct PasswordGenerator {
    charset: Vec<char>,
    charset_indices: AHashMap<char, usize>,
    charset_len: usize,
    charset_first: char,
    charset_last: char,
    max_size: usize,
    current_len: usize,
    current_index: usize,
    generated_count: usize,
    total_to_generate: usize,
    password: Vec<char>,
}
impl Iterator for PasswordGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.password.len() > self.max_size {
            return None;
        }

        // first password
        if self.generated_count == 0 {
            self.generated_count += 1;
            return Some(self.password.iter().collect());
        }

        // end of search space
        if self.generated_count == self.total_to_generate {
            return None;
        }

        // check if we need to increase the length of the password
        if self.current_len == self.current_index + 1
            && !self.password.iter().any(|&c| c != self.charset_last)
        {
            // increase length and reset letters
            self.current_index += 1;
            self.current_len += 1;
            self.password = vec![self.charset_first; self.current_len];
            // let possibilities = self.charset_len.pow(self.current_len as u32);
        } else {
            let current_char = *self.password.get(self.current_index).unwrap();
            if current_char == self.charset_last {
                // current char reached the end of the charset, reset current and bump previous
                let at_prev = self
                    .password
                    .iter()
                    .rposition(|&c| c != self.charset_last)
                    .unwrap_or_else(|| {
                        panic!(
                            "Must find something else than {} in {:?}",
                            self.charset_last, self.password
                        )
                    });
                let next_prev = if at_prev == self.charset_len - 1 {
                    self.charset.get(self.charset_len - 1).unwrap()
                } else {
                    let prev_char = *self.password.get(at_prev).unwrap();
                    let prev_index_charset =
                        self.charset.iter().position(|&c| c == prev_char).unwrap();
                    self.charset.get(prev_index_charset + 1).unwrap()
                };

                self.password[self.current_index] = self.charset_first;
                self.password[at_prev] = *next_prev;

                // reset all chars after previous
                for (i, x) in self.password.iter_mut().enumerate() {
                    if *x == self.charset_last && i > at_prev {
                        *x = self.charset_first
                    }
                }
            } else {
                // hot-path: increment current char (not at the end of charset)
                let at = *self.charset_indices.get(&current_char).unwrap();
                let next = *self.charset.get(at + 1).unwrap();
                self.password[self.current_index] = next;
            }
        }
        self.generated_count += 1;
        // TODO explore using a lending iterator to avoid allocation
        Some(self.password.iter().collect::<String>())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.total_to_generate - self.generated_count;
        (remaining, Some(remaining))
    }
}
impl PasswordGenerator {
    pub fn new(charset: Vec<char>, min_size: usize, max_size: usize) -> Self {
        let charset_len = charset.len();
        let charset_first = *charset.first().expect("charset non empty");
        let charset_last = *charset.last().expect("charset non empty");
        // pre-compute charset indices
        let charset_indices = charset
            .iter()
            .enumerate()
            .map(|(i, c)| (*c, i))
            .collect::<AHashMap<char, usize>>();

        let password = vec![charset_first; min_size];
        let current_len = password.len();
        let current_index = current_len - 1;

        let generated_count = 0;
        let total_to_generate = password_generator_count(&charset, min_size, max_size).unwrap();

        PasswordGenerator {
            charset,
            charset_indices,
            charset_len,
            charset_first,
            charset_last,
            max_size,
            current_len,
            current_index,
            generated_count,
            total_to_generate,
            password,
        }
    }
}

#[cfg(test)]

mod test {

    use super::PasswordGenerator;
    use std::clone;
    use std::fmt::Write;
    use std::time::Instant;

    #[test]
    fn tsd() {
        let chars = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        // let chars = vec!['0', '1', '2'];
        let min = 1;
        let max = 6;
        let now = Instant::now();
        let password_generator = PasswordGenerator::new(chars.clone(), min, max);
        let count = password_generator.count();
        println!("{:<10}ms,{}", now.elapsed().as_millis(), count);

        let now = Instant::now();
        let vec2 = generate_permutations(chars.clone(), min, max);
        println!("{:<10}ms,{}", now.elapsed().as_millis(), vec2.len());
    }
    struct TestA {
        chars: Vec<char>,
        min_length: usize,
        max_length: usize,
        current_len: usize,
        prefix: Vec<char>,
    }
    impl TestA {
        fn new(chars: Vec<char>, min_length: usize, max_length: usize) -> Self {
            Self {
                chars,
                min_length,
                max_length,
                current_len: min_length,
                prefix: vec![],
            }
        }
        fn generate_permutations(
            chars: Vec<char>,
            min_length: usize,
            max_length: usize,
        ) -> Vec<String> {
            let mut passwords = vec![];
            for length in min_length..=max_length {
                let mut prefix = Vec::with_capacity(length);
                generate_permutations_helper(&chars, &mut prefix, length, &mut passwords);
            }
            passwords
        }

        fn generate_permutations_helper(
            chars: &[char],
            prefix: &mut Vec<char>,
            length: usize,
            password: &mut Vec<String>,
        ) {
            if length == 0 {
                let mut s = String::with_capacity(prefix.len());
                for c in prefix {
                    write!(&mut s, "{}", c).unwrap();
                }
                password.push(s);
            } else {
                for c in chars {
                    prefix.push(*c);
                    generate_permutations_helper(chars, prefix, length - 1, password);
                    prefix.pop();
                }
            }
        }
    }
    impl Iterator for TestA {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            todo!()
        }
    }
    fn generate_permutations(
        chars: Vec<char>,
        min_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut passwords = vec![];
        for length in min_length..=max_length {
            let mut prefix = Vec::with_capacity(length);
            generate_permutations_helper(&chars, &mut prefix, length, &mut passwords);
        }
        passwords
    }

    fn generate_permutations_helper(
        chars: &[char],
        prefix: &mut Vec<char>,
        length: usize,
        password: &mut Vec<String>,
    ) {
        if length == 0 {
            let mut s = String::with_capacity(prefix.len());
            for c in prefix {
                write!(&mut s, "{}", c).unwrap();
            }
            password.push(s);
        } else {
            for c in chars {
                prefix.push(*c);
                generate_permutations_helper(chars, prefix, length - 1, password);
                prefix.pop();
            }
        }
    }
}
