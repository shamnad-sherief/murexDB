use bytes::{Buf, BufMut};
use murex_common::{Key, Value};

// OpCodes for commands
pub const OP_PING: u8 = 0x01;
pub const OP_GET: u8 = 0x02;
pub const OP_SET: u8 = 0x03;
pub const OP_DELETE: u8 = 0x04;
pub const OP_HELP: u8 = 0x05;

// OpCodes for Responses
pub const OP_RESPONSE_OK: u8 = 0x80;
pub const OP_RESPONSE_NOT_FOUND: u8 = 0x81;
pub const OP_RESPONSE_ERR_INVALID_FRAME: u8 = 0x82;
pub const OP_RESPONSE_ERR_SERVER_ERROR: u8 = 0x83;
pub const OP_RESPONSE_HELP: u8 = 0x84;

pub const MAGIC_BYTES: [u8; 2] = [0x4D, 0x58]; // "MX" magic byte
pub const MAX_PAYLOAD_LEN: u32 = 67_108_864; // 64 MB

#[derive(Debug, PartialEq)]
pub struct Header {
    pub magic: [u8; 2],
    pub op_code: u8,
    pub flags: u8,
    pub payload_len: u32,
}

impl Header {
    pub fn new(op_code: u8, flags: u8, payload_len: u32) -> Self {
        Self {
            magic: MAGIC_BYTES,
            op_code,
            flags,
            payload_len,
        }
    }

    pub fn encode(&self) -> [u8; 8] {
        let mut header = [0; 8];
        header[0..2].copy_from_slice(&self.magic);
        header[2] = self.op_code;
        header[3] = self.flags;
        header[4..8].copy_from_slice(&self.payload_len.to_be_bytes());
        header
    }

    pub fn decode(buf: &[u8; 8]) -> murex_common::Result<Self> {
        if buf[0..2] != MAGIC_BYTES {
            return Err(murex_common::MurexError::InvalidFrame(
                "Magic Bytes doesnt match".to_owned(),
            ));
        }

        let op_code = buf[2];
        let flags = buf[3];
        let payload_len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

        if payload_len > MAX_PAYLOAD_LEN {
            return Err(murex_common::MurexError::InvalidFrame(
                "Payload Length exceeds maximum allowed size".to_owned(),
            ));
        }

        Ok(Self {
            magic: MAGIC_BYTES,
            op_code,
            flags,
            payload_len,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping(Option<Value>),
    Get(Key),
    Set(Key, Value),
    Delete(Key),
    Help,
}

impl Command {
    pub fn decode(header: &Header, payload: &[u8]) -> murex_common::Result<Self> {
        match header.op_code {
            OP_PING => {
                if payload.is_empty() {
                    Ok(Command::Ping(None))
                } else {
                    Ok(Command::Ping(Some(Value::from(payload))))
                }
            }
            OP_GET => {
                let mut buf = payload;
                if buf.len() < 2 {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "GET payload missing key length prefix".into(),
                    ));
                }
                let key_len = buf.get_u16() as usize;
                if buf.len() < key_len {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "GET payload missing key bytes".into(),
                    ));
                }
                let key = buf[..key_len].to_vec();
                Ok(Command::Get(key))
            }
            OP_SET => {
                let mut buf = payload;
                if buf.len() < 2 {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "Payload too short".into(),
                    ));
                }

