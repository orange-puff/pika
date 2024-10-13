use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Duration;

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_types::region::Region;

#[derive(Deserialize)]
pub struct FileSyncConfig {
    source_path: String,
    destination_path: String,
}

async fn source_path_listener(source_path: String) {
    let credentials_provider =
        SharedCredentialsProvider::new(Credentials::new("test", "test", None, None, "test"));
    // Set up the AWS region and config
    let config = aws_types::SdkConfig::builder()
        .endpoint_url("http://192.168.0.185:4566")
        .region(Region::new("us-east-1"))
        .credentials_provider(credentials_provider)
        .build();

    // Create an S3 client
    let client = Client::new(&config);

    // Specify your bucket name
    let bucket_name = "sync-files";

    // List objects in the bucket
    let resp = client
        .list_objects_v2()
        .bucket(bucket_name)
        .send()
        .await
        .unwrap();

    // Print the keys (file names) of the objects in the bucket
    if let Some(objects) = resp.contents() {
        println!("Files in bucket '{}':", bucket_name);
        for object in objects {
            println!("{}", object.key().unwrap_or("No key"));
        }
    } else {
        println!("No files found in the bucket.");
    }
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
                println!("File name: {}", file_name);
            }
        }

        thread::sleep(Duration::new(15, 0));
    }
}

pub async fn run(file_config: FileSyncConfig) {
    // thread::spawn(move || source_path_listener(file_config.source_path));
    source_path_listener(file_config.source_path).await;
    println!("Destination path: {}", file_config.destination_path);
}
