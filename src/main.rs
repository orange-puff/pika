use std::env;

use config::{Config, File};
mod file_sync;
mod server;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // If file path is provided, sync this file to the Queue and exit
    if args.len() > 1 {
        let file_path = &args[1];
        file_sync::sync_file(file_path).await;
        return;
    }

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
