// TODO: this assignment still needs some work on data reading from client - will get to it soon hopefully.

use std::{
    collections::HashMap,
    env,
    error::Error,
    fmt,
    fs::File,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process,
    time::SystemTime,
};

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
    Text(String),
    Image(Vec<u8>),
    File(String, Vec<u8>), // Filename and its content as bytes
    Quit,
}

struct Server {
    address: Option<String>,
}

impl Server {
    fn new(address: Option<String>) -> Self {
        Server { address }
    }

    fn start(&self) -> Result<(), Box<dyn Error>> {
        let listener = match &self.address {
            Some(addr) => TcpListener::bind(addr)?,
            None => TcpListener::bind("0.0.0.0:0")?, // Bind to any IP
        };

        println!("Server listening on {}", listener.local_addr().unwrap());

        let mut clients: HashMap<SocketAddr, TcpStream> = HashMap::new();

        for stream in listener.incoming() {
            let stream = stream?;
            let addr = stream.peer_addr()?;
            clients.insert(addr, stream.try_clone()?);

            self.handle_client(clients.get(&addr).unwrap().try_clone()?, &mut clients);
        }

        Ok(())
    }

    fn handle_client(&self, mut stream: TcpStream, clients: &mut HashMap<SocketAddr, TcpStream>) {
        if let Some(message) = receive_message(&mut stream) {
            // Deserialize received data
            //let message: MessageType = bincode::deserialize(&buffer)?;

            match message {
                MessageType::File(filename, content) => {
                    self.receive_file(&filename, &content, "files/");
                }
                MessageType::Image(content) => {
                    println!("Received image");
                    self.receive_file("received_image", &content, "images/");
                }
                MessageType::Text(text) => {
                    println!("Received text message: {}", text);
                }
                MessageType::Quit => {
                    let _ = clients.remove(&stream.peer_addr().unwrap());
                    println!("Client disconnected");
                }
            }
        } else {
            // Handle the case ehwn receive_message returns None (error reading from the stream)
            println!("Error receiving message from client");
        }
    }

    fn receive_file(&self, filename: &str, content: &[u8], directory: &str) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let filepath = format!("{}{}_{}", directory, timestamp, filename);

        let mut file = File::create(&filepath).unwrap();
        file.write_all(content).unwrap();

        println!("Received file: {}", filepath);
    }
}

fn receive_message(mut stream: &TcpStream) -> Option<MessageType> {
    let mut len_bytes = [0u8; 4];
    if let Err(err) = stream.read_exact(&mut len_bytes) {
        eprintln!("Error reading message length: {}", err);
        return None;
    }
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut buffer = vec![0u8; len];
    if let Err(err) = stream.read_exact(&mut buffer) {
        eprintln!("Error reading message content: {}", err);
        return None;
    }

    match bincode::deserialize(&buffer) {
        Ok(message) => Some(message),
        Err(err) => {
            eprintln!("Error deserializing message: {}", err);
            None
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let address = match args.len() {
        1 => None, // Defaul behaviour - bind to localhost
        2 if args[1] == "0.0.0.0" => Some("0.0.0.0:11111".to_string()), // Bind to any IP
        3 => Some(format!("{}:{}", args[1], args[2])),
        _ => {
            println!("Usage: {} [hostname] [port]", args[0]);
            process::exit(1);
        }
    };

    let server = Server::new(address);
    if let Err(err) = server.start() {
        eprintln!("Server error: {}", err);
    }
}
