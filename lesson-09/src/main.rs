// NOTE: this assignment is not yet finished and needs some more work. I will update soon.

use std::error::Error;
use std::fmt;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
//use std::thread;
//use std::time::Duration;

// Custom Error type for the operations
#[derive(Debug)]
struct OperationError(String);

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operation Error: {}", self.0)
    }
}

impl Error for OperationError {}

struct Server {
    address: String,
}

impl Server {
    fn new(address: &str) -> Self {
        Server {
            address: address.to_string(),
        }
    }

    fn start(&self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.address)?;
        println!("Server listening on {}", self.address);

        for stream in listener.incoming() {
            let stream = stream?;
            self.handle_client(stream)?;
        }

        Ok(())
    }

    fn handle_client(&self, _stream: TcpStream) -> Result<(), Box<dyn Error>> {
        // Handle client messages and file transfers here
        // TODO: Implement messaging protocol logic
        Ok(())
    }
}

struct Client {
    address: String,
}

impl Client {
    fn new(address: &str) -> Self {
        Client {
            address: address.to_string(),
        }
    }

    fn send_message(&self, message: &str) -> Result<(), Box<dyn Error>> {
        let mut stream = TcpStream::connect(&self.address)?;

        // Send the message to the server
        stream.write_all(message.as_bytes())?;

        Ok(())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Usage: {} <server/client> <address>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let address = &args[2];

    match mode.as_str() {
        "server" => {
            let server = Server::new(address);
            if let Err(err) = server.start() {
                eprintln!("Server error: {}", err);
            }
        }
        "client" => {
            let client = Client::new(address);
            // Example client message sending
            if let Err(err) = client.send_message("Hello, Server!") {
                eprintln!("Client error: {}", err);
            }
        }
        _ => {
            println!("Invalid mode. Use 'server' or 'client'.");
            std::process::exit(1);
        }
    }
}
