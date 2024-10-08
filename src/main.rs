use std::env;

use config::{Config, File};
use lapin::{Connection, ConnectionProperties};
mod server;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // If file path is provided, sync this file to the Queue and exit
    if args.len() > 1 {
        let file_path = &args[1];
        println!("File path: {}", file_path);
        return;
    }
    /*
    let addr = "amqp://user:password@raspberrypi.local:5672/%2f";
    if let Ok(conn) = Connection::connect(addr, ConnectionProperties::default()).await {
        println!("Connected to RabbitMQ");
        conn.close(0, "Normal shutdown").await.unwrap();
    } else {
        println!("Failed to connect to RabbitMQ");
    }
    */

    match Config::builder()
        .add_source(File::with_name("config.json").required(true))
        .build()
    {
        Ok(config_builder) => match config_builder.get::<server::ServerConfig>("ServerConfig") {
            Ok(server_config) => {
                server::run(server_config);
            }
            Err(e) => {
                println!("Error deserializing server config: {}", e);
            }
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
