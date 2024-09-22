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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Packet {
    ConnectionRequest(ConnectionRequest) = 0,
    ChallengeRequest = 1,
    ChallengeResponse = 2,
    KeepAlive = 3,
    Data = 4,
    Disconnect = 5,
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
            2 => Packet::ChallengeResponse,
            3 => Packet::KeepAlive,
            4 => Packet::Data,
            5 => Packet::Disconnect,
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
            Packet::ChallengeResponse => {}
            Packet::Disconnect => {}
            Packet::KeepAlive => {}
            Packet::Data => {}
            Packet::Unimplemented => {}
        }

        output
    }

    pub fn packet_id(&self) -> u8 {
        match self {
            Packet::ConnectionRequest(_) => 0,
            Packet::ChallengeRequest => 1,
            Packet::ChallengeResponse => 2,
            Packet::KeepAlive => 3,
            Packet::Data => 4,
            Packet::Disconnect => 5,
            Packet::Unimplemented => 6,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct ConnectionRequest {
    pub protocol_version: [u8; 5],
    pub client_id: UnetId,
}

impl ConnectionRequest {
    pub fn new(client_id: UnetId) -> Self {
        Self {
            protocol_version: *b"UNET1",
            client_id,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        println!("ConnectionRequest::from_bytes(): {bytes:#?}");
        let sz = size_of::<[u8; 5]>() + size_of::<UnetId>();
        assert_eq!(bytes.len(), sz);

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
