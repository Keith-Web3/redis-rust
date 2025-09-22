use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    time::SystemTime,
};

use crate::redis::parser::RedisData;

use super::{redis_parse, response::respond};

pub struct StoreItem {
    pub value: String,
    pub created_at: Option<SystemTime>,
    pub expires_after: Option<String>,
}

pub fn handle_connection(mut connection: TcpStream) {
    let mut store = HashMap::<String, StoreItem>::new();
    let mut list_store = HashMap::<String, Vec<String>>::new();

    let mut buffer = [0; 1000];

    loop {
        let request = connection.read(&mut buffer);

        match request {
            Ok(0) => {
                break;
            }
            Ok(byte_count) => {
                let data = String::from_utf8_lossy(&buffer[..byte_count]).to_string();
                let parsed_data = redis_parse(data);

                let response = respond(parsed_data, &mut store, &mut list_store);

                let final_response = connection.write_all(response.as_bytes());

                match final_response {
                    Ok(_) => println!("responded successfully"),
                    Err(e) => println!("An error occurred while sending a response: {}", e),
                }
            }
            Err(e) => {
                println!("Error parsing connection data: {}", e);
            }
        }
    }
}
