use std::str;

pub struct RedisParser;

impl RedisParser {
    pub fn new() -> Self {
        RedisParser
    }

    pub fn parse(&self, buffer: &[u8], bytes_read: usize) -> Vec<String> {
        let command_string = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        let command: Vec<String> = command_string
            .trim()
            .split("\r\n")
            .skip(2)
            .step_by(2)
            .map(|s| s.to_string().to_lowercase())
            .collect();

        println!("commands: {:?}", command);
        command
    }
}

impl Default for RedisParser {
    fn default() -> Self {
        Self::new()
    }
}
