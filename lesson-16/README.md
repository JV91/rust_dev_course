# Chat Application

This is a simple chat application implemented in Rust, consisting of a server and client components.

## Aim of this release

- **Doc-comments**: Adding document comments to key functions and modules in client, server and shared code with clear and concise descriptions.
- **Basic testing**: Writing a couple of tests for parts of the application. Including some unit tests and an integration test in applicable.

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

### Author notes & comments
I learned to do proper document comments for modules and functions. I wrote couple of unit tests and integration test. I had some troubles compiling these tests so I will continue with testing on future application releases. 