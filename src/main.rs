// Uncomment this block to pass the first stage
use redis_starter_rust::{RedisParser, ThreadPool};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    // Creating a new thread pool
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                // Using threads to allow for multiple client support
                pool.execute(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .expect("Failed to read input command");

        if bytes_read == 0 {
            return;
        }

        // Parse and return inputs
        let parser = RedisParser::new();
        let output = parser.parse(&buffer, bytes_read);

        // Process Commands
        process_command(&output, &mut stream);
    }
}

fn process_command(commands: &[String], stream: &mut TcpStream) {
    match commands[0].as_str() {
        "echo" => handle_echo(stream, &commands[1..]),
        "ping" => handle_ping(stream, &commands[1..]),
        _ => {
            println!("Error: Unknown command")
        }
    }
}

fn handle_ping(stream: &mut TcpStream, commands: &[String]) {
    if !commands.is_empty() {
        println!("ERR: ping has too many arguments");
    } else {
        write_response(b"+PONG\r\n", stream)
    }
}

fn handle_echo(stream: &mut TcpStream, commands: &[String]) {
    if commands.is_empty() {
        println!("ERR: wrong number of arguments for echo")
    } else {
        write_response(
            format!("${}\r\n{}\r\n", commands[0].len(), commands[0]).as_bytes(),
            stream,
        )
    }
}

fn write_response(response: &[u8], mut stream: &TcpStream) {
    println!("response to write: {:?}", response);
    stream
        .write_all(response)
        .expect("failed to write to stream");
}
