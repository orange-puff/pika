use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use aws_types::region::Region;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::task;

#[derive(Deserialize)]
pub struct FileSyncConfig {
    source_path: String,
    destination_path: String,
    s3_path: String,
    bucket_name: String,
}

pub struct FileSync {
    file_sync_config: FileSyncConfig,
    aws_client: Client,
}

impl FileSync {
    pub fn new(file_sync_config: FileSyncConfig) -> Self {
        let credentials_provider =
            SharedCredentialsProvider::new(Credentials::new("test", "test", None, None, "test"));
        // Set up the AWS region and config
        let config = aws_types::SdkConfig::builder()
            .endpoint_url(&file_sync_config.s3_path)
            .region(Region::new("us-east-1"))
            .credentials_provider(credentials_provider)
            .build();
        Self {
            file_sync_config,
            aws_client: Client::new(&config),
        }
    }
}

async fn source_path_listener(file_sync: Arc<FileSync>) {
    /*
    // List objects in the bucket
    let resp = file_sync
        .aws_client
        .list_objects_v2()
        .bucket(&file_sync.file_sync_config.bucket_name)
        .send()
        .await
        .unwrap();

    // Print the keys (file names) of the objects in the bucket
    if let Some(objects) = resp.contents() {
        println!(
            "Files in bucket '{}':",
            file_sync.file_sync_config.bucket_name
        );
        for object in objects {
            println!("{}", object.key().unwrap_or("No key"));
        }
    } else {
        println!("No files found in the bucket.");
    }
    */

    loop {
        println!(
            "Looking for files in {}",
            file_sync.file_sync_config.source_path
        );
        let src_path = Path::new(&file_sync.file_sync_config.source_path);
        fs::create_dir_all(src_path).unwrap();
        let entries = fs::read_dir(src_path).unwrap();
        for entry in entries {
            let entry = entry.unwrap(); // Handle any errors while iterating
            let path = entry.path(); // Get the path of the entry

            // Check if the entry is a file
            if path.is_file() {
                println!("Found file: {:?}.", path);
                let mut file = File::open(&path).expect("Could not open file");
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap(); // Read the file into a Vec<u8>
                let byte_stream = ByteStream::from(buffer);
                println!(
                    "Sending to S3 bucket {}...",
                    file_sync.file_sync_config.bucket_name
                );
                file_sync
                    .aws_client
                    .put_object()
                    .bucket(&file_sync.file_sync_config.bucket_name)
                    .key(path.file_name().unwrap().to_str().unwrap())
                    .body(byte_stream)
                    .send()
                    .await
                    .unwrap();

                println!("Removing file {:?}...", path);
                std::fs::remove_file(&path).unwrap();
            }
        }

        thread::sleep(Duration::new(15, 0));
    }
}

pub async fn run(file_config: FileSyncConfig) {
    let file_sync = Arc::new(FileSync::new(file_config));
    task::spawn(async move { source_path_listener(Arc::clone(&file_sync)).await });
}
