use aws_config::ConfigLoader;
use aws_sdk_s3::config::Builder;
use aws_sdk_s3::Client;
use aws_sdk_s3::Endpoint;
use aws_sdk_s3::Region;

use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
pub struct FileSyncConfig {
    source_path: String,
    destination_path: String,
    s3_path: String,
}

async fn source_path_listener(source_path: String) {
    let shared_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let custom_endpoint =
        Endpoint::immutable("http://localhost:4566".parse().expect("Invalid URL"));
    let s3_config = Builder::from(&shared_config)
        .region(Region::new("us-east-1")) // Use any region, it's ignored by LocalStack
        .endpoint_resolver(custom_endpoint)
        .build();

    let client = Client::from_conf(s3_config);
    let custom_endpoint =
        Endpoint::immutable("http://localhost:4566".parse().expect("Invalid URL"));
    loop {
        println!("Looking for files in {}", source_path);
        let src_path = Path::new(&source_path);
        fs::create_dir_all(src_path).unwrap();
        let entries = fs::read_dir(src_path).unwrap();
        for entry in entries {
            let entry = entry.unwrap(); // Handle any errors while iterating
            let path = entry.path(); // Get the path of the entry

            // Check if the entry is a file
            if path.is_file() {
                println!("Found file: {:?}", path);
                let file = File::open(&path).expect("Could not open file");
                let mut buf_reader = std::io::BufReader::new(file);
                let mut contents = Vec::new();
                buf_reader
                    .read_to_end(&mut contents)
                    .expect("Failed to read file");

                let file_name = path.file_name().unwrap().to_str().unwrap();
            }
        }

        thread::sleep(Duration::new(15, 0));
    }
}

pub fn run(file_config: FileSyncConfig) {
    thread::spawn(move || source_path_listener(file_config.source_path));
    println!("Destination path: {}", file_config.destination_path);
}
