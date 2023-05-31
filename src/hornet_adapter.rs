use std::collections::VecDeque;
use std::sync::mpsc::RecvError;
use std::thread;
use std::time::{Duration, Instant};
use bus::BusReader;
use dotenv::dotenv;
use iota_wallet::account::{Account, AccountHandle};
use iota_wallet::iota_client::block::{Block, BlockId};
use iota_wallet::iota_client::Client;
use iota_wallet::iota_client::constants::SHIMMER_COIN_TYPE;
use iota_wallet::iota_client::crypto::ciphers::aes_kw::BLOCK;
use json::JsonValue;
use serde_json::{json, Value};
use crate::hornet_adapter;

pub struct Hornet {
    pub node: Client,
    pub account: AccountHandle,
    pub messages: Vec<IRC27>,
    pub bus_reader: BusReader<JsonValue>,
}

impl Hornet {
    pub fn attach(&mut self, json: JsonValue) {
        let epsilon = 20;
        let open_bracket = "[".as_bytes();
        let close_bracket = "]".as_bytes();
        let comma = ",".as_bytes();

        let irc27 = generate_irc27_from_json(json);

        let mut byte_length = open_bracket.len() + close_bracket.len() + comma.len();

        for message in &self.messages {
            byte_length = byte_length + message.get_json().to_string().as_bytes().len();
        }

        byte_length = byte_length + irc27.get_json().to_string().as_bytes().len();

        //If new byte length is larger then current, send current messages into the tangle
        if byte_length > Block::LENGTH_MAX - epsilon {
            let mut messages = self.messages.clone();
            self.messages = Vec::new();
            let node = self.node.clone();

            //If current message fits into block (no big market data)
            if irc27.get_json().to_string().as_bytes().len() + open_bracket.len() + close_bracket.len() < Block::LENGTH_MAX - epsilon {
                thread::spawn(move || {
                    let mut data: Vec<u8> = vec![];

                    open_bracket.iter().for_each(|byte| {
                        data.push(byte.clone());
                    });
                    let mut json_string = messages.get(0).unwrap().get_json().to_string();
                    let mut json_bytes = json_string.as_bytes();
                    let mut is_first = true;

                    while messages.len() > 0 && data.len() + json_bytes.len() + ",".as_bytes().len() + "]".as_bytes().len() < Block::LENGTH_MAX - epsilon {
                        if !is_first {
                            comma.iter().for_each(|byte| {
                                data.push(byte.clone());
                            });
                        } else {
                            is_first = false;
                        }
                        let popped_json_string = messages.pop().unwrap().get_json().to_string();
                        let byte_array = popped_json_string.as_bytes();
                        for byte in byte_array {
                            data.push(byte.clone())
                        }
                    }

                    if !is_first {
                        close_bracket.iter().for_each(|byte| {
                            data.push(byte.clone());
                        });

                        let thread_data = data.clone();
                        let thread_node = node.clone();
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(async move {
                                let result = thread_node.block()
                                    .with_tag("EDCAS".as_bytes().to_vec())
                                    .with_data(thread_data)
                                    .finish()
                                    .await;


                                match result {
                                    Ok(block) => {
                                        println!("Block send: {}", block.id())
                                    }
                                    Err(err) => {
                                        println!("Couldn't send block chunk: {:?}", err)
                                    }
                                }
                            });
                    }
                });
            } else {
                let json_string = messages.get(0).unwrap().get_json().to_string();
                let json_bytes = json_string.as_bytes();

                let chunks = json_bytes.chunks(Block::LENGTH_MAX - epsilon);
                for chunk in chunks {
                    let thread_node_blocks = node.clone();
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap()
                        .block_on(async move {
                            let result = thread_node_blocks.block()
                                .with_tag("EDCAS Blob".as_bytes().to_vec())
                                .with_data(Vec::from(chunk))
                                .finish()
                                .await;


                            match result {
                                Ok(block) => {
                                    println!("Block market send: {}", block.id())
                                }
                                Err(err) => {
                                    println!("Couldn't send block market: {:?}", err)
                                }
                            }
                        });
                }
            }
        }
        self.messages.push(irc27);
    }
    pub fn read(&mut self) {
        let json_result = self.bus_reader.recv();
        match json_result {
            Ok(json) => {
                self.attach(json);
                println!("Queue size: {}",  self.messages.len());
            }
            Err(err) => {
                println!("Rec. error: {}", err);
            }
        }
    }
}


// https://docs.opensea.io/docs/metadata-standards
#[derive(Clone)]
pub struct IRC27 {
    standard: String,
    version: String,
    description: String,
    media_type: String,
    uri: String,
    name: String,
    collection_name: String,
    attributes: Vec<Attribute>,
}

#[derive(Clone)]
pub struct Attribute {
    trait_type: String,
    value: String,
}

impl IRC27 {
    fn get_json(&self) -> Value {
        let Self {
            standard, version, description, media_type, uri: image, name, collection_name: collection, attributes
        } = self;
        let mut array = json!([]);
        for attribute in attributes {
            array.as_array_mut().unwrap().push(attribute.get_json());
        }
        json!(
            {
                "standard": standard,
                "version": version,
                "description": description,
                "type": media_type,
                "uri": image,
                "name": name,
                "collectionName": collection,
                "attributes": array
            }
        )
    }
}

impl Attribute {
    fn get_json(&self) -> Value {
        let Self {
            trait_type, value
        } = self;
        json!(
            {
                "trait_type": trait_type,
                "value": value
            }
        )
    }
}

fn generate_irc27_from_json(json: JsonValue) -> IRC27 {
    let mut attributes: Vec<Attribute> = vec![];
    let json_message = json["message"].clone();
    for entry in json_message.entries() {
        attributes.push(
            Attribute {
                trait_type: entry.0.to_string(),
                value: entry.1.to_string(),
            }
        );
    }

    let mut name = json["BodyName"].to_string();
    if name == "null" {
        name = json["StarSystem"].to_string();
    }

    IRC27 {
        standard: "IRC27".to_string(),
        version: "v1.0".to_string(),
        description: "".to_string(),
        media_type: "image/png".to_string(),
        uri: "https://edassets.org/static/img/ed-logos/elite-dangerous-minimalistic.png".to_string(),
        name,
        collection_name: json["StarSystem"].to_string(),
        attributes,
    }
}