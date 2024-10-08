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
    source_s3_prefix: String,
    destination_s3_prefix: String,
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

async fn destination_path_listener(file_sync: Arc<FileSync>) {
    println!("Destination path listener started");
    println!(
        "Downloading files from S3 bucket {} to {}",
        file_sync.file_sync_config.bucket_name, file_sync.file_sync_config.destination_path
    );

    let dest_path = Path::new(&file_sync.file_sync_config.destination_path);
    if let Err(e) = fs::create_dir_all(dest_path) {
        eprintln!("Failed to create directory: {}", e);
        return ();
    }

    loop {
        match file_sync
            .aws_client
            .list_objects_v2()
            .bucket(&file_sync.file_sync_config.bucket_name)
            .send()
            .await
        {
            Ok(files) => match files.contents() {
                Some(contents) => {
                    for file in contents {
                        if let Some(key) = file.key() {
                            if key.starts_with(&file_sync.file_sync_config.destination_s3_prefix) {
                                println!("Downloading file: {}", key);

                                let get_object_output = match file_sync
                                    .aws_client
                                    .get_object()
                                    .bucket(&file_sync.file_sync_config.bucket_name)
                                    .key(key)
                                    .send()
                                    .await
                                {
                                    Ok(output) => output,
                                    Err(e) => {
                                        eprintln!("Failed to get object {}: {}", key, e);
                                        continue;
                                    }
                                };

                                let local_path =
                                    Path::new(&file_sync.file_sync_config.destination_path).join(
                                        key.replace(
                                            &file_sync.file_sync_config.destination_s3_prefix,
                                            "",
                                        ),
                                    );

                                let bytes = match get_object_output.body.collect().await {
                                    Ok(bytes) => bytes.into_bytes(),
                                    Err(e) => {
                                        eprintln!("Failed to collect bytes for {}: {}", key, e);
                                        continue;
                                    }
                                };
                                // Write the bytes to file
                                if let Err(e) = fs::write(&local_path, bytes) {
                                    eprintln!(
                                        "Failed to write file {}: {}",
                                        local_path.display(),
                                        e
                                    );
                                    continue;
                                }

                                // delete file from S3
                                if let Err(e) = file_sync
                                    .aws_client
                                    .delete_object()
                                    .bucket(&file_sync.file_sync_config.bucket_name)
                                    .key(key)
                                    .send()
                                    .await
                                {
                                    eprintln!("Failed to delete object {}: {}", key, e);
                                    continue;
                                }
                            }
                        }
                    }
                }
                None => {}
            },
            Err(e) => {
                eprintln!("Failed to list objects: {}", e);
            }
        }
        thread::sleep(Duration::new(15, 0));
    }
}

async fn source_path_listener(file_sync: Arc<FileSync>) {
    println!("Source path listener started");
    println!(
        "Looking for files in {} and uploading to S3 bucket {}",
        file_sync.file_sync_config.source_path, file_sync.file_sync_config.bucket_name
    );

    let src_path = Path::new(&file_sync.file_sync_config.source_path);
    if let Err(e) = fs::create_dir_all(src_path) {
        eprintln!("Failed to create directory: {}", e);
        return ();
    }

    loop {
        let entries = match fs::read_dir(src_path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Failed to read directory: {}", e);
                thread::sleep(Duration::new(15, 0));
                continue;
            }
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            if path.is_file() {
                println!("Found file: {:?}.", path);

                let mut file = match File::open(&path) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("Could not open file {:?}: {}", path, e);
                        continue;
                    }
                };

                let mut buffer = Vec::new();
                if let Err(e) = file.read_to_end(&mut buffer) {
                    eprintln!("Failed to read file {:?}: {}", path, e);
                    continue;
                }

                let byte_stream = ByteStream::from(buffer);

                // |n| n.to_str() is a closure where n is the arg
                let file_name = file_sync.file_sync_config.source_s3_prefix.clone()
                    + match path.file_name().and_then(|n| n.to_str()) {
                        Some(name) => name,
                        None => {
                            eprintln!("Invalid file name: {:?}", path);
                            continue;
                        }
                    };

                println!(
                    "Sending {} to S3 bucket {}...",
                    file_name, file_sync.file_sync_config.bucket_name
                );

                if let Err(e) = file_sync
                    .aws_client
                    .put_object()
                    .bucket(&file_sync.file_sync_config.bucket_name)
                    .key(file_name)
                    .body(byte_stream)
                    .send()
                    .await
                {
                    eprintln!("Failed to upload file to S3: {}", e);
                    continue;
                }

                println!("Removing file {:?}...", path);
                if let Err(e) = std::fs::remove_file(&path) {
                    eprintln!("Failed to remove file {:?}: {}", path, e);
                }
            }
        }

        thread::sleep(Duration::new(15, 0));
    }
}

pub async fn run(file_config: FileSyncConfig) {
    let file_sync = Arc::new(FileSync::new(file_config));

    let file_sync_for_source = Arc::clone(&file_sync);
    let file_sync_for_destination = Arc::clone(&file_sync);

    task::spawn(async move { source_path_listener(file_sync_for_source).await });
    task::spawn(async move { destination_path_listener(file_sync_for_destination).await });
}
