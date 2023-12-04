// server/src/main.rs
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    time::SystemTime,
};

use clap::{App, Arg};
use log::{info, error};
use tracing::{debug, instrument};
use tracing_subscriber::fmt;

use shared::{MessageType, receive_message};

#[derive(Debug)]
struct Server {
    address: Option<String>,
}

impl Server {
    fn new(address: Option<String>) -> Self {
        Server { address }
    }

    #[instrument]
    fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize tracing
        fmt::init();

        let listener = match &self.address {
            Some(addr) => TcpListener::bind(addr)?,
            None => TcpListener::bind("0.0.0.0:0")?, // Bind to any IP
        };

        info!("Server listening on {}", listener.local_addr().unwrap());

        let mut clients: HashMap<SocketAddr, TcpStream> = HashMap::new();

        for stream in listener.incoming() {
            let stream = stream?;
            let addr = stream.peer_addr()?;
            clients.insert(addr, stream.try_clone()?);

            self.handle_client(clients.get(&addr).unwrap().try_clone()?, &mut clients);
        }

        Ok(())
    }

    #[instrument]
    fn handle_client(&self, mut stream: TcpStream, clients: &mut HashMap<SocketAddr, TcpStream>) {
        if let Some(message) = receive_message(&mut stream) {
            match message {
                MessageType::File(ref filename, ref content) => {
                    self.receive_file(&filename, &content, "../files/");
                }
                MessageType::Image(ref content) => {
                    info!("Received image");
                    self.receive_file("received_image", &content, "../images/");
                }
                MessageType::Text(ref text) => {
                    info!("Received text message: {}", text);
                }
                MessageType::Quit => {
                    let _ = clients.remove(&stream.peer_addr().unwrap());
                    info!("Client disconnected");
                }
            }

            debug!("Received message: {:?}", message);
        } else {
            error!("Error receiving message from client");
        }
    }

    #[instrument]
    fn receive_file(&self, filename: &str, content: &[u8], directory: &str) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let filepath = format!("{}{}_{}", directory, timestamp, filename);

        let mut file = File::create(&filepath).unwrap();
        file.write_all(content).unwrap();

        info!("Received file: {}", filepath);
    }
}

fn main() {
    // Parse command-line arguments using Clap
    let matches = App::new("Server")
        .version("1.0")
        .author("Your Name")
        .about("Server application for the chat server")
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .value_name("ADDRESS")
                .help("Sets the server address")
                .takes_value(true),
        )
        .get_matches();

    let address = matches.value_of("address").map(String::from);

    let server = Server::new(address);
    if let Err(err) = server.start() {
        error!("Server error: {}", err);
    }
}
