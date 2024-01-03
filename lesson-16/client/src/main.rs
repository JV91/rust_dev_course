// client/src/main.rs

use std::io;

use anyhow::{Context, Result}; // Use anyhow for better error handling
use clap::{App, Arg}; // Clap for command-line argument parsing
use tokio::io::{self as tokio_io, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader}; // tokio for async programming
use tokio::net::TcpStream;
use tokio::task;

use shared::MessageType; // Shared module with message types and file sending logic

/// # Client Main Module
///
/// This module contains the main entry point for the client application. It handles command-line
/// argument parsing, establishes a connection to the server, and manages user input to send messages.
///
/// ## Examples
///
/// ```
/// // Run the client with default settings
/// cargo run
///
/// // Specify a custom server hostname and port
/// cargo run -- --hostname hostexample --port 12345
/// ```

/// # Async Helper Function to Send a Message
///
/// This function serializes and sends a message to the server over the provided TcpStream.
/// It returns a Result indicating success or failure, with an `anyhow::Error` providing
/// additional context in case of failure.
///
/// # Arguments
///
/// * `stream` - A mutable reference to a TcpStream representing the connection to the server.
/// * `message` - The message to be sent to the server, encapsulated in the `MessageType` enum.
///
/// # Example
///
/// ```rust
/// use shared::MessageType;
/// use tokio::net::TcpStream;
///
/// let mut stream = TcpStream::connect("localhost:8080").await.unwrap();
/// let message = MessageType::Text("Hello, server!".to_string());
/// let result = send_message(&mut stream, &message).await;
/// assert!(result.is_ok());
/// ```
pub async fn send_message(
    stream: &mut TcpStream,
    message: &MessageType,
) -> Result<(), anyhow::Error> {
    let serialized_message = bincode::serialize(&message)
        .with_context(|| format!("Failed to serialize message: {:?}", message))?;

    stream
        .write_all(&serialized_message)
        .await
        .with_context(|| format!("Failed to send message: {:?}", message))?;

    Ok(())
}

// Helper function to read and convert image content to PNG format
/// # Read and Convert Image
///
/// This asynchronous function reads an image file from the specified path, converts it to the PNG
/// format, and returns the resulting bytes as a `Vec<u8>`. The function uses Tokio's `spawn_blocking`
/// to perform blocking operations, such as opening the image file, without blocking the Tokio runtime.
///
/// # Arguments
///
/// * `path` - A string slice representing the path to the image file.
///
/// # Returns
///
/// A `Result` containing the PNG-encoded image bytes if successful, or an `anyhow::Error` in case
/// of failure.
///
/// # Example
///
/// ```rust
/// use anyhow::Result;
/// use client::read_and_convert_image;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let path = "path/to/image.jpg";
///     let png_bytes = read_and_convert_image(path).await?;
///     println!("Image converted to PNG with {} bytes", png_bytes.len());
///     Ok(())
/// }
/// ```
async fn read_and_convert_image(path: &str) -> Result<Vec<u8>> {
    let path_clone = path.to_owned(); // Clone path before moving into closure

    let image_result = task::spawn_blocking(move || {
        image::open(&path_clone).with_context(|| format!("Failed to open image at {}", &path_clone))
    })
    .await?;

    let image = image_result?;

    let mut png_bytes = Vec::new();
    let mut cursor = io::Cursor::new(&mut png_bytes);

    image
        .write_to(&mut cursor, image::ImageOutputFormat::Png)
        .with_context(|| "Failed to convert image to PNG format")?;

    Ok(png_bytes)
}

/// # Main Function
///
/// The main entry point for the client application. It parses command-line arguments,
/// establishes a connection to the server, and manages user input to send messages.
///
/// ## Example
///
/// ```rust
/// // Run the client with default settings
/// cargo run
///
/// // Specify a custom server hostname and port
/// cargo run -- --hostname hostexample.com --port 12345
/// ```
#[tokio::main]
async fn main() -> Result<()> {
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
        .await
        .with_context(|| format!("Failed to connect to the server at {}", server_address))?;

    // Read user input and send messages to the server
    loop {
        let mut input = String::new();
        tokio_io::stdout().flush().await?;
        BufReader::new(tokio_io::stdin())
            .read_line(&mut input)
            .await?;
        let input = input.trim();

        // Convert user input to a message based on commands or text
        let message = match input {
            ".quit" => MessageType::Quit,
            _ => {
                if input.starts_with(".file") {
                    let path = input.trim_start_matches(".file").trim();

                    let mut file = tokio::fs::File::open(path)
                        .await
                        .with_context(|| format!("Failed to open file: {}", path))?;

                    let mut file_content = Vec::new();
                    file.read_to_end(&mut file_content)
                        .await
                        .with_context(|| format!("Failed to read file: {}", path))?;

                    MessageType::File(path.to_string(), file_content)
                } else if input.starts_with(".image") {
                    let path = input.trim_start_matches(".image").trim();
                    let image_content = read_and_convert_image(path)
                        .await
                        .context("Failed to read and convert image")?;
                    MessageType::Image(image_content)
                } else {
                    MessageType::Text(input.to_string())
                }
            }
        };

        // Serialize and send the message to the server
        send_message(&mut stream, &message).await?;

        // If the user wants to quit, break the loop
        if let MessageType::Quit = message {
            break;
        }
    }

    Ok(())
}
