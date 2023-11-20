use std::{
  env, 
  process,
  error::Error,
  io::Write,
  net::TcpStream,
};

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
  let args: Vec<String> = env::args().collect();

  if args.len() != 2 {
      println!("Usage: {} <address>", args[0]);
      process::exit(1);
  }

  let address = &args[1];
  let client = Client::new(address);
  // Example client message sending
  if let Err(err) = client.send_message("Hello, Server!") {
      eprintln!("Client error: {}", err);
  }
}