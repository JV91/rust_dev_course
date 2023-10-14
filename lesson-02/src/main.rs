use slug::slugify;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if there is correct number of CLI arguments
    if args.len() != 3 {
        println!("Correct input format: Cargo run <text> <modifier>");
        println!(
            "Valid <modifier> values are: lowercase, uppercase, no-spaces, slugify, reverse, rot13"
        );
        return;
    }

    let text = &args[1];
    let modifier = &args[2];

    let modified_text = match modifier.as_str() {
        "lowercase" => text.to_lowercase(),
        "uppercase" => text.to_uppercase(),
        "no-spaces" => text.replace(' ', ""),
        "slugify" => slugify(text),
        "reverse" => reverse(text),
        "rot13" => rot13(text),
        _ => {
            println!("Unknown modifier: {}", modifier);
            println!(
                "Valid <modifier> values are: lowercase, uppercase, no-spaces, slugify, reverse, rot13"
            );
            return;
        }
    };

    println!(
        "Output string with '{}' modifier: {}",
        modifier, modified_text
    );
}

// simple function for reversing a string
fn reverse(input: &str) -> String {
    input.chars().rev().collect()
}

// function that applies ROT13 transformation to alphabetic chars while leaving other chars unchanged
fn rot13(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                (((c as u8 - base + 13) % 26) + base) as char
            } else {
                c
            }
        })
        .collect()
}
