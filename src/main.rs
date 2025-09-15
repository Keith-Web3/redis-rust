#![allow(unused_imports)]
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use regex::Regex;

#[derive(Debug)]
enum RedisData {
    String(String),
    Int(isize),
    Error(String),
    BulkString(String),
    NullBulkString(()),
    Array(Vec<RedisData>),
    Null(()),
}

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

                            match parsed_data {
                                RedisData::String(val) => {
                                    if val == String::from("ping") {
                                        stream_result.write_all(b"+PONG\r\n").unwrap()
                                    }
                                }
                                RedisData::BulkString(val) => {
                                    if val == String::from("ping") {
                                        stream_result.write_all(b"+PONG\r\n").unwrap()
                                    }
                                }
                                RedisData::Array(val) => {
                                    if let RedisData::BulkString(cmd) = &val[0] {
                                        if cmd == &String::from("echo") {
                                            if let RedisData::BulkString(arg) = &val[1] {
                                                stream_result
                                                    .write_all(
                                                        format!("${}\r\n{}\r\n", arg.len(), arg)
                                                            .as_bytes(),
                                                    )
                                                    .unwrap();
                                            } else {
                                                println!("Invalid argument");
                                            }
                                        } else if cmd == &String::from("ping") {
                                            stream_result.write_all(b"+PONG\r\n").unwrap()
                                        } else {
                                            print!("Invalid command");
                                        }
                                    };
                                }
                                _ => println!("type not handled"),
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

fn redis_parse(command: String) -> RedisData {
    if command.starts_with("+") {
        return RedisData::String(command[1..].to_lowercase().split("\r\n").collect());
    } else if command.starts_with("-") {
        return RedisData::Error(command[1..].to_lowercase().split("\r\n").collect());
    } else if command.starts_with(":") {
        let cmd_string = command[1..].split("\r\n").collect::<String>();
        let int_value = cmd_string.parse::<isize>();

        match int_value {
            Ok(val) => return RedisData::Int(val),
            Err(_) => println!("Invalid integer"),
        }
    } else if command.starts_with("$") {
        let bulk_string_regex = Regex::new(r"^\$([0-9]+)\r\n(.+)\r\n$");

        match bulk_string_regex {
            Ok(re) => {
                if let Some(bulk_string) = re.captures(&command) {
                    let (_, [length, value]) = bulk_string.extract();

                    let parsed_length = length.parse::<usize>().unwrap_or(0);
                    if parsed_length != value.len() {
                        println!("Invalid bulk string, mismatching lengths")
                    } else {
                        return RedisData::BulkString(value.to_lowercase());
                    }
                } else {
                    println!("Invalid bulk string, invalid pattern");
                }
            }
            Err(_) => {
                println!("Invalid bulk regex");
            }
        }

        let null_bulk_regex = Regex::new(r"^\$-1\r\n$");

        match null_bulk_regex {
            Ok(re) => {
                if re.is_match(&command) {
                    return RedisData::NullBulkString(());
                }
            }
            Err(_) => {}
        }
    } else if command.starts_with("*") {
        let mut prev_value = "";
        let mut recombined = vec![];
        let mut index = 0;
        let mut length: usize = 0;

        command
            .split("\r\n")
            .filter(|cmd| !cmd.is_empty())
            .for_each(|cmd| {
                println!("command: {}", cmd);
                if index == 0 {
                    match cmd[1..].parse::<usize>() {
                        Ok(val) => length = val,
                        Err(_) => {
                            println!("length is invalid");
                            return;
                        }
                    }
                } else if prev_value.starts_with("$") {
                    recombined.push(format!("{}\r\n{}\r\n", prev_value, cmd))
                } else if !cmd.starts_with("$") {
                    recombined.push(format!("{}\r\n", cmd));
                }
                prev_value = cmd;
                index += 1;
            });

        println!("recombined: {:?}", recombined);
        let final_arr = recombined
            .into_iter()
            .map(|cmd| redis_parse(cmd))
            .collect::<Vec<RedisData>>();

        return RedisData::Array(final_arr);
    }
    RedisData::Null(())
}
