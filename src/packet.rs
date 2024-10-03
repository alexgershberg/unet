pub mod challenge_response;
pub mod connection_request;
pub mod data;
pub mod disconnect;
pub mod keep_alive;

use crate::packet::challenge_response::ChallengeResponse;
use crate::packet::connection_request::ConnectionRequest;
use crate::packet::data::Data;
use crate::packet::disconnect::Disconnect;
use crate::packet::keep_alive::KeepAlive;
use rand::random;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct UnetId(pub u64);

impl UnetId {
    pub fn new() -> Self {
        let id = random::<u64>();
        Self(id)
    }
}

impl Default for UnetId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum PacketKind {
    ConnectionRequest = 0,
    ChallengeRequest = 1,
    ChallengeResponse = 2,
    KeepAlive = 3,
    Data = 4,
    Disconnect = 5,
    Unimplemented,
}
impl PacketKind {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => PacketKind::ConnectionRequest,
            1 => PacketKind::ChallengeRequest,
            2 => PacketKind::ChallengeResponse,
            3 => PacketKind::KeepAlive,
            4 => PacketKind::Data,
            5 => PacketKind::Disconnect,
            _ => PacketKind::Unimplemented,
        }
    }

    pub fn as_byte(&self) -> u8 {
        match self {
            PacketKind::ConnectionRequest => 0,
            PacketKind::ChallengeRequest => 1,
            PacketKind::ChallengeResponse => 2,
            PacketKind::KeepAlive => 3,
            PacketKind::Data => 4,
            PacketKind::Disconnect => 5,
            PacketKind::Unimplemented => {
                panic!("Tried calling as_byte() on PacketKind::Unimplemented")
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum Packet {
    ConnectionRequest(ConnectionRequest),
    ChallengeRequest,
    ChallengeResponse(ChallengeResponse),
    KeepAlive(KeepAlive),
    Data(Data),
    Disconnect(Disconnect),
    Unimplemented,
}

impl Packet {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let minimal_length = 1;
        if bytes.len() < minimal_length {
            return None;
        }

        let packet_kind = PacketKind::from_byte(bytes[0]);
        let packet = match packet_kind {
            PacketKind::ConnectionRequest => {
                let connection_request = ConnectionRequest::from_bytes(&bytes[1..]);
                Packet::ConnectionRequest(connection_request)
            }
            PacketKind::ChallengeRequest => Packet::ChallengeRequest,
            PacketKind::ChallengeResponse => {
                Packet::ChallengeResponse(ChallengeResponse::from_bytes(&bytes[1..]))
            }
            PacketKind::KeepAlive => {
                let keep_alive = KeepAlive::from_bytes(&bytes[1..]);
                Packet::KeepAlive(keep_alive)
            }
            PacketKind::Data => {
                let data = Data::from_bytes(&bytes[1..]);
                Packet::Data(data)
            }
            PacketKind::Disconnect => {
                let disconnect = Disconnect::from_bytes(&bytes[1..]);
                Packet::Disconnect(disconnect)
            }
            _ => Packet::Unimplemented,
        };

        Some(packet)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut output = vec![self.kind().as_byte()];
        match self {
            Packet::ConnectionRequest(connection_request) => {
                let mut bytes = connection_request.as_bytes();
                output.append(&mut bytes);
            }
            Packet::ChallengeRequest => {}
            Packet::ChallengeResponse(challenge_response) => {
                let mut bytes = challenge_response.as_bytes();
                output.append(&mut bytes);
            }
            Packet::Disconnect(disconnect) => {
                let mut bytes = disconnect.as_bytes();
                output.append(&mut bytes);
            }
            Packet::KeepAlive(keep_alive) => {
                let mut bytes = keep_alive.as_bytes();
                output.append(&mut bytes);
            }
            Packet::Data(data) => {
                let mut bytes = data.as_bytes();
                output.append(&mut bytes);
            }
            Packet::Unimplemented => {}
        }

        output
    }

    pub fn kind(&self) -> PacketKind {
        match self {
            Packet::ConnectionRequest(_) => PacketKind::ConnectionRequest,
            Packet::ChallengeRequest => PacketKind::ChallengeRequest,
            Packet::ChallengeResponse(_) => PacketKind::ChallengeResponse,
            Packet::KeepAlive(_) => PacketKind::KeepAlive,
            Packet::Data(_) => PacketKind::Data,
            Packet::Disconnect(_) => PacketKind::Disconnect,
            Packet::Unimplemented => PacketKind::Unimplemented,
        }
    }

    pub fn header(&self) -> Header {
        match self {
            Packet::ConnectionRequest(connection_request) => connection_request.header,
            Packet::ChallengeRequest => todo!(),
            Packet::ChallengeResponse(challenge_response) => challenge_response.header,
            Packet::KeepAlive(keep_alive) => keep_alive.header,
            Packet::Data(data) => data.header,
            Packet::Disconnect(disconnect) => disconnect.header,
            Packet::Unimplemented => todo!(),
        }
    }

    pub fn set_sequence(&mut self, sequence: u64) {
        match self {
            Packet::ConnectionRequest(mut connection_request) => {
                connection_request.header.sequence = sequence
            }
            Packet::ChallengeRequest => {}
            Packet::ChallengeResponse(challenge_response) => {
                challenge_response.header.sequence = sequence
            }
            Packet::KeepAlive(keep_alive) => keep_alive.header.sequence = sequence,
            Packet::Data(data) => data.header.sequence = sequence,
            Packet::Disconnect(disconnect) => disconnect.header.sequence = sequence,
            Packet::Unimplemented => {}
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Header {
    pub protocol_version: [u8; 5],
    pub client_id: UnetId,
    pub sequence: u64,
}

impl Header {
    const SIZE: usize = size_of::<[u8; 5]>() + size_of::<UnetId>() + size_of::<u64>();

    pub fn new(client_id: UnetId) -> Self {
        Self {
            protocol_version: *b"UNET1",
            client_id,
            sequence: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::SIZE);

        let protocol_version: [u8; 5] = [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]];
        let client_id = UnetId(u64::from_be_bytes([
            bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12],
        ]));
        let sequence = u64::from_be_bytes([
            bytes[13], bytes[14], bytes[15], bytes[16], bytes[17], bytes[18], bytes[19], bytes[20],
        ]);

        Self {
            protocol_version,
            client_id,
            sequence,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut output = vec![];
        output.extend_from_slice(&self.protocol_version);
        output.extend_from_slice(&self.client_id.0.to_be_bytes());
        output.extend_from_slice(&self.sequence.to_be_bytes());
        assert_eq!(output.len(), Header::SIZE);
        output
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{Header, UnetId};

    #[test]
    fn from_bytes() {
        let bytes = vec![
            85, 78, 69, 84, 49, 0, 0, 0, 0, 0, 0, 3, 231, 0, 0, 0, 0, 0, 0, 0, 123,
        ];
        let header = Header::from_bytes(&bytes);
        assert_eq!(header.protocol_version, *b"UNET1");
        assert_eq!(header.client_id, UnetId(999));
        assert_eq!(header.sequence, 123);
    }

    #[test]
    fn as_bytes() {
        let mut header = Header::new(UnetId(999));
        header.sequence = 123;
        let bytes = header.as_bytes();
        assert_eq!(
            bytes,
            vec![85, 78, 69, 84, 49, 0, 0, 0, 0, 0, 0, 3, 231, 0, 0, 0, 0, 0, 0, 0, 123]
        )
    }
}
