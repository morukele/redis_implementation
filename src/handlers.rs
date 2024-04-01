use std::{
    io::Read,
    net::TcpStream,
    str,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use bytes::{BufMut, BytesMut};

use crate::{
    return_bulk_string, return_null, return_ok, write_response, Database, Mode, RedisParser,
    RedisValueRef, Server,
};

pub async fn handle_client(
    mut stream: TcpStream,
    store: Arc<Mutex<Database>>,
    server_info: Arc<Mutex<Server>>,
) {
    let mut buffer = [0; 1024];
    loop {
        let db = Arc::clone(&store);
        let server_info = Arc::clone(&server_info);
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
        process_command(&output, &mut stream, db, server_info).await;
    }
}

async fn process_command(
    commands: &RedisValueRef,
    stream: &mut TcpStream,
    store: Arc<Mutex<Database>>,
    server_info: Arc<Mutex<Server>>,
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
                        "ping" => handle_ping(stream).await,
                        "echo" => handle_echo(stream, &arr[1..]).await,
                        "get" => handle_get(stream, &arr[1..], store).await,
                        "set" => handle_set(stream, &arr[1..], store).await,
                        "info" => handle_info(stream, &arr[1..], server_info).await,
                        "replconf" => handle_replconf(stream, &arr[1..]),
                        _ => println!("Unknown command"),
                    }
                }
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
}

fn handle_replconf(stream: &mut TcpStream, _commands: &[RedisValueRef]) {
    return_ok(stream)
}

async fn handle_info(
    stream: &mut TcpStream,
    _commands: &[RedisValueRef],
    server_info: Arc<Mutex<Server>>,
) {
    // Extract master replid
    let master_replid = server_info
        .lock()
        .expect("unable to get lock for server info")
        .master_replid
        .clone()
        .expect("unable to get master_replid");

    // Extract master repl offset
    let master_repl_offset = server_info
        .lock()
        .expect("unable to get lock for server info")
        .master_repl_offset;

    let mode = server_info.lock().unwrap().mode.clone();

    match mode {
        Mode::Master => {
            let response = format!(
                "role:{}\r\nmaster_replid:{}\r\nmaster_repl_offset:{}",
                mode, master_replid, master_repl_offset
            );

            return_bulk_string(response, stream);
        }
        Mode::Slave => {
            let value = format!("role:{}", mode);
            return_bulk_string(value, stream);
        }
    }
}

async fn handle_set(
    stream: &mut TcpStream,
    commands: &[RedisValueRef],
    store: Arc<Mutex<Database>>,
) {
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

async fn handle_get(
    stream: &mut TcpStream,
    commands: &[RedisValueRef],
    store: Arc<Mutex<Database>>,
) {
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

async fn handle_ping(stream: &mut TcpStream) {
    write_response(b"+PONG\r\n", stream)
}

async fn handle_echo(stream: &mut TcpStream, commands: &[RedisValueRef]) {
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
