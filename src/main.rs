use config::{Config, File};
use std::thread;
use std::time::Duration;

mod file_sync;
mod server;

#[tokio::main]
async fn main() {
    // For now, we do not need the server. But it could be useful in the future
    match Config::builder()
        .add_source(File::with_name("config.json").required(true))
        .build()
    {
        Ok(config_builder) => {
            match config_builder.get::<server::ServerConfig>("ServerConfig") {
                Ok(server_config) => {
                    thread::spawn(move || server::run(server_config));
                }
                Err(e) => {
                    println!("Error deserializing server config: {}", e);
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
            loop {
                thread::sleep(Duration::new(15, 0));
            }
        }
        Err(e) => {
            println!("Error parsing config: {}", e);
        }
    }
}
