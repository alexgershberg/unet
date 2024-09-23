pub mod challenge_response;
pub mod connection_request;
pub mod disconnect;
pub mod keep_alive;

use crate::packet::challenge_response::ChallengeResponse;
use crate::packet::connection_request::ConnectionRequest;
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
#[repr(u8)]
pub enum Packet {
    ConnectionRequest(ConnectionRequest) = 0,
    ChallengeRequest = 1,
    ChallengeResponse(ChallengeResponse) = 2,
    KeepAlive(KeepAlive) = 3,
    Data = 4,
    Disconnect(Disconnect) = 5,
    Unimplemented,
}

impl Packet {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let minimal_length = 1;
        if bytes.len() < minimal_length {
            return None;
        }

        let packet_type = bytes[0];
        let packet = match packet_type {
            0 => {
                let connection_request = ConnectionRequest::from_bytes(&bytes[1..]);
                Packet::ConnectionRequest(connection_request)
            }
            1 => Packet::ChallengeRequest,
            2 => Packet::ChallengeResponse(ChallengeResponse::from_bytes(&bytes[1..])),
            3 => {
                let keep_alive = KeepAlive::from_bytes(&bytes[1..]);
                Packet::KeepAlive(keep_alive)
            }
            4 => Packet::Data,
            5 => {
                let disconnect = Disconnect::from_bytes(&bytes[1..]);
                Packet::Disconnect(disconnect)
            }
            _ => Packet::Unimplemented,
        };

        Some(packet)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut output = vec![self.packet_id()];
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
            Packet::Data => {}
            Packet::Unimplemented => {}
        }

        output
    }

    pub fn packet_id(&self) -> u8 {
        match self {
            Packet::ConnectionRequest(_) => 0,
            Packet::ChallengeRequest => 1,
            Packet::ChallengeResponse(_) => 2,
            Packet::KeepAlive(_) => 3,
            Packet::Data => 4,
            Packet::Disconnect(_) => 5,
            Packet::Unimplemented => 6,
        }
    }

    pub fn header(&self) -> Header {
        match self {
            Packet::ConnectionRequest(connection_request) => connection_request.header,
            Packet::ChallengeRequest => todo!(),
            Packet::ChallengeResponse(challenge_response) => challenge_response.header,
            Packet::KeepAlive(keep_alive) => keep_alive.header,
            Packet::Data => todo!(),
            Packet::Disconnect(disconnect) => disconnect.header,
            Packet::Unimplemented => todo!(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Header {
    pub protocol_version: [u8; 5],
    pub client_id: UnetId,
}

impl Header {
    const SIZE: usize = size_of::<[u8; 5]>() + size_of::<UnetId>();

    pub fn new(client_id: UnetId) -> Self {
        Self {
            protocol_version: *b"UNET1",
            client_id,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::SIZE);

        let protocol_version: [u8; 5] = [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]];
        let client_id = UnetId(u64::from_be_bytes([
            bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12],
        ]));

        Self {
            protocol_version,
            client_id,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut output = vec![];
        output.append(&mut self.protocol_version.to_vec().clone());
        output.append(&mut self.client_id.0.to_be_bytes().to_vec().clone());
        output
    }
}
