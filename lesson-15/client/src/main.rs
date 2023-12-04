// client/src/main.rs
use std::{
    io::{self, Cursor, Write},
    net::TcpStream,
};

use anyhow::{Context, Result}; // Use anyhow for better error handling
use clap::{App, Arg}; // Clap for command-line argument parsing
use image::ImageOutputFormat; // Image processing library for handling images
use log::info; // Logging with the info level
use tracing_subscriber::fmt; // Tracing subscriber for structured logging

use shared::{send_file, MessageType}; // Shared module with message types and file sending logic

fn main() -> Result<()> {
    // Initialize tracing
    fmt::init();

    // Parse command-line arguments using Clap
    let matches = App::new("Client")
        .version("1.0")
        .author("Your Name")
        .about("Client application for the chat server")
        .arg(
            Arg::with_name("hostname")
                .short("h")
                .long("hostname")
                .value_name("HOST")
                .help("Sets the server hostname")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Sets the server port")
                .takes_value(true),
        )
        .get_matches();

    // Extract hostname and port from CL arguments or use defaults
    let (hostname, port) = match (
        matches.value_of("hostname").map(String::from),
        matches.value_of("port").map(String::from),
    ) {
        (Some(h), Some(p)) => (h, p.parse().context("Invalid port number")?),
        _ => ("localhost".to_string(), 11111),
    };

    // Build the server address from hostname and port
    let server_address = format!("{}:{}", hostname, port);

    // Connect to the server
    let mut stream = TcpStream::connect(server_address.clone())
        .with_context(|| format!("Failed to connect to the server at {}", server_address))?;

    // Log the successful connection to the server
    info!("Connected to server on {}", server_address);

    // Read user input and send messages to the server
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read user input")?;

        // Convert user input to a message based on commands or text
        let message = match input.trim() {
            ".quit" => MessageType::Quit,
            _ => {
                if input.starts_with(".file") {
                    let path = input.trim_start_matches(".file").trim();
                    send_file(&mut stream, path).context("Failed to send file")?;
                    continue;
                } else if input.starts_with(".image") {
                    let path = input.trim_start_matches(".image").trim();
                    let image_content =
                        read_and_convert_image(path).context("Failed to read and convert image")?;
                    MessageType::Image(image_content)
                } else {
                    MessageType::Text(input.trim().to_string())
                }
            }
        };

        // Serialize and send the message to the server
        let serialized_message =
            bincode::serialize(&message).context("Failed to serialize message")?;
        stream
            .write_all(&serialized_message)
            .context("Failed to send message to the server")?;

        // If the user wants to quit, break the loop
        if let MessageType::Quit = message {
            break;
        }
    }

    Ok(())
}

// Helper function to read and convert image content to PNG format
fn read_and_convert_image(path: &str) -> Result<Vec<u8>> {
    let image = image::open(path).with_context(|| format!("Failed to open image at {}", path))?;
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image
        .write_to(&mut cursor, ImageOutputFormat::Png)
        .with_context(|| "Failed to convert image to PNG format")?;
    Ok(png_bytes)
}
