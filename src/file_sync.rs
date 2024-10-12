use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
pub struct FileSyncConfig {
    source_path: String,
    destination_path: String,
}

fn source_path_listener(source_path: String) {
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
                // Optionally read the file or do something with it
            }
        }

        thread::sleep(Duration::new(15, 0));
    }
}

pub fn run(file_config: FileSyncConfig) {
    thread::spawn(move || source_path_listener(file_config.source_path));
    println!("Destination path: {}", file_config.destination_path);
}
