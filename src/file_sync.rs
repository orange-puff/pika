use serde::Deserialize;

#[derive(Deserialize)]
pub struct FileSyncConfig {
    source_path: String,
    destination_path: String,
}

pub fn run(config: FileSyncConfig) {
    println!("Source path: {}", config.source_path);
    println!("Destination path: {}", config.destination_path);
}
