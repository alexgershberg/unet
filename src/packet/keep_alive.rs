use crate::packet::{Header, UnetId};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct KeepAlive {
    pub header: Header,
}

impl KeepAlive {
    pub fn new(client_id: UnetId) -> Self {
        Self {
            header: Header::new(client_id),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let header = Header::from_bytes(&bytes);
        Self { header }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut output = vec![];
        output.append(&mut self.header.as_bytes());
        output
    }
}
