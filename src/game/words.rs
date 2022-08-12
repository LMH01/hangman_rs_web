use std::fs;
use rand::Rng;

/// Reads the words file and returns a random word
pub fn random_word() -> String {
    let file = fs::read_to_string("../../words.txt").expect("Unable to read words file!");
    let words: Vec<&str> = file.split("\n").collect();
    let number = rand::thread_rng().gen_range(0..words.len());
    String::from(words[number])
}