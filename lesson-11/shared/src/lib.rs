// shared/lib.rs
use std::error::Error;
use std::io::{self, Read, Write};
use std::net::TcpStream;

use serde_derive::{Deserialize, Serialize};

// Custom Error type for the operations
#[derive(Debug)]
pub struct OperationError(String);

impl std::fmt::Display for OperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation Error: {}", self.0)
    }
}

impl Error for OperationError {}

// Define message types using serde serialization
#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    File(String),
    Image(Vec<u8>),
    Text(String),
    Quit,
}

// Helper function to send a file to the server
pub fn send_file(stream: &mut TcpStream, path: &str) -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    let message = MessageType::File(path.to_string());
    let serialized_message = bincode::serialize(&message)?;
    stream.write_all(&serialized_message)?;

    Ok(())
}

// Helper function to serialize and send a message to the server
pub fn send_message(stream: &mut TcpStream, message: MessageType) -> Result<(), Box<dyn Error>> {
    let serialized_message = bincode::serialize(&message)?;
    stream.write_all(&serialized_message)?;

    Ok(())
}
