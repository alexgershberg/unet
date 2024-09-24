use std::time::{SystemTime, UNIX_EPOCH};

pub struct ConnectToken {
    create_timestamp: u64,
    expire_timestamp: u64,
}

impl ConnectToken {
    pub fn new(expire_seconds: u64) -> Self {
        let create_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expire_timestamp = create_timestamp + expire_seconds;
        Self {
            create_timestamp,
            expire_timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {}
}
