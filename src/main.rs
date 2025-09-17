#![allow(unused_imports)]
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Binary;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, SystemTime};

mod redis;
mod utils;

use redis::parser::RedisData;
use redis::utils::{redis_parse, redis_serialize};

// trait Then {
//     fn then<F: Fn(&Self)>(&self, closure: F) {
//         match self {
//             Ok(val) => closure(val)
//         }
//         ;
//     }
// }

// impl<T, E> Then for Result<T, E> {}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let mut handlers = vec![];

    for stream in listener.incoming() {
        let handler = thread::spawn(move || {
            let mut buffer = [0; 1024]; // A buffer to store incoming data
            let mut redis_dictionary = HashMap::new();

            match stream {
                Ok(mut stream_result) => loop {
                    let data = stream_result.read(&mut buffer);
                    println!("attempting to respond");
                    match data {
                        Ok(0) => {
                            break;
                        }
                        Ok(data) => {
                            let msg = String::from_utf8_lossy(&buffer[..data]).to_string();
                            let parsed_data = redis_parse(msg);

                            if parsed_data.parses_to_string(Some("ping"))
                            {
                                utils::then(
                                    stream_result.write_all(b"+PONG\r\n"),
                                    |()| {},
                                    "Error responding to ping",
                                );
                            }

                            if parsed_data.is_arr() {
                                utils::then(
                                    parsed_data.as_arr(),
                                    |arr| {
                                        if arr[0].parses_to_string(Some("ping")) {
                                            utils::then(
                                                stream_result.write_all(b"+PONG\r\n"),
                                                |()| {},
                                                "Error responding to ping",
                                            );
                                        } else if arr.len() >= 2 {
                                            if arr[0].is_bulk_string(Some("echo")) {
                                                utils::then(
                                                    arr[1].as_string(),
                                                    |arg| {
                                                        utils::then(
                                                            stream_result.write_all(
                                                                redis_serialize(
                                                                    &RedisData::BulkString(arg),
                                                                )
                                                                .as_bytes(),
                                                            ),
                                                            |()| {},
                                                            "Error responding to echo",
                                                        )
                                                    },
                                                    "Error converting Redis Data to string",
                                                );
                                            } else if arr[0].is_bulk_string(Some("set")) {
                                                let key =
                                                    arr[1].as_string().unwrap_or(String::from(""));
                                                let value =
                                                    arr[2].as_string().unwrap_or(String::from(""));
                                                let can_expire = arr.len() >= 5 && arr[3].parses_to_string(Some("px"));

                                                if can_expire {
                                                    let expiry = arr[4].as_string().unwrap_or(String::from("0"));

                                                    redis_dictionary.insert(
                                                        key,
                                                        (
                                                            value,
                                                            Some(SystemTime::now()),
                                                            Some(expiry.clone()),
                                                        ),
                                                    );
                                                } else {
                                                    redis_dictionary
                                                        .insert(key, (value, None, None));
                                                };

                                                utils::then(
                                                    stream_result.write_all(b"+OK\r\n"),
                                                    |()| {},
                                                    "Error setting key",
                                                );
                                            } else if arr[0].is_bulk_string(Some("get")) {
                                                utils::then(
                                                    arr[1].as_string(),
                                                    |key| {
                                                        let value = redis_dictionary.get(&key);
                                                        let data = match value {
                                                            Some(val) => {
                                                                let (value, time_stored, expiry) =
                                                                    val;

                                                                    println!("val: {:?}", val);
                                                                if let Some(expired_at) = expiry {
                                                                    match time_stored.unwrap().elapsed() {
                                                                        Ok(time) => { 
                                                                            if time.as_millis() >= expired_at.parse::<u128>().unwrap() {
                                                                                RedisData::NullBulkString(())
                                                                            } else {
                                                                                 RedisData::BulkString(
                                                                value.to_string()
                                                            )
                                                                            }
                                                                        },
                                                                        _ => RedisData::NullBulkString(())
                                                                    }
                                                                } else {
                                                                    RedisData::BulkString(
                                                                        value.to_string(),
                                                                    )
                                                                }
                                                            }

                                                            None => RedisData::NullBulkString(()),
                                                        };

                                                        println!("response: {}", redis_serialize(&data));
                                                        utils::then(
                                                            stream_result.write_all(
                                                                redis_serialize(&data).as_bytes(),
                                                            ),
                                                            |()| {},
                                                            "Error Responding to get",
                                                        );
                                                    },
                                                    "Error converting Redis Data to string",
                                                );
                                            }
                                        }
                                    },
                                    "Error converting RedisData to vector",
                                )
                            }
                        }
                        Err(_) => {
                            println!("response failed");
                            break;
                        }
                    }
                },
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        });
        handlers.push(handler);
    }

    for handler in handlers {
        handler.join().unwrap();
    }
}
