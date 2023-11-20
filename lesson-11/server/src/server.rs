use std::{
  env, 
  process, 
  fmt, 
  error::Error,
  net::{TcpListener, TcpStream},
};

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