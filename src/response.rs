use std::{collections::HashMap, time::SystemTime};

use super::{handler::StoreItem, redis_serialize, RedisData};

pub fn respond(req: RedisData, store: &mut HashMap<String, StoreItem>) -> String {
    if req.parses_to_string(Some("ping")) {
        return String::from("+PONG\r\n");
    }

    if req.is_arr() {
        let arr = req.as_arr();

        match arr {
            Ok(data) => {
                let cmd = &data[0];

                if cmd.parses_to_string(Some("ping")) {
                    return String::from("+PONG\r\n");
                } else if data.len() >= 2 {
                    if cmd.is_bulk_string(Some("echo")) {
                        return redis_serialize(&data[1]);
                    } else if cmd.is_bulk_string(Some("set")) {
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
                    } else if cmd.is_bulk_string(Some("get")) {
                        let key = data[1].as_string().unwrap_or(String::from("empty"));
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
                                            if time.as_millis()
                                                >= expired_at.parse::<u128>().unwrap()
                                            {
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
