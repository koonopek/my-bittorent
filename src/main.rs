use std::env;

use serde_json::{self, json};

type BenCodedValue = String | i32;

fn decode_bencoded_value(encoded_value: &str) -> Result<BenCodedValue, &str> {
    // If encoded_value starts with a digit, it's a number
    match encoded_value.split_once(":") {
        Some((count, value)) => {
            if value.len()
                != count
                    .parse::<usize>()
                    .expect("Supplied count cant be parsed to in")
            {
                return Err("Length in string missmatched");
            }
            return Ok(value.to_string());
        }
        None => {
            let mut chars = encoded_value.chars();

            match chars.next() {
                Some('i') => {
                    return Ok(serde_json::Value::Number(
                        chars
                            .take_while(|c| c == &'e')
                            .collect::<String>()
                            .parse::<i32>()
                            .unwrap(),
                    ))
                }
                _ => return Err(""),
            }
        }
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.unwrap().to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
