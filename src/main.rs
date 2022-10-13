mod password_finder;
mod password_reader;
mod password_worker;

use std::{
    cmp::max,
    env,
    io::{stdin, Read},
};

use crate::password_finder::password_finder;

fn main() {
    let zip_path = env::args().nth(1).expect("No zip file provided");
    let dictionary_path = "xato-net-10-million-passwords.txt";
    let num_cores = num_cpus::get_physical();
    println!("Using {} cores", num_cores);
    let workers = max(1, num_cores - 1);
    match password_finder(&zip_path, dictionary_path, workers) {
        Some(password) => println!("Password found: {}", password),
        None => println!("Password not found"),
    }
    println!("按回车键退出...");
    stdin().read(&mut [0]).unwrap();
}
