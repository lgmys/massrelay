# Massrelay

Mass relay pushes messages from one amqp server to another. It is designed to be used with [RabbitMQ](https://www.rabbitmq.com/).

## Configuration

Single instance of this application relays events from exactly one source to exactly one destination. It is possible to run multiple instances of this application to relay events from multiple sources to multiple destinations.

Configuration is done via environment variables. The following variables are available:

```
DECLARE = 'false' # If true, declare the target exchange and queue. False by default.

SOURCE_ADDR = 'amqp://guest:guest@localhost:5672/'
TARGET_ADDR = 'amqp://guest:guest@localhost:5672/'
SOURCE_QUEUE = 'source_queue'
TARGET_EXCHANGE = 'target_exchange'
TARGET_QUEUE = 'target_queue'

# Required with DECLARE = true, otherwise ignored.
SOURCE_ROUTING_KEY = 'source_routing_key'
SOURCE_EXCHANGE = 'source_exchange'
```
