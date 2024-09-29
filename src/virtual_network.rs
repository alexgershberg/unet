use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct VirtualNetwork {
    pub tx: Sender<Vec<u8>>,
    pub rx: Receiver<Vec<u8>>
}

