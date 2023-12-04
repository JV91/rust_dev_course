// shared/lib.rs
use std::{
    error::Error,
    io::{Read, Write},
    net::TcpStream,
    time::SystemTime,
};

use anyhow::{Context, Result};
use log::{error, info}; // Added logging
use serde_derive::{Deserialize, Serialize}; // Added anyhow

// Define message types using serde serialization
#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    File(String, Vec<u8>),
    Image(Vec<u8>),
    Text(String),
    Quit,
}

// Helper function to send a file to the server
pub fn send_file(stream: &mut TcpStream, path: &str) -> Result<(), anyhow::Error> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("Failed to open file: {}", path))?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    let message = MessageType::File(path.to_string(), content);
    let serialized_message = bincode::serialize(&message)
        .with_context(|| format!("Failed to serialize message: {:?}", message))?;
    stream
        .write_all(&serialized_message)
        .with_context(|| format!("Failed to send file: {}", path))?;

    Ok(())
}

// Helper function to serialize and send a message to the server
pub fn send_message(stream: &mut TcpStream, message: MessageType) -> Result<(), anyhow::Error> {
    let serialized_message = bincode::serialize(&message)
        .with_context(|| format!("Failed to serialize message: {:?}", message))?;
    stream
        .write_all(&serialized_message)
        .with_context(|| format!("Failed to send message: {:?}", message))?;

    Ok(())
}

// Helper function to receive and deserialize a message
pub fn receive_message(mut stream: &TcpStream) -> Option<MessageType> {
    let mut len_bytes = [0u8; 4];
    if let Err(err) = stream.read_exact(&mut len_bytes) {
        log_error(err);
        return None;
    }
    let len = u32::from_be_bytes(len_bytes) as usize;

    log_info(&format!("Received message length: {}", len));

    if len == 0 {
        log_info("Empty message received");
        return None;
    }

    let mut buffer = vec![0u8; len];
    if let Err(err) = stream.read_exact(&mut buffer) {
        log_error(err);
        return None;
    }

    match bincode::deserialize(&buffer) {
        Ok(message) => {
            log_info(&format!("Received message: {:?}", message));
            Some(message)
        }
        Err(err) => {
            log_error(err);
            None
        }
    }
}

// Helper function to receive and save a file
pub fn receive_file(filename: &str, content: &[u8], directory: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filepath = format!("{}{}_{}", directory, timestamp, filename);

    if let Err(err) = std::fs::write(&filepath, content) {
        log_error(err);
        return;
    }

    log_info(&format!("Received file: {}", filepath));
}

// Helper function to log errors
fn log_error<E: Error>(error: E) {
    error!("Error: {}", error);
}

// Helper function to log information
fn log_info(message: &str) {
    info!("{}", message);
}
