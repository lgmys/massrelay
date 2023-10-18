use futures_lite::stream::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties, Result,
};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let declare = std::env::var("DECLARE").is_ok();

    let source_addr =
        std::env::var("SOURCE_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/".into());
    let source_routing_key = std::env::var("ROUTING_KEY").unwrap_or_else(|_| "hello".into());
    let source_exchange = std::env::var("SOURCE_EXCHANGE").unwrap_or_else(|_| "hello".into());
    let source_queue = std::env::var("SOURCE_QUEUE").unwrap_or_else(|_| "hello".into());

    let target_addr =
        std::env::var("TARGET_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/".into());
    let target_exchange = std::env::var("TARGET_EXCHANGE").unwrap_or_else(|_| "hello".into());
    let target_queue = std::env::var("TARGET_QUEUE").unwrap_or_else(|_| "hello".into());

    let source_conn = Connection::connect(&source_addr, ConnectionProperties::default())
        .await
        .unwrap();

    info!("CONNECTED source server {}", source_addr);

    let target_conn = Connection::connect(&target_addr, ConnectionProperties::default())
        .await
        .unwrap();

    info!("CONNECTED target server {}", target_addr);

    let source_channel = source_conn.create_channel().await.unwrap();
    let target_channel = target_conn.create_channel().await.unwrap();

    if declare {
        let queue = source_channel
            .queue_declare(
                &source_queue,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();

        info!(?queue, "Declared queue");

        source_channel
            .queue_bind(
                &source_queue,
                &source_exchange,
                &source_routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
    }

    let mut consumer = source_channel
        .basic_consume(
            &source_queue,
            "massrelay",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();
    info!("will consume");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");

        debug!(?delivery, "received message");

        target_channel
            .basic_publish(
                &target_exchange,
                &target_queue,
                BasicPublishOptions::default(),
                &delivery.data,
                BasicProperties::default(),
            )
            .await
            .expect("publish");

        delivery.ack(BasicAckOptions::default()).await.expect("ack");

        debug!(?delivery, "published message");
    }

    Ok(())
}
