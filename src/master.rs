use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
};

use crate::{handle_client, Database, Server};

pub async fn run_master(server_info: Server) {
    //Setting up master server
    let listener =
        TcpListener::bind(format!("{}:{}", &server_info.host, &server_info.port)).unwrap();

    // Preparing for multithreading
    let database = Arc::new(Mutex::new(Database::new()));
    let server_info = Arc::new(Mutex::new(server_info));

    // Processing the stream
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let store = Arc::clone(&database);
                let server_info = Arc::clone(&server_info);
                tokio::spawn(async move { handle_client(stream, store, server_info).await });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
