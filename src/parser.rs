use bytes::{Bytes, BytesMut};
use memchr::memchr;
use std::{collections::btree_map::Values, str};

pub type Value = Bytes;
pub type Key = Bytes;

#[derive(PartialEq, Clone, Debug)]
pub enum RedisValueRef {
    String(Bytes),
    Error(Bytes),
    Int(i64),
    Array(Vec<RedisValueRef>),
    NullArray,
    NullBulkString,
}

/// A type for viewing byte slices
pub struct BufSplit(usize, usize);

/// BufSplit based equivalent for RedisValueRef
pub enum RedisBufSplit {
    String(BufSplit),
    Error(BufSplit),
    Int(i64),
    Array(Vec<RedisBufSplit>),
    NullArray,
    NullBulkString,
}

#[derive(Debug)]
pub enum RESPError {
    UnexpectedEnd,
    UnknownStartingByte,
    IOError(std::io::Error),
    IntParseFailure,
    BadBulkStringSize(i64),
    BadArraySize(i64),
}

pub type RedisResult = Result<Option<(usize, RedisBufSplit)>, RESPError>;

pub struct RedisParser;

impl RedisParser {
    /// Create a new RESP parser
    pub fn new() -> Self {
        RedisParser
    }

    /// Parse word from RESP
    pub fn word(&self, buffer: &[u8], pos: usize) -> Option<(usize, BufSplit)> {
        // We're at the edge of `buf`, so we can't find a word.
        if buffer.len() <= pos {
            return None;
        }
        // Find the position of the b'\r'
        memchr(b'\r', &buffer[pos..]).and_then(|end| {
            if end + 1 < buffer.len() {
                // pos + end == first index of b'\r' after `pos`
                // pos + end + 2 == ..word\r\n<HERE> -- skip to after CLRF
                Some((pos + end + 2, BufSplit(pos, pos + end)))
            } else {
                // Edge case: We received just enough bytes from the client
                // to get the \r but not the \n
                None
            }
        })
    }

    /// Parse simple string from RESP
    pub fn simple_string(&self, buffer: &[u8], pos: usize) -> RedisResult {
        Ok(self
            .word(buffer, pos)
            .map(|(pos, word)| (pos, RedisBufSplit::String(word))))
    }

    /// Parse error from RESP
    pub fn error(&self, buffer: &[u8], pos: usize) -> RedisResult {
        Ok(self
            .word(buffer, pos)
            .map(|(pos, word)| (pos, RedisBufSplit::Error(word))))
    }

    /// Parse ints from RESP
    pub fn int(&self, buffer: &[u8], pos: usize) -> Result<Option<(usize, i64)>, RESPError> {
        match self.word(buffer, pos) {
            Some((pos, word)) => {
                // convert buffer to str
                let s = str::from_utf8(word.as_slice(buffer))
                    .map_err(|_| RESPError::IntParseFailure)?;
                // convert the string to a i64
                let i = s.parse().map_err(|_| RESPError::IntParseFailure)?;
                Ok(Some((pos, i)))
            }
            None => Ok(None),
        }
    }

    /// Parse RESP int
    pub fn resp_int(&self, buffer: &[u8], pos: usize) -> RedisResult {
        Ok(self
            .int(buffer, pos)?
            .map(|(pos, int)| (pos, RedisBufSplit::Int(int))))
    }

    /// Parse bulk strings
    pub fn bulk_string(&self, buffer: &[u8], pos: usize) -> RedisResult {
        match self.int(buffer, pos)? {
            // Null bulk string encountered
            Some((pos, -1)) => Ok(Some((pos, RedisBufSplit::NullBulkString))),
            Some((pos, size)) if size >= 0 => {
                let total_size = pos + size as usize;
                if buffer.len() < total_size + 2 {
                    Ok(None)
                } else {
                    let bb = RedisBufSplit::String(BufSplit(pos, total_size));
                    Ok(Some((total_size + 2, bb)))
                }
            }
            Some((_pos, bad_size)) => Err(RESPError::BadBulkStringSize(bad_size)),
            None => Ok(None),
        }
    }

    /// Parse RESP array
    pub fn array(&self, buffer: &[u8], pos: usize) -> RedisResult {
        match self.int(buffer, pos)? {
            None => Ok(None),
            Some((pos, -1)) => Ok(Some((pos, RedisBufSplit::NullArray))),
            Some((pos, num_elements)) if num_elements >= 0 => {
                let mut values = Vec::with_capacity(num_elements as usize);
                let mut curr_pos = pos;
                for _ in 0..num_elements {
                    match self.parse(buffer, curr_pos)? {
                        Some((new_pos, value)) => {
                            curr_pos = new_pos;
                            values.push(value);
                        }
                        None => return Ok(None),
                    }
                }
                Ok(Some((curr_pos, RedisBufSplit::Array(values))))
            }
            Some((_pos, bad_num_elements)) => Err(RESPError::BadArraySize(bad_num_elements)),
        }
    }

    /// Top level parse function
    pub fn parse(&self, buffer: &[u8], pos: usize) -> RedisResult {
        if buffer.is_empty() {
            return Ok(None);
        }

        match buffer[pos] {
            b'+' => self.simple_string(buffer, pos + 1),
            b'-' => self.error(buffer, pos + 1),
            b'$' => self.bulk_string(buffer, pos + 1),
            b':' => self.resp_int(buffer, pos + 1),
            b'*' => self.array(buffer, pos + 1),
            _ => Err(RESPError::UnknownStartingByte),
        }
    }

    /// decode function that will be used at the top level
    pub fn decode(&mut self, buffer: &mut BytesMut) -> Result<Option<RedisValueRef>, RESPError> {
        if buffer.is_empty() {
            return Ok(None);
        }

        match self.parse(buffer, 0)? {
            Some((pos, value)) => {
                let data = buffer.split_to(pos);
                Ok(Some(value.redis_value(&data.freeze())))
            }
            None => todo!(),
        }
    }
}

impl Default for RedisParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BufSplit {
    /// Get a lifetime appropriate slice of the underlying buffer.
    pub fn as_slice<'a>(&self, buffer: &'a [u8]) -> &'a [u8] {
        &buffer[self.0..self.1]
    }

    /// Get a Byte object representing the appropriate slice of bytes.
    pub fn as_bytes(&self, buffer: &Bytes) -> Bytes {
        buffer.slice(self.0..self.1)
    }
}

// Convert RedisBufSplit -> RedisValueRef given a &[u8] buffer.
impl RedisBufSplit {
    pub fn redis_value(self, buffer: &Bytes) -> RedisValueRef {
        match self {
            RedisBufSplit::String(bfs) => RedisValueRef::String(bfs.as_bytes(buffer)),
            RedisBufSplit::Error(bfs) => RedisValueRef::Error(bfs.as_bytes(buffer)),
            RedisBufSplit::Int(i) => RedisValueRef::Int(i),
            RedisBufSplit::Array(arr) => {
                RedisValueRef::Array(arr.into_iter().map(|bfs| bfs.redis_value(buffer)).collect())
            }
            RedisBufSplit::NullArray => RedisValueRef::NullArray,
            RedisBufSplit::NullBulkString => RedisValueRef::NullBulkString,
        }
    }
}