                //Reads u16 and automatically moves cursor forward 2 bytes!
                let key_len = buf.get_u16() as usize;
                if buf.len() < key_len + 4 {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "Payload missing key data".into(),
                    ));
                }

                // Take key bytes and move cursor forward by key_len
                let key = buf[..key_len].to_vec();
                buf.advance(key_len);

                //  Reads u32 and automatically moves cursor forward 4 bytes!
                let val_len = buf.get_u32() as usize;
                if buf.len() < val_len {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "Payload missing value data".into(),
                    ));
                }

                //  Take value bytes
                let value = buf[..val_len].to_vec();
                Ok(Command::Set(key, value))
            }
            OP_DELETE => {
                let mut buf = payload;
                if buf.len() < 2 {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "DELETE payload missing key length prefix".into(),
                    ));
                }
                let key_len = buf.get_u16() as usize;
                if buf.len() < key_len {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "DELETE payload missing key bytes".into(),
                    ));
                }
                let key = buf[..key_len].to_vec();
                Ok(Command::Delete(key))
            }
            OP_HELP => Ok(Command::Help),
            _ => Err(murex_common::MurexError::InvalidFrame(
                "Unknown OpCode".to_owned(),
            )),
        }
    }

    pub fn encode(&self) -> murex_common::Result<(Header, Vec<u8>)> {
        match self {
            Command::Ping(msg) => {
                let payload = msg.clone().unwrap_or_default();
                let header = Header::new(OP_PING, 0, payload.len() as u32);
                Ok((header, payload))
            }
            Command::Get(key) => {
                if key.len() > u16::MAX as usize {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "Key is too big".to_owned(),
                    ));
                }
                // allocate space (2 bytes) for key len
                let mut payload = bytes::BytesMut::with_capacity(2 + key.len());
                payload.put_u16(key.len() as u16);

                // put the payload next
                payload.put_slice(key);
                let header = Header::new(OP_GET, 0, payload.len() as u32);
                Ok((header, payload.to_vec()))
            }
            Command::Set(key, value) => {
                if key.len() > u16::MAX as usize {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "Key is too big".to_owned(),
                    ));
                }

                // first allocate the space for the keylen& keybytes and valuelen and valuebytes
                let mut payload = bytes::BytesMut::with_capacity(2 + key.len() + 4 + value.len());
                payload.put_u16(key.len() as u16);

                // put the key bytes next
                payload.put_slice(key);

                // next allocate the space for thevaluelen and valuebytes
                payload.put_u32(value.len() as u32);

                payload.put_slice(value);

                let header = Header::new(OP_SET, 0, payload.len() as u32);
                Ok((header, payload.to_vec()))
            }
            Command::Delete(key) => {
                if key.len() > u16::MAX as usize {
                    return Err(murex_common::MurexError::InvalidFrame(
                        "Key size exceeds u16 limit".into(),
                    ));
                }

                // add the space for the key_len and key_bytes
                let mut payload = bytes::BytesMut::with_capacity(2 + key.len());

                payload.put_u16(key.len() as u16);

                payload.put_slice(key);

                let header = Header::new(OP_DELETE, 0, payload.len() as u32);
                Ok((header, payload.to_vec()))
            }
            Command::Help => {
                let header = Header::new(OP_HELP, 0, 0);
                Ok((header, Vec::new()))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Response {
    Ok(Option<Value>),
    Error(String),
    NotFound,
    Help(String),
}

impl Response {
    pub fn decode(header: &Header, payload: &[u8]) -> Result<Self, murex_common::MurexError> {
        match header.op_code {
            OP_RESPONSE_OK => {
                if payload.is_empty() {
                    Ok(Response::Ok(None))
                } else {
                    let mut buf = payload;
                    if buf.len() < 4 {
                        return Err(murex_common::MurexError::InvalidFrame(
                            "OK response payload too short".into(),
                        ));
                    }
                    let val_len = buf.get_u32() as usize;
                    if buf.len() < val_len {
                        return Err(murex_common::MurexError::InvalidFrame(
                            "OK response payload missing value bytes".into(),
                        ));
                    }
                    let val = buf[..val_len].to_vec();
                    Ok(Response::Ok(Some(val)))
                }
            }
            OP_RESPONSE_ERR_INVALID_FRAME | OP_RESPONSE_ERR_SERVER_ERROR => Ok(Response::Error(
                String::from_utf8_lossy(payload).to_string(),
            )),
            OP_RESPONSE_NOT_FOUND => Ok(Response::NotFound),
            OP_RESPONSE_HELP => Ok(Response::Help(String::from_utf8_lossy(payload).to_string())),
            _ => Err(murex_common::MurexError::InvalidFrame(
                "Unknown OpCode".to_owned(),
            )),
        }
    }

    pub fn encode(&self) -> murex_common::Result<(Header, Vec<u8>)> {
        match self {
            Response::Ok(value) => {
                if let Some(val) = value {
                    let mut payload = bytes::BytesMut::with_capacity(4 + val.len());
                    payload.put_u32(val.len() as u32);
                    payload.put_slice(val);

                    let header = Header::new(OP_RESPONSE_OK, 0, val.len() as u32);
                    Ok((header, payload.to_vec()))
                } else {
                    let header = Header::new(OP_RESPONSE_OK, 0, 0);
                    Ok((header, Vec::new()))
                }
            }
            Response::NotFound => {
                let header = Header::new(OP_RESPONSE_NOT_FOUND, 0, 0);
                Ok((header, Vec::new()))
            }
            Response::Error(err) => {
                let payload = err.as_bytes().to_vec();
                let header = Header::new(OP_RESPONSE_ERR_SERVER_ERROR, 0, payload.len() as u32);
                Ok((header, payload))
            }
            Response::Help(msg) => {
                let payload = msg.as_bytes().to_vec();
                let header = Header::new(OP_RESPONSE_HELP, 0, payload.len() as u32);
                Ok((header, payload))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_encode_decode() {
        let header = Header::new(OP_SET, 0, 1024);
        let bytes = header.encode();
        let decoded = Header::decode(&bytes).unwrap();
        assert_eq!(header, decoded)
    }

    #[test]
    fn test_invalid_magic_bytes() {
        let magic = [0x4E, 0x58];
        let header = Header {
            flags: 0,
            magic,
            op_code: OP_GET,
            payload_len: 1024,
        };

        let bytes = header.encode();

        let decoded = Header::decode(&bytes);
        assert!(decoded.is_err())
    }

    #[test]
    fn test_command_set_roundtrip() {
        let cmd = Command::Set(b"user:1".to_vec(), b"Alice".to_vec());
        let (header, payload) = cmd.encode().unwrap();
        let decoded = Command::decode(&header, &payload).unwrap();
        assert_eq!(cmd, decoded);
    }
    #[test]
    fn test_command_get_roundtrip() {
        let cmd = Command::Get(b"user:1".to_vec());
        let (header, payload) = cmd.encode().unwrap();
        let decoded = Command::decode(&header, &payload).unwrap();
        assert_eq!(cmd, decoded);
    }
    #[test]
    fn test_response_ok_roundtrip() {
        let resp = Response::Ok(Some(b"Alice".to_vec()));
        let (header, payload) = resp.encode().unwrap();
        let decoded = Response::decode(&header, &payload).unwrap();
        assert_eq!(resp, decoded);
    }
}
