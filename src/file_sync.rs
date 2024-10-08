use lapin::{Connection, ConnectionProperties};

pub async fn sync_file(file_path: &str) {
    let addr = "amqp://user:password@raspberrypi.local:5672/%2f";
    if let Ok(conn) = Connection::connect(addr, ConnectionProperties::default()).await {
        println!("Connected to RabbitMQ");
        conn.close(0, "Normal shutdown").await.unwrap();
    } else {
        println!("Failed to connect to RabbitMQ");
    }
    println!("File path: {}", file_path);
}
