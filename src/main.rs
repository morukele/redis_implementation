use bytes::{BufMut, BytesMut};
use clap::Parser;
use redis_starter_rust::{
    return_bulk_string, return_null, return_ok, write_response, Database, RedisParser,
    RedisValueRef, ThreadPool,
};
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    str,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
struct Config {
    #[arg(short, long, default_value_t = 6379)]
    port: usize,
    #[arg(short, long, number_of_values = 2, require_equals = false)]
    replicaof: Option<Vec<String>>,
}

enum Mode {
    Master,
    Slave(Vec<String>),
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Getting the args
    let config = Config::parse();

    // Determining server mode
    let mode = match config.replicaof {
        Some(value) => Mode::Slave(value),
        None => Mode::Master,
    };
    let mode = Arc::new(Mutex::new(mode));

    let listener = TcpListener::bind(format!("127.0.0.1:{}", &config.port)).unwrap();

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
                let mode = Arc::clone(&mode);
                // Using threads to allow for multiple client support
                pool.execute(|| {
                    handle_client(stream, store, mode);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, store: Arc<Mutex<Database>>, mode: Arc<Mutex<Mode>>) {
    let mut buffer = [0; 1024];
    loop {
        let db = Arc::clone(&store);
        let mode = Arc::clone(&mode);
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
        process_command(&output, &mut stream, db, mode);
    }
}

fn process_command(
    commands: &RedisValueRef,
    stream: &mut TcpStream,
    store: Arc<Mutex<Database>>,
    mode: Arc<Mutex<Mode>>,
) {
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
                        "info" => handle_info(stream, &arr[1..], mode),
                        _ => println!("Unknown command"),
                    }
                }
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
}

fn handle_info(stream: &mut TcpStream, commands: &[RedisValueRef], mode: Arc<Mutex<Mode>>) {
    let mode = mode.lock().unwrap();
    match &*mode {
        Mode::Master => {
            let value = String::from("role:master");
            return_bulk_string(value, stream);
        }
        Mode::Slave(info) => {
            dbg!(info);
            let value = String::from("role:slave");
            return_bulk_string(value, stream);
        }
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
                            return_bulk_string(set_object.value, stream);
                        }
                    }
                    None => return_bulk_string(set_object.value, stream),
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
                return_bulk_string(data.to_string(), stream)
            }
            _ => todo!(),
        }
    }
}
