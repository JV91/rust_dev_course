# Chat Application

This is a simple chat application implemented in Rust, consisting of a server and client components.

## Features

- **Multi-Client Support**: The server can handle multiple client connections concurrently.
- **Message Types**: Supports sending text messages, files, and images between clients.
- **Command-Line Interface**: Both the server and client have a command-line interface for configuration.

### Prerequisites

- Rust programming language. Install it from [rustup.rs](https://rustup.rs/).

### Installation

1. Clone the repository:

    ```bash
    git clone https://github.com/JV91/rust_dev_course/tree/master/lesson-15
    cd lesson-15
    ```

2. Build the server and client:

    ```bash
    cargo build --release
    ```

#### Server

Run the server with the following command:

```bash
cargo run --release --bin server [OPTIONS]
```

### Structure

- **Server (`server` directory)**:
  - `main.rs`: Entry point for the server application & implementation with multi-client support.

- **Client (`client` directory)**:
  - `main.rs`: Entry point for the client application & implementation with multi-client support.

- **Shared (`shared` directory)**:
  - `lib.rs`: Shared functionality of server and client. Custom error handling.

### Dependencies

- `clap`: Command-line argument parsing.
- `log` and `tracing`: Logging and structured logging.
- `image`: Image processing library.
- `serde` and `bincode`: Serialization and deserialization.
