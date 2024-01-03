/// Integration tests
#[cfg(test)]
mod integration_tests {
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::sleep;
    use std::time::Duration;
    use std::path::Path;
    use super::Server; // Adjust the import path based on your code structure

    #[tokio::test]
    async fn test_receive_file_integration() {
        // Start a real TcpListener on a random available port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn the server in a separate task
        tokio::spawn(async move {
            // Assuming your server initialization logic is here
            let server = Server::new(); // Adjust as needed

            loop {
                let (stream, _) = listener.accept().await.unwrap();
                // Spawn a separate task to handle the connection
                tokio::spawn(async move {
                    // Handle the connection using your server logic
                    server.handle_client(stream, &mut HashMap::new(), &server.db_pool).await.unwrap();
                });
            }
        });

        // Allow some time for the server to start
        sleep(Duration::from_millis(100)).await;

        // Connect to the server using a real TcpStream
        let stream = TcpStream::connect(addr).await.unwrap();

        // Prepare test data
        let content = b"Test content";
        let filename = "test.txt";
        let directory = "test_dir";

        // Call the receive_file function with the real stream
        let result = Server::receive_file(filename, content, directory, stream).await;

        // Check if the function executed without errors
        assert!(result.is_ok());

        // Check if the file was created
        let file_path = format!("{}/{}_{}", directory, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), filename);
        assert!(Path::new(&file_path).exists());

        // Clean up the created file
        std::fs::remove_file(&file_path).unwrap();
    }
}