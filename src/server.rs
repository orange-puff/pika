use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerConfig {
    enabled: bool,
    host: String,
    port: u16,
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    while match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            // Echo the data back to the client
            stream.write(&buffer[0..size]).unwrap();
            true
        }
        Ok(_) => false,
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            false
        }
    } {}
}

pub fn run(server_config: ServerConfig) {
    if !server_config.enabled {
        println!("Server is not enabled");
        return;
    }

    let addr = format!("{}:{}", server_config.host, server_config.port);
    println!("Server binding to {}", addr);

    let listener = TcpListener::bind(addr);
    if let Ok(listener) = listener {
        println!("listening...");
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    thread::spawn(move || handle_client(stream));
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    } else if let Err(e) = listener {
        println!("Error: {}", e);
        return;
    }
}
