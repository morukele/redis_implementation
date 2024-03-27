use bytes::{BufMut, BytesMut};
// Uncomment this block to pass the first stage
use redis_starter_rust::{Database, RedisParser, RedisValueRef, ThreadPool};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    // Creating a new thread pool
    let pool = ThreadPool::new(4);

    // Creating data store
    let database = Database::new();
    let database = Arc::new(Mutex::new(database));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let store = Arc::clone(&database);
                // Using threads to allow for multiple client support
                pool.execute(|| {
                    handle_client(stream, store);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, store: Arc<Mutex<Database>>) {
    let mut buffer = [0; 1024];
    loop {
        let db = Arc::clone(&store);
        let bytes_read = stream
            .read(&mut buffer)
            .expect("Failed to read input command");

        if bytes_read == 0 {
            return;
        }

        // Parse and return inputs
        let mut bytes = BytesMut::new();
        bytes.put_slice(&buffer);
        let mut parser = RedisParser::new();
        let output = parser
            .decode(&mut bytes)
            .expect("failed to decode encryption")
            .unwrap();

        println!("output: {:?}", &output);
        // Process Commands
        process_command(&output, &mut stream, db);
    }
}

fn process_command(commands: &RedisValueRef, stream: &mut TcpStream, store: Arc<Mutex<Database>>) {
    match commands {
        RedisValueRef::Array(arr) => {
            let start_cmd = &arr[0];
            match start_cmd {
                RedisValueRef::String(cmd) => {
                    match str::from_utf8(cmd)
                        .expect("unable to convert byte to string")
                        .to_lowercase()
                        .as_str()
                    {
                        "ping" => handle_ping(stream),
                        "echo" => handle_echo(stream, &arr[1..]),
                        "get" => handle_get(stream, &arr[1..], store),
                        "set" => handle_set(stream, &arr[1..], store),
                        _ => println!("Unknown command"),
                    }
                }
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
}

fn handle_set(stream: &mut TcpStream, commands: &[RedisValueRef], store: Arc<Mutex<Database>>) {
    if commands.len() == 2 {
        // set without ttl
        match (&commands[0], &commands[1]) {
            (RedisValueRef::String(k), RedisValueRef::String(v)) => {
                let key = str::from_utf8(k).expect("failed to decode buffer");
                let value = str::from_utf8(v).expect("failed to decode buffer");
                let _result = store.lock().unwrap().set(key, value, None);

                // Write the response
                return_ok(stream);
            }
            (_, _) => todo!(),
        }
    }

    if commands.len() == 4 {
        dbg!(commands);
        match (&commands[0], &commands[1], &commands[2], &commands[3]) {
            (
                RedisValueRef::String(k),
                RedisValueRef::String(v),
                RedisValueRef::String(opt),
                RedisValueRef::String(ttl),
            ) => {
                let key = str::from_utf8(k).expect("failed to decode buffer");
                let value = str::from_utf8(v).expect("failed to decode buffer");
                let opt = str::from_utf8(opt).expect("failed to decode buffer");
                let ttl = str::from_utf8(ttl)
                    .unwrap()
                    .parse::<u64>()
                    .expect("failed to decode TTL");
                let ttl = Duration::from_millis(ttl);
                if opt.to_lowercase() == "px" {
                    let _result = store.lock().unwrap().set(key, value, Some(ttl));
                    // Write the response
                    return_ok(stream);
                }
            }
            (_, _, _, _) => todo!(),
        }
    }
}

fn handle_get(stream: &mut TcpStream, commands: &[RedisValueRef], store: Arc<Mutex<Database>>) {
    dbg!(&store.lock().unwrap().store);
    match &commands[0] {
        RedisValueRef::String(k) => {
            let key = str::from_utf8(k).expect("failed to decode buffer");
            let result = store.lock().unwrap().get(key);
            match result {
                Some(set_object) => match set_object.duration {
                    Some(duration) => {
                        // Compute the duration of set object
                        // If not expired, return value else return NIL
                        if Instant::now() > duration {
                            return_null(stream)
                        } else {
                            let response =
                                format!("${}\r\n{}\r\n", set_object.value.len(), set_object.value);
                            write_response(response.as_bytes(), stream);
                        }
                    }
                    None => {
                        let response =
                            format!("${}\r\n{}\r\n", set_object.value.len(), set_object.value);
                        write_response(response.as_bytes(), stream)
                    }
                },
                None => {
                    return_null(stream);
                }
            }
        }
        _ => todo!(),
    }
}

fn handle_ping(stream: &mut TcpStream) {
    write_response(b"+PONG\r\n", stream)
}

fn handle_echo(stream: &mut TcpStream, commands: &[RedisValueRef]) {
    if commands.is_empty() || commands.len() > 1 {
        println!("ERR: wrong number of arguments for echo")
    } else {
        match &commands[0] {
            RedisValueRef::String(s) => {
                let data = str::from_utf8(s).expect("failed to decode buffer");
                let response = format!("${}\r\n{}\r\n", data.len(), data);
                write_response(response.as_bytes(), stream);
            }
            _ => todo!(),
        }
    }
}

fn write_response(response: &[u8], mut stream: &TcpStream) {
    stream
        .write_all(response)
        .expect("failed to write to stream");
}

fn return_null(stream: &mut TcpStream) {
    let response = b"$-1\r\n";
    dbg!(&response);
    write_response(response, stream);
}

fn return_ok(stream: &mut TcpStream) {
    let response = b"+OK\r\n";
    write_response(response, stream);
}
