use clap::Parser;
use redis_starter_rust::{handle_client, Database};
use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
};

#[derive(Parser, Debug, Clone)]
struct Config {
    #[arg(short, long, default_value_t = 6379)]
    port: usize,
    #[arg(short, long, number_of_values = 2, require_equals = false)]
    replicaof: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Configuring server
    let config = Config::parse();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", &config.port)).unwrap();
    let mut database = Database::new();

    // Determining and set server mode
    if let Some(info) = config.replicaof {
        database.toggle_slave_mode(info);
    };
    let database = Arc::new(Mutex::new(database));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let store = Arc::clone(&database);
                tokio::spawn(async move { handle_client(stream, store).await });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
