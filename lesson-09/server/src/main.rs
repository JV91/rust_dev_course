// TODO: this assignment still needs some work on .file / .image processing - will get to it soon hopefully.

use std::{
    env,
    error::Error,
    fmt,
    fs::{self, File},
    io::{BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    process,
    time::SystemTime,
};

use serde_derive::{Deserialize, Serialize};
//use bincode::{deserialize, serialize};

// Custom Error type for the operations
#[derive(Debug)]
struct OperationError(String);

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operation Error: {}", self.0)
    }
}

impl Error for OperationError {}

////

// Define message types using serde serialization
#[derive(Serialize, Deserialize, Debug)]
enum MessageType {
    File(String),  // Include a String field for the file path
    Image(String), // Include a String field for the image path
    Text(String),  // Include a String field for the text message
    Quit,
}

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

    fn handle_client(&self, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;

        // Deserialize received data
        let message: MessageType = bincode::deserialize(&buffer)?;

        match message {
            MessageType::File(path) => {
                self.receive_file(&stream, &path, "files/")?;
            }
            MessageType::Image(path) => {
                self.receive_file(&stream, &path, "images/")?;
            }
            MessageType::Text(text) => {
                println!("Received text message: {}", text);
            }
            MessageType::Quit => {
                println!("Client requested to quit");
                return Ok(());
            }
        }

        Ok(())
    }

    fn receive_file(
        &self,
        mut stream: &TcpStream,
        _path: &str,
        directory: &str,
    ) -> Result<(), Box<dyn Error>> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();
        let filename = format!("{}{}.png", directory, timestamp);

        let mut file = BufWriter::new(File::create(&filename)?);

        let mut buffer = [0, 254];
        loop {
            let bytes_read = stream.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            file.write_all(&buffer[0..bytes_read])?;
        }

        println!("Received file: {}", filename);

        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <address>", args[0]);
        process::exit(1);
    }

    let address = &args[1];
    let server = Server::new(address);
    if let Err(err) = server.start() {
        eprintln!("Server error: {}", err);
    }
}
