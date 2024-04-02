use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::encode_bulk_string_from_array;

pub async fn run_replica(host: &String, port: &String) {
    let mut stream = TcpStream::connect(format!("{}:{}", &host, &port)).unwrap();
    let buffer = [0; 1024];

    // Connect replica to the master server
    handle_handshake(&mut stream, buffer).await;
}

async fn handle_handshake(stream: &mut TcpStream, mut buffer: [u8; 1024]) {
    // Send ping response
    let ping_response = encode_bulk_string_from_array(vec!["ping"]);
    stream
        .write_all(ping_response.as_bytes())
        .expect("failed to write to stream on replica");
    stream.flush().unwrap();
    let _res = stream.read(&mut buffer).unwrap();

    // Send first REPLCONF response
    let repl_conf_1 = encode_bulk_string_from_array(vec!["REPLCONF", "listening-port", "6380"]);
    stream
        .write_all(repl_conf_1.as_bytes())
        .expect("failed to write first REPLCONF response");
    let _res = stream.read(&mut buffer).unwrap();
    stream.flush().unwrap();

    // Send second REPLCONF response
    let repl_conf_2 = encode_bulk_string_from_array(vec!["REPLCONF", "capa", "psync2"]);
    stream
        .write_all(repl_conf_2.as_bytes())
        .expect("failed to write second REPLCONF response");
    stream.flush().unwrap();
    let _res = stream.read(&mut buffer).unwrap();

    // Send PSYNC command
    let psync = encode_bulk_string_from_array(vec!["PSYNC", "?", "-1"]);
    stream
        .write_all(psync.as_bytes())
        .expect("failed to send psync response");
    stream.flush().unwrap();
    let _res = stream.read(&mut buffer).unwrap();
}
