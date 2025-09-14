#![allow(unused_imports)]
use std::net::TcpListener;
use std::io::Write;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream_result) => {
                println!("accepted new connection");
                loop {
                    println!("attempting to respond");
                    match stream_result.write_all(b"+PONG\r\n") {
                        Ok(_) => println!("responded successfully"),
                        Err(_) => println!("response failed")
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
