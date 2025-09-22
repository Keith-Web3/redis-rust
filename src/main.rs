#![allow(unused_imports)]

mod handler;
mod redis;
mod response;
mod server;
mod utils;

use redis::parser::RedisData;
use redis::utils::{redis_parse, redis_serialize};

use server::server;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    server();
}
