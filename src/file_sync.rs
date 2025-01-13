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

#[derive(Deserialize)]
pub struct FileSyncConfig {
    s3_path: String,
    bucket_name: String,
    s3_prefix: String,
    destination_path: String,
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

    pub async fn upload(&self, path: &Path) -> i32 {
        let file_name =
            self.file_sync_config.s3_prefix.clone() + path.file_name().unwrap().to_str().unwrap();

        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Could not open file {:?}: {}", path, e);
                return 1;
            }
        };

        let mut buffer = Vec::new();
        if let Err(e) = file.read_to_end(&mut buffer) {
            eprintln!("Failed to read file {:?}: {}", path, e);
            return 1;
        }

        let byte_stream = ByteStream::from(buffer);

        if let Err(e) = self
            .aws_client
            .put_object()
            .bucket(&self.file_sync_config.bucket_name)
            .key(&file_name)
            .body(byte_stream)
            .send()
            .await
        {
            eprintln!("Failed to upload file to S3: {}", e);
            return 1;
        }

        println!(
            "Uploaded file {} to S3 bucket {}",
            file_name, self.file_sync_config.bucket_name
        );
        return 0;
    }

    pub async fn download(&self) -> i32 {
        let dest_path = Path::new(&self.file_sync_config.destination_path);
        if let Err(e) = fs::create_dir_all(dest_path) {
            eprintln!("Failed to create directory: {}", e);
            return 1;
        }

        match self
            .aws_client
            .list_objects_v2()
            .bucket(&self.file_sync_config.bucket_name)
            .send()
            .await
        {
            Ok(files) => match files.contents() {
                Some(contents) => {
                    for file in contents {
                        if let Some(key) = file.key() {
                            if !key.starts_with(&self.file_sync_config.s3_prefix) {
                                println!("Downloading file: {}", key);

                                let get_object_output = match self
                                    .aws_client
                                    .get_object()
                                    .bucket(&self.file_sync_config.bucket_name)
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
                                    Path::new(&self.file_sync_config.destination_path).join(key);

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
                                if let Err(e) = self
                                    .aws_client
                                    .delete_object()
                                    .bucket(&self.file_sync_config.bucket_name)
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
        return 0;
    }
}
