// client/src/main.rs
use std::{
    env,
    error::Error,
    io::{self, Cursor, Write},
    net::TcpStream,
    process,
};

use log::info;
use image::ImageOutputFormat;
use tracing_subscriber::fmt;

use shared::{MessageType, send_file};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    fmt::init();

    let args: Vec<String> = env::args().collect();

    let (hostname, port) = match args.len() {
        1 => ("localhost".to_string(), 11111),
        3 => (args[1].clone(), args[2].parse().unwrap()),
        _ => {
            println!("Usage: {} [hostname] [port]", args[0]);
            process::exit(1);
        }
    };

    let server_address = format!("{}:{}", hostname, port);
    let mut stream = TcpStream::connect(server_address.clone())?;

    info!("Connected to server on {}", server_address);

    // Read user input and send messages to the server
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let message = match input.trim() {
            ".quit" => MessageType::Quit,
            _ => {
                if input.starts_with(".file") {
                    let path = input.trim_start_matches(".file").trim();
                    send_file(&mut stream, path)?;
                    continue;
                } else if input.starts_with(".image") {
                    let path = input.trim_start_matches(".image").trim();
                    let image_content = read_and_convert_image(path)?;
                    MessageType::Image(image_content)
                } else {
                    MessageType::Text(input.trim().to_string())
                }
            }
        };

        // Serialize and send the message to the server
        let serialized_message = bincode::serialize(&message)?;
        stream.write_all(&serialized_message)?;

        // If the user wants to quit, break the loop
        if let MessageType::Quit = message {
            break;
        }
    }

    Ok(())
}

// Helper function to read and convert image content to PNG format
fn read_and_convert_image(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let image = image::open(path)?;

    // Convert the image to PNG format
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image.write_to(&mut cursor, ImageOutputFormat::Png)?;

    Ok(png_bytes)
}
