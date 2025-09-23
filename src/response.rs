use std::{collections::HashMap, time::SystemTime};

use super::{handler::StoreItem, redis_serialize, RedisData};

pub fn respond(
    req: RedisData,
    store: &mut HashMap<String, StoreItem>,
    list_store: &mut HashMap<String, Vec<RedisData>>,
) -> String {
    if req.parses_to_string(Some("ping")) {
        return handle_ping();
    }

    if req.is_arr() {
        let arr = req.as_arr();

        match arr {
            Ok(data) => {
                let cmd = &data[0];

                if cmd.parses_to_string(Some("ping")) {
                    return handle_ping();
                } else if data.len() >= 2 {
                    if cmd.is_bulk_string(Some("echo")) {
                        return handle_echo(data);
                    } else if cmd.is_bulk_string(Some("set")) {
                        return handle_set(data, store);
                    } else if cmd.is_bulk_string(Some("get")) {
                        return handle_get(data, store);
                    } else if cmd.is_bulk_string(Some("rpush")) {
                        return handle_rpush(data, list_store);
                    } else if cmd.is_bulk_string(Some("lrange")) {
                        return handle_lrange(data, list_store);
                    }
                }
            }
            Err(_) => {
                return redis_serialize(&RedisData::Error(String::from("Error matching array")));
            }
        }
    }
    return redis_serialize(&RedisData::Error(String::from(
        "Error responding to request",
    )));
}

/* Helpers */

fn handle_ping() -> String {
    String::from("+PONG\r\n")
}

fn handle_echo(data: &Vec<RedisData>) -> String {
    redis_serialize(&data[1])
}

fn handle_set(data: &Vec<RedisData>, store: &mut HashMap<String, StoreItem>) -> String {
    if data.len() < 3 {
        return redis_serialize(&RedisData::Error(String::from(
            "Error setting key value pair",
        )));
    }

    let key = data[1].as_string().unwrap_or(String::from(""));
    let value = data[2].as_string().unwrap_or(String::from(""));

    let can_expire = data.len() >= 5 && data[3].parses_to_string(Some("px"));

    if can_expire {
        let expiry = data[4].as_string().unwrap_or(String::from("0"));

        store.insert(
            key,
            StoreItem {
                value,
                created_at: Some(SystemTime::now()),
                expires_after: Some(expiry),
            },
        );
    } else {
        store.insert(
            key,
            StoreItem {
                value,
                created_at: None,
                expires_after: None,
            },
        );
    };

    return String::from("+OK\r\n");
}

fn handle_get(data: &Vec<RedisData>, store: &mut HashMap<String, StoreItem>) -> String {
    let key = data[1].as_string().unwrap_or(String::from(""));
    let value = store.get(&key);

    let data = match value {
        Some(val) => {
            let StoreItem {
                value,
                created_at,
                expires_after,
            } = val;

            if let Some(expired_at) = expires_after {
                match created_at.unwrap().elapsed() {
                    Ok(time) => {
                        if time.as_millis() >= expired_at.parse::<u128>().unwrap() {
                            RedisData::NullBulkString(())
                        } else {
                            RedisData::BulkString(value.to_string())
                        }
                    }
                    _ => RedisData::NullBulkString(()),
                }
            } else {
                RedisData::BulkString(value.to_string())
            }
        }

        None => RedisData::NullBulkString(()),
    };

    return redis_serialize(&data);
}

fn handle_rpush(data: &Vec<RedisData>, list_store: &mut HashMap<String, Vec<RedisData>>) -> String {
    if data.len() < 4 {
        return redis_serialize(&RedisData::Error(String::from(
            "Invalid number of arguments",
        )));
    }

    let key = data[1].as_string().unwrap_or(String::from(""));
    let value = data[2..].to_vec();

    let default = vec![];
    let mut existing_list = list_store.get(&key).unwrap_or(&default).to_vec();

    existing_list.extend(value);
    let length = existing_list.len();

    let new_list = existing_list;

    list_store.insert(key, new_list);

    return redis_serialize(&RedisData::Int(length.try_into().unwrap_or(0)));
}

fn handle_lrange(
    data: &Vec<RedisData>,
    list_store: &mut HashMap<String, Vec<RedisData>>,
) -> String {
    if data.len() != 4 {
        return redis_serialize(&RedisData::Error(String::from(
            "Invalid number of arguments",
        )));
    }

    let key = data[1].as_string().unwrap_or(String::from(""));
    let parsed_start_index = data[2].as_int().unwrap_or(&0);
    let parsed_stop_index = data[3].as_int().unwrap_or(&0);

    let default = vec![];

    let existing_list = list_store.get(&key).unwrap_or(&default);
    let list_length = existing_list.len();

    let unsigned_start = parsed_start_index.unsigned_abs();
    let unsigned_end = parsed_stop_index.unsigned_abs();

    let start_index = if parsed_start_index.is_positive() {
        if unsigned_start >= list_length {
            list_length - 1
        } else {
            unsigned_start
        }
    } else {
        if unsigned_start >= list_length {
            0
        } else {
            list_length - unsigned_start
        }
    };

    let stop_index = if parsed_stop_index.is_positive() {
        if unsigned_end >= list_length {
            list_length - 1
        } else {
            unsigned_end
        }
    } else {
        if unsigned_end >= list_length {
            0
        } else {
            list_length - unsigned_end
        }
    };

    if existing_list.is_empty() || start_index >= existing_list.len() || start_index > stop_index {
        return redis_serialize(&RedisData::Array(vec![]));
    }

    let slice = &existing_list[start_index..stop_index + 1];

    return redis_serialize(&RedisData::Array(slice.to_vec()));
}
