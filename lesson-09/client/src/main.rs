use std::{
    env,
    error::Error,
    fmt,
    fs::File,
    io::{self, Cursor, Read, Write},
    net::TcpStream,
    process,
};

use image::ImageOutputFormat;
use serde_derive::{Deserialize, Serialize};

// Custom Error type for the operations
#[derive(Debug)]
struct OperationError(String);

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operation Error: {}", self.0)
    }
}

impl Error for OperationError {}

// Define message types using serde serialization
#[derive(Serialize, Deserialize, Debug)]
enum MessageType {
    File(String),
    Image(Vec<u8>),
    Text(String),
    Quit,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let (hostname, port) = match args.len() {
        1 => ("localhost".to_string(), 11111), // Defaul values
        3 => (args[1].clone(), args[2].parse().unwrap()),
        _ => {
            println!("Usage: {} [hostname] [port]", args[0]);
            process::exit(1);
        }
    };

    let server_address = format!("{}:{}", hostname, port);
    let mut stream = TcpStream::connect(server_address.clone())?;

    println!("Connected to server on {}", server_address);

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
        // DEBUG:
        // println!("serialized_message: {:?}", &serialized_message);
        //
        stream.write_all(&serialized_message)?;

        // DEBUG info
        if let Err(err) = stream.write_all(&serialized_message) {
            eprintln!("Error sending message: {}", err);
        } else {
            println!("Message successfully sent to the server");
        }

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

// Helper function to send a file to the server
fn send_file(stream: &mut TcpStream, path: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    let message = MessageType::File(path.to_string());
    let serialized_message = bincode::serialize(&message)?;
    stream.write_all(&serialized_message)?;
    // DEBUG:
    // Print success message to the command line
    println!("File '{}' successfully sent to the server", path);
    //

    Ok(())
}
