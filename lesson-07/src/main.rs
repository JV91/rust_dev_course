use csv::ReaderBuilder;
use flume::{Receiver, Sender};
use slug::slugify;
use std::io::{self, Write};
use std::str::FromStr;
use std::thread::{sleep, spawn};
use std::time::Duration;
use std::{env, error::Error, fmt, fs, iter, process::exit};

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
        // Calculate maximum width for each column
        let max_widths: Vec<usize> = self
            .headers
            .iter()
            .enumerate()
            .map(|(i, header)| {
                iter::once(header.len())
                    .chain(self.rows.iter().map(|row| row[i].len()))
                    .max()
                    .unwrap()
            })
            .collect();

        println!("\nCSV output: \n");

        // Display headers
        write_row(f, &self.headers, &max_widths)?;

        // Display separator line
        write_separator(f, &max_widths)?;

        // Dispaly rows
        for row in &self.rows {
            write_row(f, row, &max_widths)?;
        }

        Ok(())
    }
}

fn write_row(f: &mut fmt::Formatter<'_>, row: &[String], max_widths: &[usize]) -> fmt::Result {
    write!(f, "| ")?;
    for (field, &width) in row.iter().zip(max_widths) {
        write!(f, "{:<width$} | ", field, width = width)?;
    }
    writeln!(f)
}

fn write_separator(f: &mut fmt::Formatter<'_>, max_widths: &[usize]) -> fmt::Result {
    write!(f, "|")?;
    for &width in max_widths {
        write!(f, "{:-<width$}|", "", width = width + 2)?;
    }
    writeln!(f)
}

#[derive(Debug)]
enum Modifier {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    Reverse,
    Rot13,
    Csv,
}

impl FromStr for Modifier {
    type Err = OperationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lowercase" => Ok(Modifier::Lowercase),
            "uppercase" => Ok(Modifier::Uppercase),
            "no-spaces" => Ok(Modifier::NoSpaces),
            "slugify" => Ok(Modifier::Slugify),
            "reverse" => Ok(Modifier::Reverse),
            "rot13" => Ok(Modifier::Rot13),
            "csv" => Ok(Modifier::Csv),
            _ => Err(OperationError(format!("Unknown modifier '{}'", s))),
        }
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
            .delimiter(b';')
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

fn execute_operation(modifier: Modifier, text: &str) -> Result<String, Box<dyn Error>> {
    match modifier {
        Modifier::Lowercase => Ok(TextModifier::apply_lowercase(text)),
        Modifier::Uppercase => Ok(TextModifier::apply_uppercase(text)),
        Modifier::NoSpaces => Ok(TextModifier::remove_spaces(text)),
        Modifier::Slugify => Ok(TextModifier::apply_slugify(text)),
        Modifier::Reverse => Ok(TextModifier::apply_reverse(text)),
        Modifier::Rot13 => Ok(TextModifier::apply_rot13(text)),
        Modifier::Csv => Ok(TextModifier::parse_csv(text)?.to_string()),
    }
}

// MULTI-THREADING
fn interactive_mode(tx: Sender<String>) {
    loop {
        // Wait for 10 millisecs to loop again so that next 'Enter command: ' line isn't shown quicker than response from receiver.
        sleep(Duration::from_millis(10));

        print!("\nEnter command: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        //tx.send(input.trim().to_string()).unwrap();

        if let Err(err) = tx.send(input.trim().to_string()) {
            eprintln!("Error sending message through channel: {}", err);
        }

        /* TO REMEMBER: 
            - io::stdout.flush() method is used on the standard output stream handle. It flushes the internal buffer, ensuring that any data we've written so far is sent to the console.
            - io::stdin() returns a handle to the standard input stream, allowing us to read user input from the console. 
            - .read_line(&mut input) reads a line from the standard input and appends it to the provided String variable input. 
            - .send(input.trim().to_string()) sends the trimmed and converted input string through the channel.
        */
    }
}

fn processing_thread(rx: Receiver<String>) {
    loop {
        let input = rx.recv().unwrap();
        let args: Vec<&str> = input.splitn(2, ' ').collect();

        if args.len() != 2 {
            eprintln!("Invalid input '{}'. Use format: <modifier> <text>.", input);
            continue;
        }

        let modifier_str = args[0];
        let text = args[1].trim();

        // Check if text contains more than one word without single quotes
        if !text.starts_with('\'') && !text.ends_with('\'') && text.split_whitespace().count() > 1 {
            eprintln!("Invalid input '{}'. <text> must contain only one word or be enclosed in single quotes.", input);
            continue;
        }

        // Extract text within single quotes as a single argument
        let text = if text.starts_with('\'') && text.ends_with('\'') {
            &text[1..text.len() - 1]
        } else {
            text
        };

        match modifier_str.parse::<Modifier>() {
            Ok(modifier) => {
                match execute_operation(modifier, text) {
                    Ok(result) => println!("{}", result),
                    Err(err) => eprintln!("{}", err),
                }
            }
            Err(_) => {
                eprintln!("Unknown modifier. Valid modifiers: lowercase, uppercase, no-spaces, slugify, reverse, rot13, csv");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        let (tx, rx) = flume::unbounded();
        // NOTE: use tx.clone() or rx.clone() when interacting with multiple input/output threads.

        spawn(move || interactive_mode(tx));
        spawn(move || processing_thread(rx));

        // Keep the program running after spawning the interactive and processing threads.
        loop {
            sleep(Duration::from_secs(1));
        }
    } else if args.len() == 2 {
        // For this to work, input 'cargo run example.csv' or use your cvs file.
        let filename = &args[1];

        match fs::read_to_string(filename) {
            Ok(content) => match TextModifier::parse_csv(&content) {
                Ok(csv) => println!("{}", csv),
                Err(err) => eprintln!("{}", err),
            },
            Err(err) => {
                eprintln!("Error reading file: {}", err);
            }
        }
    } else {
        eprintln!("Invalid number of arguments. Usage: <modifier> <text>");
        exit(1);
    }
}
