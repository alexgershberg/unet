use crate::packet::{Header, UnetId};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Data {
    pub header: Header,
    pub val: i32,
}

impl Data {
    pub fn new(client_id: UnetId, val: i32) -> Self {
        Self {
            header: Header::new(client_id),
            val,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let header = Header::from_bytes(&bytes[..Header::SIZE]);
        let val = i32::from_be_bytes([
            bytes[bytes.len() - 4],
            bytes[bytes.len() - 3],
            bytes[bytes.len() - 2],
            bytes[bytes.len() - 1],
        ]);

        Self { header, val }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut output = vec![];
        output.append(&mut self.header.as_bytes());
        output.append(&mut self.val.to_be_bytes().to_vec());
        output
    }
}
