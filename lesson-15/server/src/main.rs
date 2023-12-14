// server/src/main.rs
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    time::SystemTime,
    sync::Arc,
};

use anyhow::{Context, Result};
use log::info;
use tracing::instrument;

use tokio::sync::Mutex;
use async_std::net::TcpListener;
use async_std::stream::StreamExt;
use async_std::task;
use serde_derive::{Deserialize, Serialize};

use shared::{receive_message, MessageType};

#[derive(Debug)]
struct Server {
    #[allow(dead_code)] // Allowing unused code for the address field for future use
    address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Database {
    // ... your database fields
}

impl Server {
    fn new(address: Option<String>) -> Self {
        Server { address }
    }

    async fn start(&self, bind_address: Option<&str>) -> Result<(), anyhow::Error> {
        let listener = TcpListener::bind(bind_address.unwrap_or("localhost:11111")).await?;
        println!("Server listening on {:?}", listener.local_addr()?);

        let database = Arc::new(Mutex::new(Database::new())); // Use Arc<Mutex<Database>> for concurrent access

        let clients: HashMap<_, _> = HashMap::new();

        while let Some(stream) = listener.incoming().next().await {
            let stream = stream?;
            let _addr = stream.peer_addr()?;
            let cloned_stream = stream.clone();
            let mut clients = clients.clone();
            let database = database.clone();

            task::spawn(async move {
                if let Err(err) = Self::handle_client(cloned_stream, &mut clients, &database).await {
                    println!("Error handling client: {}", err);
                }
            });
        }

        Ok(())
    }

    async fn handle_client(
        stream: async_std::net::TcpStream,
        clients: &mut HashMap<async_std::net::SocketAddr, async_std::net::TcpStream>,
        database: &Mutex<Database>,
    ) -> Result<(), anyhow::Error> {
        // Attempt to receive a message from the client
        if let Some(message) = receive_message(&mut stream) {
            // Process the received message based on its type
            match message {
                MessageType::File(ref filename, ref content) => {
                    self.receive_file(&filename, &content, "../files/")?;
                }
                MessageType::Image(ref content) => {
                    info!("Received image");
                    self.receive_file("received_image", &content, "../images/")?;
                }
                MessageType::Text(ref text) => {
                    info!("Received text message: {}", text);
                }
                MessageType::Quit => {
                    // Remove the client from the HashMap on Quit message
                    let _ = clients.remove(&stream.peer_addr().unwrap());
                    info!("Client disconnected");
                }
            }

            debug!("Received message: {:?}", message);
        } else {
            // Log an error if there is an issue receiving the message
            error!("Error receiving message from client");
        }

        // Use the database
        let mut db = database.lock().await;
        db.save_message("example_user", "Hello, world!");

        Ok(())
    }

    #[instrument]
    fn receive_file(&self, filename: &str, content: &[u8], directory: &str) -> Result<()> {
        // Create a unique filepath based on timestamp and filename
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Failed to calculate timestamp")?
            .as_secs();
        let filepath = format!("{}{}_{}", directory, timestamp, filename);

        // Write the received file content to a new file
        let mut file =
            File::create(&filepath).context(format!("Failed to create file at {}", filepath))?;
        file.write_all(content)
            .context(format!("Failed to write content to file at {}", filepath))?;

        // Log the received file information
        info!("Received file: {}", filepath);

        Ok(())
    }
}

impl Database {
    fn new() -> Self {
        Database {
            // ... initialize your database
        }
    }

    fn save_message(&mut self, user: &str, message: &str) {
        // ... save the message to the database
    }
}

#[tokio::main]
async fn main() {
    let server = Server::new(None);
    if let Err(err) = server.start(None).await {
        println!("Server error: {}", err);
    }
}
