use colored::*;
use std::io;

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

fn main() {
    let name = get_user_input("Please, enter your name");

    let greeting = format!("Hello, {}", name)
        .bright_red()
        .on_bright_white()
        .bold();

    println!("{}", greeting);
}
