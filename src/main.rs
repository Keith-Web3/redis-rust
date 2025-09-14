#![allow(unused_imports)]
use std::net::TcpListener;
use std::io::{Write, Read};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        let mut buffer = [0; 1024]; // A buffer to store incoming data   

        match stream {
            Ok(mut stream_result) => {
                println!("accepted new connection");
                loop {
                    let data = stream_result.read(&mut buffer);
                    println!("attempting to respond");
                    match data{
                        Ok (0) => {
                            break;
                        }
                        Ok(_) => {
                            println!("responded successfully");
                            stream_result.write_all(b"+PONG\r\n").unwrap();
                        },
                        Err(_) => {
                            println!("response failed");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
