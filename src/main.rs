use futures_lite::stream::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties,
    Result,
};
use tracing::{error, info};

fn remove_trailing_slash(string: &str) -> String {
    let mut string = string.to_string();
    if string.ends_with("/") {
        string.pop();
    }
    string
}

async fn declare_queue_and_binding(source_channel: &Channel, source_queue: &str) {
    let source_routing_key = std::env::var("SOURCE_ROUTING_KEY").unwrap();
    let source_exchange = std::env::var("SOURCE_EXCHANGE").unwrap();

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

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let declare = std::env::var("DECLARE").is_ok();

    let source_addr = std::env::var("SOURCE_ADDR").unwrap();
    let source_queue = std::env::var("SOURCE_QUEUE").unwrap();

    let target_addr = std::env::var("TARGET_ADDR").unwrap();
    let target_exchange = std::env::var("TARGET_EXCHANGE").unwrap();
    let target_routing_key = std::env::var("TARGET_ROUTING_KEY").unwrap();

    // Remove trailing slashes
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
        declare_queue_and_binding(&source_channel, &source_queue).await;
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

        let publish_result = target_channel
            .basic_publish(
                &target_exchange,
                &target_routing_key,
                BasicPublishOptions::default(),
                &delivery.data,
                BasicProperties::default(),
            )
            .await;

        match publish_result {
            Ok(confirm) => {
                match confirm.await {
                    Ok(_) => match delivery.ack(BasicAckOptions::default()).await {
                        Ok(_) => {
                            info!("published message to routing key {}", &target_routing_key);
                        }
                        Err(e) => {
                            error!("error acknowledging message");
                            error!("error: {}", e);
                            continue;
                        }
                    },
                    Err(e) => {
                        error!(
                            "error publishing message to routing key {}",
                            &target_routing_key
                        );
                        error!("error: {}", e);
                        continue;
                    }
                };
            }
            Err(e) => {
                error!(
                    "error publishing message to routing key {}",
                    &target_routing_key
                );
                error!("error: {}", e);
                continue;
            }
        };
    }

    Ok(())
}
