use csv::ReaderBuilder;
use slug::slugify;
use std::{env, error::Error, fmt, process::exit};

// Custom Error type for the operations
#[derive(Debug)]
struct OperationError(String);

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operation Error: {}", self.0)
    }
}

impl Error for OperationError {}

// Csv struct to store headers and rows
struct Csv {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

// Implementing the Display trait for Csv from: https://doc.rust-lang.org/std/fmt/trait.Display.html#examples
impl fmt::Display for Csv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display headers
        writeln!(f, "| {} |", self.headers.join(" | "))?;
        // Display separator line
        writeln!(f, "|{}|", "-".repeat(self.headers.len() * 3 - 1))?;
        // Display rows
        for row in &self.rows {
            writeln!(f, "| {} |", row.join(" | "))?;
        }
        Ok(())
    }
}

struct TextModifier;

impl TextModifier {
    pub fn apply_lowercase(input: &str) -> String {
        input.to_lowercase()
    }

    pub fn apply_uppercase(input: &str) -> String {
        input.to_uppercase()
    }

    pub fn remove_spaces(input: &str) -> String {
        input.replace(' ', "")
    }

    pub fn apply_slugify(input: &str) -> String {
        slugify(input)
    }

    pub fn apply_reverse(input: &str) -> String {
        input.chars().rev().collect()
    }

    pub fn apply_rot13(input: &str) -> String {
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

    pub fn parse_csv(input: &str) -> Result<Csv, Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .has_headers(false) // default value is true and then we miss the first row (headers)
            .from_reader(input.as_bytes());
        let records = reader.records().collect::<Result<Vec<_>, _>>()?;

        let headers: Vec<String> = records
            .get(0)
            .ok_or_else(|| "CSV must have at least one row".to_string())?
            .iter()
            .map(|field| field.to_string())
            .collect();

        let rows: Vec<Vec<String>> = records[1..]
            .iter()
            .map(|record| record.iter().map(|field| field.to_string()).collect())
            .collect();

        Ok(Csv { headers, rows })
    }
}

fn execute_operation(args: &[String]) -> Result<String, Box<dyn Error>> {
    if args.len() != 3 {
        return Err(Box::new(OperationError(
            "Invalid number of arguments".to_string(),
        )));
    }

    let modifier = &args[1];
    let text = &args[2];

    match modifier.as_str() {
        "lowercase" => Ok(TextModifier::apply_lowercase(text)),
        "uppercase" => Ok(TextModifier::apply_uppercase(text)),
        "no-spaces" => Ok(TextModifier::remove_spaces(text)),
        "slugify" => Ok(TextModifier::apply_slugify(text)),
        "reverse" => Ok(TextModifier::apply_reverse(text)),
        "rot13" => Ok(TextModifier::apply_rot13(text)),
        "csv" => Ok(TextModifier::parse_csv(text)?.to_string()),
        _ => Err(Box::new(OperationError(format!(
            "Unknown modifier '{}'. Valid modifiers: lowercase, uppercase, no-spaces, slugify, reverse, rot13, csv",
            modifier
        )))),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match execute_operation(&args) {
        Ok(result) => println!("{}", result),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
