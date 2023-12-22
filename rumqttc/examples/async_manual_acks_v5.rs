#![allow(dead_code, unused_imports)]
use tokio::{task, time};

use rumqttc::v5::{AsyncClient, Event, EventLoop, Filter, Message, MqttOptions, Packet, QoS};
use std::error::Error;
use std::time::Duration;

fn create_conn() -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new("test-1", "localhost", 1884);
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_manual_acks(true)
        .set_clean_start(false);

    AsyncClient::new(mqttoptions, 10)
}

#[tokio::main(worker_threads = 1)]
async fn main() -> Result<(), Box<dyn Error>> {
    // todo!("fix this example with new way of spawning clients")
    pretty_env_logger::init();

    // create mqtt connection with clean_session = false and manual_acks = true
    let (client, mut eventloop) = create_conn();

    // subscribe example topic
    let filter = Filter::new("hello/world", QoS::AtLeastOnce);
    client.subscribe(filter).await.unwrap();

    task::spawn(async move {
        // send some messages to example topic and disconnect
        requests(&client).await;
        client.disconnect().await.unwrap()
    });

    // get subscribed messages without acking
    loop {
        let event = eventloop.poll().await;
        match &event {
            Ok(v) => {
                println!("Event = {v:?}");
            }
            Err(e) => {
                println!("Error = {e:?}");
                break;
            }
        }
    }

    // create new broker connection
    let (client, mut eventloop) = create_conn();

    while let Ok(event) = eventloop.poll().await {
        println!("{event:?}");

        if let Event::Incoming(packet) = event {
            let publish = match packet {
                Packet::Publish(publish) => publish,
                _ => continue,
            };
            // this time we will ack incoming publishes.
            // Its important not to block notifier as this can cause deadlock.
            let c = client.clone();
            tokio::spawn(async move {
                c.ack(&publish).await.unwrap();
            });
        }
    }

    Ok(())
}

async fn requests(client: &AsyncClient) {
    let mut message = Message::new("hello/world", QoS::AtLeastOnce);

    for i in 1..=10 {
        message.payload = vec![1; i];

        client.publish(message.clone()).await.unwrap();

        time::sleep(Duration::from_secs(1)).await;
    }
}
