use config::{Config, File};
use std::env;
mod file_sync;
use crate::file_sync::FileSync;

fn help() -> String {
    return "pika is a file uploader\n\n-u|--upload [path]\n-d|--download\n-h|--help".to_string();
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args[1] == "-h" || args[1] == "--help" || args.len() > 3 {
        println!("{}", help());
        return;
    }

    let mut upload = false;
    if args[1] == "-u" || args[1] == "--upload" {
        if args.len() == 3 {
            upload = true;
        }
    }

    if !upload && args[1] != "-d" && args[1] == "--download" {
        println!("{} ", help());
        return;
    }

    // For now, we do not need the server. But it could be useful in the future
    match Config::builder()
        .add_source(File::with_name("config.json").required(false))
        .add_source(File::with_name("/Users/john/.pika/config.json").required(false))
        .build()
    {
        Ok(config_builder) => {
            match config_builder.get::<file_sync::FileSyncConfig>("FileSyncConfig") {
                Ok(file_sync_config) => {
                    let file_sync = FileSync::new(file_sync_config);

                    if upload {
                        file_sync.upload(&std::path::Path::new(&args[2])).await;
                    } else {
                        file_sync.download().await;
                    }
                }
                Err(e) => {
                    println!("Error deserializing file sync config: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error parsing config: {}", e);
        }
    }
}
