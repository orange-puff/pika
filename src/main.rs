use config::{Config, File};
use serde::Deserialize;
use std::thread;
use std::time::Duration;

mod file_sync;
mod server;

#[derive(Deserialize)]
struct PikaConfig {
    start_server: bool,
}

#[tokio::main]
async fn main() {
    // For now, we do not need the server. But it could be useful in the future
    match Config::builder()
        .add_source(File::with_name("config.json").required(true))
        .build()
    {
        Ok(config_builder) => {
            if let Ok(pika_config) = config_builder.get::<PikaConfig>("PikaConfig") {
                if pika_config.start_server {
                    match config_builder.get::<server::ServerConfig>("ServerConfig") {
                        Ok(server_config) => {
                            thread::spawn(move || server::run(server_config));
                        }
                        Err(e) => {
                            println!("Error deserializing server config: {}", e);
                        }
                    }
                }
                match config_builder.get::<file_sync::FileSyncConfig>("FileSyncConfig") {
                    Ok(file_sync_config) => {
                        file_sync::run(file_sync_config).await;
                    }
                    Err(e) => {
                        println!("Error deserializing file sync config: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    loop {
        thread::sleep(Duration::new(15, 0));
    }
}
