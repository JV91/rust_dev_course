// shared/lib.rs
use std::{
    error::Error,
    time::SystemTime,
};

use anyhow::{Context, Result};
use log::{error, info}; // Added logging
use serde_derive::{Deserialize, Serialize}; // Added anyhow
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// # Message Types
///
/// This module defines an enumeration `MessageType` representing various types of messages that
/// can be exchanged between the client and server. These include messages for sending files,
/// images, plain text, and a Quit signal.
#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    File(String, Vec<u8>),
    Image(Vec<u8>),
    Text(String),
    Quit,
}

/// # Send File
///
/// This asynchronous function sends a file to the server over a TCP stream. The file is specified
/// by its path, and the function reads the file content and sends it with the filename as a
/// `MessageType::File` variant.
///
/// # Arguments
///
/// * `stream` - A mutable reference to a `TcpStream` representing the communication channel with
///              the server.
/// * `path`   - A string slice representing the path to the file to be sent.
///
/// # Returns
///
/// A `Result` indicating success or an `anyhow::Error` if an error occurs during the process.
pub async fn send_file(stream: &mut TcpStream, path: &str) -> Result<(), anyhow::Error> {
    let mut file = tokio::fs::File::open(path)
        .await
        .with_context(|| format!("Failed to open file: {}", path))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .await
        .with_context(|| format!("Failed to read file: {}", path))?;

    let message = MessageType::File(path.to_string(), content);
    let serialized_message = bincode::serialize(&message)
        .with_context(|| format!("Failed to serialize message: {:?}", message))?;

    stream
        .write_all(&serialized_message)
        .await
        .with_context(|| format!("Failed to send file: {}", path))?;

    Ok(())
}

/// # Receive Message
///
/// This asynchronous function receives a message from the server over a TCP stream. It first reads
/// the length of the message, then reads the message content, deserializes it using `bincode`, and
/// returns the deserialized `MessageType`.
///
/// # Arguments
///
/// * `stream` - A mutable reference to a `TcpStream` representing the communication channel with
///              the server.
///
/// # Returns
///
/// An `Option` containing the deserialized `MessageType` if successful, or `None` if an error
/// occurs during the process.
pub async fn receive_message(stream: &mut TcpStream) -> Option<MessageType> {
    let mut len_bytes = [0u8; 4];

    if let Err(err) = stream.read_exact(&mut len_bytes).await {
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

    if let Err(err) = stream.read_exact(&mut buffer).await {
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

/// # Receive File
///
/// This function receives a file from the server and saves it to the local filesystem. The
/// filename is combined with a timestamp to ensure uniqueness.
///
/// # Arguments
///
/// * `filename`  - A string slice representing the original filename of the received file.
/// * `content`   - A slice of bytes containing the content of the received file.
/// * `directory` - A string representing the directory where the file should be saved.
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

/// # Log Error
///
/// This function logs an error message using the `log` crate.
fn log_error<E: Error>(error: E) {
    error!("Error: {}", error);
}

/// # Log Information
///
/// This function logs an informational message using the `log` crate.
fn log_info(message: &str) {
    info!("{}", message);
}
