// server/src/main.rs
use std::{
    collections::HashMap, fs::File, io::Write, net::SocketAddr, sync::Arc, time::SystemTime,
};

//use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use anyhow::{Context, Result};
use log::{debug, error, info};
use serde_derive::{Deserialize, Serialize};
use sqlx::{Error as SqlxError, FromRow, PgPool};
use tracing::instrument;
use tokio::{net::TcpListener, net::TcpStream, sync::Mutex};

use shared::{receive_message, MessageType};

/// Structure representing the server application.
#[derive(Debug, Clone)]
struct Server {
    #[allow(dead_code)] // Allowing unused code for the address field for future use
    address: Option<String>,
    db_pool: PgPool,
}

/// Structure representing the database connection.
#[derive(Debug)]
pub struct Database {
    pool: PgPool,
}

/// Structure representing the configuration for the database.
#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConfig {
    database_url: String,
}

/// Structure representing a message entity in the database.
#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Message {
    // Define your fields corresponding to the columns in the "messages" table
    id: i32,
    user: String,
    content: String,
}

impl Server {
    /// Creates a new instance of the server.
    ///
    /// # Arguments
    ///
    /// * `address` - An optional string representing the server address.
    /// * `database` - A `Database` instance representing the database connection.
    ///
    /// # Returns
    ///
    /// A `Server` instance.
    fn new(address: Option<String>, database: Database) -> Self {
        let db_pool = database.pool.clone(); // Assuming Database has a `pool` field
        Server { address, db_pool }
    }

    /// Starts the server and listens for incoming connections.
    ///
    /// # Arguments
    ///
    /// * `bind_address` - An optional string representing the address to bind to.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `anyhow::Error` if an error occurs during the process.
    async fn start(&self, bind_address: Option<&str>) -> Result<(), anyhow::Error> {
        let listener = TcpListener::bind(bind_address.unwrap_or("localhost:11111")).await?;
        println!("Server listening on {:?}", listener.local_addr()?);

        //let database = Arc::new(Mutex::new(Database::new())); // Use Arc<Mutex<Database>> for concurrent access

        let clients: HashMap<SocketAddr, Arc<Mutex<TcpStream>>> = HashMap::new();

        while let Ok(stream) = listener.accept().await {
            let cloned_stream = stream.0;
            let mut clients = clients.clone();
            let db_pool = self.db_pool.clone();

            tokio::spawn(async move {
                if let Err(err) = Server::handle_client(cloned_stream, &mut clients, &db_pool).await
                {
                    println!("Error handling client: {}", err);
                }
            });
        }

        Ok(())
    }

    /// Handles an incoming client connection.
    ///
    /// # Arguments
    ///
    /// * `stream` - A `TcpStream` representing the client connection.
    /// * `clients` - A mutable reference to a `HashMap` containing client connections.
    /// * `db_pool` - A reference to the database pool.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `anyhow::Error` if an error occurs during the process.
    async fn handle_client(
        mut stream: TcpStream,
        clients: &mut HashMap<SocketAddr, Arc<Mutex<TcpStream>>>,
        db_pool: &sqlx::PgPool,
    ) -> Result<(), anyhow::Error> {
        // Attempt to receive a message from the client
        if let Some(message) = receive_message(&mut stream).await {
            // Process the received message based on its type
            match message {
                MessageType::File(ref filename, ref content) => {
                    Server::receive_file(&filename, &content, "../files")?;
                }
                MessageType::Image(ref content) => {
                    info!("Received image");
                    Server::receive_file("received_image", &content, "../images")?;
                }
                MessageType::Text(ref text) => {
                    info!("Received text message: {}", text);
                }
                MessageType::Quit => {
                    // Remove the client from the HashMap on Quit message
                    let _ = clients.remove(&stream.peer_addr().unwrap());
                    info!("Client disconnected");
                }
            }

            debug!("Received message: {:?}", message);
        } else {
            // Log an error if there is an issue receiving the message
            error!("Error receiving message from client");
        }

        // Use the database
        //let mut db = db_pool.acquire().await?;
        Message::save(&db_pool, "example_user", "Hello!").await?;

        Ok(())
    }

