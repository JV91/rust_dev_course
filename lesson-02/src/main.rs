use slug::slugify;
use std::env;

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

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if there is correct number of CLI arguments
    if args.len() != 3 {
        eprintln!("Error: invalid number of arguments");
        eprintln!("Correct input format: Cargo run <text> <modifier>");
        eprintln!(
            "Valid <modifier> values are: lowercase, uppercase, no-spaces, slugify, reverse, rot13"
        );
        std::process::exit(1);
    }

    let text = &args[1];
    let modifier = &args[2];

    let modified_text = match modifier.as_str() {
        "lowercase" => text.to_lowercase(),
        "uppercase" => text.to_uppercase(),
        "no-spaces" => text.replace(' ', ""),
        "slugify" => slugify(text),
        "reverse" => text.chars().rev().collect(),
        "rot13" => rot13(text),
        _ => {
            eprintln!("Error: unknown modifier {}", modifier);
            eprintln!(
                "Valid <modifier> values: lowercase, uppercase, no-spaces, slugify, reverse, rot13"
            );
            std::process::exit(1);
        }
    };

    println!(
        "Output string with '{}' modifier: {}",
        modifier, modified_text
    );
}
