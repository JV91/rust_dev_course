use std::{
  env,
  error::Error,
  fmt,
  fs::File,
  io::{self, BufRead, BufReader, Write},
  net::TcpStream,
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
  File(String),
  Image(String),
  Text(String),
  Quit,
}

fn main() -> Result<(), Box<dyn Error>> {
  let args: Vec<String> = env::args().collect();

  if args.len() != 3 {
      println!("Usage: {} <hostname> <port>", args[0]);
      process::exit(1);
  }

  let hostname = &args[1];
  let port: u16 = args[2].parse()?;

  let server_address = format!("{}:{}", hostname, port);
  let mut stream = TcpStream::connect(server_address.clone())?;

  println!("Connected to server on {}", server_address);

  // Read user input and send messages to the server
  loop {
      let mut input = String::new();
      io::stdin().read_line(&mut input)?;

      let message = match input.trim() {
          ".quit" => MessageType::Quit,
          _ => {
              if input.starts_with(".file") {
                  let path = input.trim_start_matches(".file").trim();
                  MessageType::File(path.to_string())
              } else if input.starts_with(".image") {
                  let path = input.trim_start_matches(".image").trim();
                  println!("path: {}", &path);
                  MessageType::Image(path.to_string())
              } else {
                  MessageType::Text(input.trim().to_string())
              }
          }
      };

      // Serialize and send the message to the server
      let serialized_message = bincode::serialize(&message)?;
      println!("serialized_message: {:?}", &serialized_message);
      stream.write_all(&serialized_message)?;

      // If the user wants to quit, break the loop
      if let MessageType::Quit = message {
          break;
      }
  }

  Ok(())
}
