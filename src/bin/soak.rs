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
            let mut client = UnetClient::new("127.0.0.1:10010", None).unwrap();
            let mut count = 0;
            loop {
                client.send(Packet::Data(Data::new(client.id, count)));
                client.update();
                count += 1;
                sleep(Duration::from_millis(1)).await;
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
