use regex::Regex;

use super::parser::RedisData;

pub fn redis_parse(command: String) -> RedisData {
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

pub fn redis_serialize(data: &RedisData) -> String {
    let result = match data {
        RedisData::String(val) => format!("+{}\r\n", val),
        RedisData::Int(val) => format!(":{}\r\n", val),
        RedisData::Error(val) => format!("-{}\r\n", val),
        RedisData::BulkString(val) => format!("${}\r\n{}\r\n", val.len(), val),
        RedisData::NullBulkString(_) => String::from("$-1\r\n"),
        RedisData::Null(_) => String::from("_\r\n"),
        RedisData::Array(val) => {
            let data = val.iter().map(|el| redis_serialize(el)).collect::<String>();

            format!("*{}\r\n{}", val.len(), data)
        }
    };

    result
}
