use std::{
    io::{Read, Write},
    net::TcpStream,
};
use tokio::time::Duration;

pub async fn run_replica(host: &String, port: &String) {
    let mut stream = TcpStream::connect(format!("{}:{}", &host, &port)).unwrap();
    let mut buffer = [0; 1024];

    // Send ping response
    let ping_response = "*1\r\n$4\r\nping\r\n".as_bytes();
    stream
        .write_all(ping_response)
        .expect("failed to write to stream on replica");
    stream.flush().unwrap();
    let _res = stream.read(&mut buffer).unwrap();

    // Send first REPLCONF response
    let repl_conf_1 = "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n".as_bytes();
    stream
        .write_all(repl_conf_1)
        .expect("failed to write first REPLCONF response");
    let _res = stream.read(&mut buffer).unwrap();
    stream.flush().unwrap();

    // Send second REPLCONF response
    let repl_conf_2 = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n".as_bytes();
    stream
        .write_all(repl_conf_2)
        .expect("failed to write second REPLCONF response");
    stream.flush().unwrap();
    let _res = stream.read(&mut buffer).unwrap();
}
