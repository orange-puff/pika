use anyhow::Result;
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
use walkdir::WalkDir;
use zip::write::FileOptions;

#[derive(Deserialize)]
pub struct FileSyncConfig {
    s3_path: String,
    bucket_name: String,
    s3_prefix: String,
    destination_path: String,
}

fn unzip_file<P: AsRef<Path>>(src_file: P, dst_dir: P) -> Result<()> {
    println!(
        "Unzipping file: {} to directory {}",
        src_file.as_ref().display(),
        dst_dir.as_ref().display()
    );

    let file = File::open(src_file)?;
    let mut archive = zip::ZipArchive::new(file)?;

    if let Err(e) = archive.extract(dst_dir) {
        eprintln!("Failed to extract file: {}", e);
        return Err(anyhow::anyhow!("Failed to extract file: {}", e));
    }

    Ok(())
}

fn zip_directory<P: AsRef<Path>>(src_dir: P, dst_file: P) -> Result<()> {
    println!(
        "Zipping directory: {} to file {}",
        src_dir.as_ref().display(),
        dst_file.as_ref().display()
    );
    let file = File::create(dst_file)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Traverse the source directory
    let src_path = src_dir.as_ref();
    for entry in WalkDir::new(src_path) {
        let entry = entry?;
        let path = entry.path();

        // Skip if the path is the source directory itself
        if path == src_path {
            continue;
        }

        // Create relative path for the zip file
        let relative_path = path.strip_prefix(src_path)?;

        if path.is_file() {
            // Add file to zip
            zip.start_file(relative_path.to_string_lossy(), options)?;
            let mut file = File::open(path)?;
            std::io::copy(&mut file, &mut zip)?;
        } else if path.is_dir() {
            // Add directory to zip
            zip.add_directory(relative_path.to_string_lossy(), options)?;
        }
    }

    zip.finish()?;
    Ok(())
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
        let mut source_path = path.to_path_buf();

        let mut is_dir = false;
        if path.is_dir() {
            is_dir = true;
            let zip_path = path.with_extension("zip");

            if let Err(e) = zip_directory(path, &zip_path) {
                eprintln!("Failed to zip directory {}: {}", path.display(), e);
                return 1;
            }

            source_path = zip_path;
        }

        let upload_file_name = self.file_sync_config.s3_prefix.clone()
            + source_path.file_name().unwrap().to_str().unwrap();

        let mut file = match File::open(&source_path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Could not open file {:?}: {}", path, e);
                return 1;
            }
        };

        let mut buffer = Vec::new();
        if let Err(e) = file.read_to_end(&mut buffer) {
            eprintln!("Failed to read file {:?}: {}", source_path, e);
            return 1;
        }

        let byte_stream = ByteStream::from(buffer);

        if let Err(e) = self
            .aws_client
            .put_object()
            .bucket(&self.file_sync_config.bucket_name)
            .key(&upload_file_name)
            .body(byte_stream)
            .send()
            .await
        {
            eprintln!("Failed to upload file to S3: {}", e);
            return 1;
        }

        if is_dir {
            fs::remove_file(&source_path).unwrap();
        }

        println!(
            "Uploaded file {} to S3 bucket {}",
            upload_file_name, self.file_sync_config.bucket_name
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

                                let ext = local_path.extension();
                                if ext.is_some() && ext.unwrap() == "zip" {
                                    if let Err(e) = unzip_file(
                                        &local_path,
                                        &dest_path.join(local_path.file_stem().unwrap()),
                                    ) {
                                        eprintln!("Failed to unzip file {}: {}", key, e);
                                        return 1;
                                    }

                                    fs::remove_file(&local_path).unwrap();
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
