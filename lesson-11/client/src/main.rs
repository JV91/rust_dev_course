// client/src/main.rs

use std::{
    error::Error,
    io::{self, Cursor, Write},
    net::TcpStream,
};

use clap::{App, Arg}; // Clap for command-line argument parsing
use image::ImageOutputFormat; // Image processing library for handling images
use log::info; // Logging with the info level
use tracing_subscriber::fmt; // Tracing subscriber for structured logging

use shared::{send_file, MessageType}; // Shared module with message types and file sending logic

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    fmt::init();

    // Parse command-line arguments using Clap
    let matches = App::new("Client")
        .version("1.0")
        .author("Jan Vais")
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
        (Some(h), Some(p)) => (h, p.parse().unwrap()), // Use provided values
        _ => ("localhost".to_string(), 11111),         // Default values if not provided
    };

    // Build the server address from hostname and port
    let server_address = format!("{}:{}", hostname, port);

    // Connect to the server
    let mut stream = TcpStream::connect(server_address.clone())?;

    // Log the successful connection to the server
    info!("Connected to server on {}", server_address);

    // Read user input and send messages to the server
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Convert user input to a message based on commands or text
        let message = match input.trim() {
            ".quit" => MessageType::Quit, // Quit the application
            _ => {
                if input.starts_with(".file") {
                    // If the input is a file command, extract the path and send the file
                    let path = input.trim_start_matches(".file").trim();
                    send_file(&mut stream, path)?;
                    continue;
                } else if input.starts_with(".image") {
                    // If the input is an image command, extract the path, read, and convert the image
                    let path = input.trim_start_matches(".image").trim();
                    let image_content = read_and_convert_image(path)?;
                    MessageType::Image(image_content)
                } else {
                    // Without special command, treat it as a text message
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
    // Open the image file
    let image = image::open(path)?;

    // Convert the image to PNG format
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image.write_to(&mut cursor, ImageOutputFormat::Png)?;

    Ok(png_bytes)
}
