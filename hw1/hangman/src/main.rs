extern crate rand;

use rand::prelude::*;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::rng().random_range(0..words.len())].trim())
}

fn print_chars(ch: &Vec<char>) {
    // print all the characters in the vector
    for x in ch {
        print!("{} ", x);
    }
    println!();
}

fn num_valid_chars(ch: &Vec<char>) -> usize {
    // number of characters in the vector that are not '_'
    let mut count = 0;
    for c in ch {
        if *c != '_' {
            count += 1;
        }
    }
    count
}

fn main() {
    let secret_word = pick_a_random_word();
    let secret_word_chars: Vec<char> = secret_word.chars().collect();

    let mut guesses_left = NUM_INCORRECT_GUESSES;
    let mut guessed_letters: Vec<char> = Vec::new();
    let mut current_guess: Vec<char> = vec!['_'; secret_word_chars.len()];

    while guesses_left > 0 {
        // print the characters in the current guess
        print!("\nThe word so far is ");
        print_chars(&current_guess);
        print!("You have guessed the following letters: ");
        print_chars(&guessed_letters);
        // print # of guesses left
        println!("You have {} guesses left", guesses_left);
        print!("Please guess a letter: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input_char = input.chars().next().unwrap();

        if guessed_letters.contains(&input_char) {
            println!("You already guessed '{}'", input_char);
            continue;
        }

        // adjust guessed_letters, current guess
        guessed_letters.push(input_char);
        let mut found = false;
        for i in 0..secret_word_chars.len() {
            if secret_word_chars[i] == input_char {
                current_guess[i] = input_char;
                found = true;
            }
        }

        // adjust guesses_left
        if !found {
            guesses_left -= 1;
        }

        // return if successfully guessed word
        if num_valid_chars(&current_guess) == secret_word_chars.len() {
            println!("Congratulations you guessed the secret word: {}!", secret_word);
            return;
        }
    }
    // print failure message
    println!("\nSorry, you ran out of guesses! The word was: {}", secret_word);
}
