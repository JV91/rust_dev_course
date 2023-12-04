// server/src/main.rs
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::SystemTime,
};

use anyhow::{Context, Result};
use log::{error, info};
use tracing::{debug, instrument};
use tracing_subscriber::fmt;

use shared::{receive_message, MessageType};

#[derive(Debug)]
struct Server {
    #[allow(dead_code)] // Allowing unused code for the address field for future use
    address: Option<String>,
}

impl Server {
    // Constructor to create a new server instance
    fn new(address: Option<String>) -> Self {
        Server { address }
    }

    #[instrument]
    fn start(&self, bind_address: Option<&str>) -> Result<(), anyhow::Error> {
        // Initialize tracing
        fmt::init();

        // Create a TcpListener based on the provided or default bind_address
        let listener = match bind_address {
            Some(addr) if addr == "0.0.0.0" => TcpListener::bind("0.0.0.0:11111")?, // Allow connections from any IP
            Some(addr) => TcpListener::bind(addr)?,
            None => TcpListener::bind("localhost:11111")?, // Default to localhost:11111
        };

        // Log the address the server is listening on
        info!("Server listening on {:?}", listener.local_addr()?);

        // HashMap to store connected clients
        let mut clients: HashMap<SocketAddr, TcpStream> = HashMap::new();

        // Main loop for handling incoming connections
        for stream in listener.incoming() {
            let stream = stream?;
            let addr = stream.peer_addr()?;
            clients.insert(addr, stream.try_clone()?);

            // Handle messages from the connected client
            if let Err(err) =
                self.handle_client(clients.get(&addr).unwrap().try_clone()?, &mut clients)
            {
                error!("Error handling client: {}", err);
            }
        }

        Ok(())
    }

    #[instrument]
    fn handle_client(
        &self,
        mut stream: TcpStream,
        clients: &mut HashMap<SocketAddr, TcpStream>,
    ) -> Result<()> {
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

fn main() {
    // Collect CL arguments
    let args: Vec<String> = env::args().collect();

    // Create a new Server instance with no specified address
    let server = Server::new(None);

    // Start the server with the provided or default bind_address
    if let Err(err) = server.start(args.get(1).map(|s| s.as_str())) {
        // Log an error if there is an issue starting the server
        error!("Server error: {}", err);
    }
}
