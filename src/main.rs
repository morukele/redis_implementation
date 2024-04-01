use clap::Parser;
use redis_starter_rust::{handle_client, Database, Mode, Server};
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
    // Getting server config
    let config = Config::parse();

    // Determining and set server mode
    let host = "127.0.0.1".to_string();
    let master_replid = "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string();
    let server_info = match config.replicaof {
        Some(info) => {
            dbg!(info);
            todo!()
        }
        None => Server::new(host, config.port, Mode::Master, Some(master_replid), 0),
    };

    //Setting up server
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
