use std::{io::Write, net::TcpStream};

use crate::encode_bulk_string;

pub fn write_response(response: &[u8], mut stream: &TcpStream) {
    stream
        .write_all(response)
        .expect("failed to write to stream");
}

pub fn return_null(stream: &mut TcpStream) {
    let response = b"$-1\r\n";
    write_response(response, stream);
}

pub fn return_ok(stream: &mut TcpStream) {
    let response = b"+OK\r\n";
    write_response(response, stream);
}

pub fn return_bulk_string(value: String, stream: &mut TcpStream) {
    let response = encode_bulk_string(&value);
    write_response(response.as_bytes(), stream)
}
