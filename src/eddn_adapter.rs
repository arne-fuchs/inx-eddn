use std::{io, thread};
use std::io::Read;
use std::time::Duration;
use bus::Bus;

use dotenv::dotenv;
use flate2::read::ZlibDecoder;
use json::JsonValue;
use serde_json::{json, Value};
use crate::hornet_adapter::Hornet;

pub struct EddnAdapter {
    pub hornet_bus: Bus<JsonValue>
}

impl EddnAdapter {
    pub async fn subscribe_to_eddn(mut self){
        dotenv().expect(".env file not found");

        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        subscriber.connect(std::env::var("ZEROMQ_URL").unwrap().as_str()).expect("Failed to connect to ZeroMQ Server");
        subscriber.set_subscribe(b"").expect("Failed to subscribe to channel");

        thread::sleep(Duration::from_millis(1));

        let mut update_nbr : u32 = 0;
        loop {
            let data = subscriber
                .recv_bytes(0)
                .expect("Failed receiving update");

            let message = decode_reader(data).unwrap();

            if message == "END" {
                break;
            }

            let json_result = json::parse(message.as_str());
            match json_result {
                Ok(json) => {
                    let mut tag = "UNCATEGORIZED".to_string();
                    let value = &json["message"].to_string();
                    if &json["message"]["event"].to_string() != "null" {
                        tag = json["message"]["event"].to_string();
                    }else {
                        if &json["message"]["stationName"].to_string() != "null" {
                            tag = "Station".to_string();
                        }else {
                            println!("UNCATEGORIZED: {}", value.clone())
                        }
                    }
                    self.hornet_bus.broadcast(json);
                }
                Err(err) => {
                    println!("{}",err);
                }
            }
            update_nbr += 1;
        }
        println!("Received {} updates", update_nbr);
    }
}

// Uncompresses a Zlib Encoded vector of bytes and returns a string or error
// Here &[u8] implements Read
fn decode_reader(bytes: Vec<u8>) -> io::Result<String> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}