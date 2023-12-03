// TODO: this assignment still needs some work on data reading from client - in progress.

use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::File,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process,
    time::SystemTime,
};

// use serde_derive::{Deserialize, Serialize};
use shared::{MessageType};

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

            //println!("clients: {:?}", &clients);

            match message {
                MessageType::File(ref filename, ref content) => {
                    self.receive_file(&filename, &content, "../files/");
                }
                MessageType::Image(ref content) => {
                    println!("Received image");
                    self.receive_file("received_image", &content, "../images/");
                }
                MessageType::Text(ref text) => {
                    println!("Received text message: {}", text);
                }
                MessageType::Quit => {
                    let _ = clients.remove(&stream.peer_addr().unwrap());
                    println!("Client disconnected");
                }
            }

            println!("Received message: {:?}", message);
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

    println!("Received message length: {}", len); // Debug

    if len == 0 {
        println!("Empty message received"); // Debug
        return None;
    }

    let mut buffer = vec![0u8; len];
    if let Err(err) = stream.read_exact(&mut buffer) {
        eprintln!("Error reading message content: {}", err);
        return None;
    }

    match bincode::deserialize(&buffer) {
        Ok(message) => {
            println!("Received message: {:?}", message); // Debug
            Some(message)
        }
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
