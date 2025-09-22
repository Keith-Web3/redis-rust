use std::{net::TcpListener, thread};

use super::handler;

pub fn server() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let mut threads = vec![];

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let stream_thread = thread::spawn(|| {
                    handler::handle_connection(stream);
                });

                threads.push(stream_thread);
            }
            Err(e) => {
                println!("connection error: {}", e);
            }
        }
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
