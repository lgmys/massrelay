use futures_lite::stream::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties, Result,
};
use tracing::info;

fn remove_trailing_slash(string: &str) -> String {
    let mut string = string.to_string();
    if string.ends_with("/") {
        string.pop();
    }
    string
}

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let declare = std::env::var("DECLARE").is_ok();

    let source_addr =
        std::env::var("SOURCE_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/".into());
    let source_routing_key = std::env::var("SOURCE_ROUTING_KEY").unwrap_or_else(|_| "hello".into());
    let source_exchange = std::env::var("SOURCE_EXCHANGE").unwrap_or_else(|_| "hello".into());
    let source_queue = std::env::var("SOURCE_QUEUE").unwrap_or_else(|_| "hello".into());

    let target_addr =
        std::env::var("TARGET_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/".into());
    let target_exchange = std::env::var("TARGET_EXCHANGE").unwrap_or_else(|_| "hello".into());
    let target_queue = std::env::var("TARGET_QUEUE").unwrap_or_else(|_| "hello".into());

    let source_addr = remove_trailing_slash(&source_addr);
    let target_addr = remove_trailing_slash(&target_addr);

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

    info!("will consume queue {}", &source_queue);

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");

        info!("received message from queue {}", &source_queue);

        target_channel
            .basic_publish(
                &target_exchange,
                &target_queue,
                BasicPublishOptions::default(),
                &delivery.data,
                BasicProperties::default(),
            )
            .await
            .unwrap()
            .await?;

        delivery.ack(BasicAckOptions::default()).await?;

        info!("published message to queue {}", &target_queue);
    }

    Ok(())
}
