use std::{io::Write, net::TcpStream};

pub async fn run_replica(host: &String, port: &String) {
    let mut stream = TcpStream::connect(format!("{}:{}", &host, &port)).unwrap();
    let response = "*1\r\n$4\r\nping\r\n".as_bytes();

    stream
        .write_all(response)
        .expect("failed to write to stream on replica");
}