    /// Receives a file from the client and saves it to the local filesystem.
    ///
    /// # Arguments
    ///
    /// * `filename` - A string representing the original filename of the received file.
    /// * `content`  - A slice of bytes containing the content of the received file.
    /// * `directory` - A string representing the directory where the file should be saved.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `anyhow::Error` if an error occurs during the process.
    #[instrument]
    fn receive_file(filename: &str, content: &[u8], directory: &str) -> Result<()> {
        // Create a unique filepath based on timestamp and filename
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Failed to calculate timestamp")?
            .as_secs();
        let filepath = format!("{}/{}_{}", directory, timestamp, filename);

        // Write the received file content to a new file
        let mut file =
            File::create(&filepath).context(format!("Failed to create file at {}", filepath))?;
        file.write_all(content)
            .context(format!("Failed to write content to file at {}", filepath))?;

        // Log the received file information
        info!("Received file: {}", filepath);

        Ok(())
    }
}

impl Database {
    /// Creates a new instance of the database with the specified database URL.
    ///
    /// # Arguments
    ///
    /// * `database_url` - A string representing the URL of the PostgreSQL database.
    ///
    /// # Returns
    ///
    /// A `Result` containing the newly created `Database` instance or a `SqlxError` if an error occurs.
    pub async fn new(database_url: &str) -> Result<Self, SqlxError> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Database { pool })
    }

    /// Saves a message to the database.
    ///
    /// # Arguments
    ///
    /// * `user` - A string representing the username associated with the message.
    /// * `message` - A string containing the content of the message.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `SqlxError` if an error occurs during the process.
    pub async fn save_message(&self, user: &str, message: &str) -> Result<(), SqlxError> {
        // Your database interaction logic goes here
        // For simplicity, let's print the user and message for now
        println!("Saving message for user {}: {}", user, message);

        // Placeholder for actual database interaction
        // You might perform SQL queries using self.pool
        // For example: sqlx::query!("INSERT INTO messages (user, content) VALUES ($1, $2)", user, message).execute(&self.pool).await?;

        Ok(())
    }
}

/*
/// Structure representing the configuration for the database.
impl DatabaseConfig {
    fn new(database_url: &str) ->Self {
        DatabaseConfig {
            database_url: database_url.to_string(),
        }
    }
}
*/

impl Message {
    /// Saves a message to the database.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the PostgreSQL database pool.
    /// * `user` - A string representing the username associated with the message.
    /// * `content` - A string containing the content of the message.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `SqlxError` if an error occurs during the process.
    async fn save(db: &sqlx::PgPool, user: &str, content: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO messages (user, content) VALUES ($1, $2)")
            .bind(user)
            .bind(content)
            .execute(db)
            .await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    // Initialize the database pool
    let database_url = "postgresql://username:password@localhost/database_name";
    let database = Database::new(database_url)
        .await
        .expect("Failed to create a database connection");

    // Create the server with the database pool
    let server = Server::new(None, database);

    if let Err(err) = server.start(None).await {
        println!("Server error: {}", err);
    }
}

/// Unit tests
#[cfg(test)]
mod tests {
    use tokio::net::TcpListener;
    use tokio::net::TcpStream;
    use tokio_test::io::Builder;
    use std::io::Cursor;
    use super::Server; // Adjust the import path based on your code structure

    #[tokio::test]
    async fn test_receive_file() {
        // Start a TcpListener to get a TcpStream
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn an async block to simulate the server accepting a connection
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            // You can modify this to perform any additional setup if needed
        });

        // Convert the cursor to a slice
        let content = b"Test content";
        let cursor = Cursor::new(content);
        let cursor_slice = &*cursor.get_ref();

        // Create a mock stream
        let mock_stream = Builder::new().read(cursor_slice).build();

        // Call the receive_file function with the test stream
        let result = Server::receive_file("test.txt", content, "test_dir", mock_stream).await;

        // Check if the function executed without errors
        assert!(result.is_ok());

        // Clean up resources if needed
    }

    /* 
    #[tokio::test]
    async fn test_handle_client() {
        // Create a test TcpStream (a simple in-memory stream)
        let mock_stream = Builder::new().read(cursor).build();
        let stream = TcpStream::from_std(mock_stream, &tokio::runtime::Handle::current()).unwrap();

        // Create an empty HashMap for the clients
        let mut clients = HashMap::new();

        // Create a test database pool
        let database_url = "postgresql://username:password@localhost/test_database";
        let database = Database::new(database_url).await.expect("Failed to create a database connection");

        // Create a test Server instance
        let server = Server::new(None, database);

        // Call the handle_client function with the test stream and clients
        let result = Server::handle_client(stream, &mut clients, &server.db_pool).await;

        // Check if the function executed without errors
        assert!(result.is_ok());
    }
    */
}