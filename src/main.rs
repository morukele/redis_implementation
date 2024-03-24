// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_client(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let bytes_read = stream
        .read(&mut buffer)
        .expect("Failed to read input command");

    println!("received {} bytes", bytes_read);
    println!("{:?}", String::from_utf8_lossy(&buffer[..bytes_read]));

    stream
        .write_all(b"+PONG\r\n")
        .expect("failed to write to stream");
}
