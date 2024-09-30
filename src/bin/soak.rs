use futures::future::TryJoinAll;
use std::time::Duration;
use tokio::time::sleep;
use unet::client::UnetClient;
use unet::packet::data::Data;
use unet::packet::{Packet, UnetId};
use unet::MAX_CONNECTIONS;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let mut handles = vec![];
    for _ in 0..MAX_CONNECTIONS {
        let handle = tokio::spawn(async move {
            let mut client = UnetClient::new().unwrap();
            let mut count = 0;
            sleep(Duration::from_secs(1)).await; // Without this Tokio only spawns ~7 clients, no idea why.
            while client.update() {
                client.send(Packet::Data(Data::new(client.id, count)));
                count += 1;
            }
        });
        handles.push(handle);
    }

    handles
        .into_iter()
        .collect::<TryJoinAll<_>>()
        .await
        .unwrap();
}
