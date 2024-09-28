use crate::packet::{Header, UnetId};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DisconnectReason {
    Timeout = 0,
    ServerFull = 1,
    Spam = 2,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Disconnect {
    pub header: Header,
    pub reason: DisconnectReason,
}

impl DisconnectReason {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::Timeout,
            1 => Self::ServerFull,
            2 => Self::Spam,
            _ => panic!("Badly formed DisconnectReason value: {byte}"),
        }
    }
}

impl Disconnect {
    pub fn new(client_id: UnetId, reason: DisconnectReason) -> Self {
        Self {
            header: Header::new(client_id),
            reason,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let header = Header::from_bytes(&bytes[..Header::SIZE]);
        let reason = DisconnectReason::from_byte(bytes[Header::SIZE]);

        Self { header, reason }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut output = vec![];
        output.append(&mut self.header.as_bytes());
        output.push(self.reason as u8);
        output
    }
}
