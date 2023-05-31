use std::collections::VecDeque;
use std::io::Write;
use std::sync::mpsc::RecvError;
use std::thread;
use std::time::{Duration, Instant};

use bus::BusReader;
use dotenv::dotenv;
use flate2::Compression;
use flate2::write::ZlibEncoder;
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
    pub blobs: Vec<Vec<u8>>,
    pub bus_reader: BusReader<Vec<u8>>,
}

impl Hornet {
    pub fn attach(&mut self, blob: Vec<u8>) {
        let epsilon = 200;
        let open_bracket = "[".as_bytes();
        let close_bracket = "]".as_bytes();
        let comma = ",".as_bytes();

        let mut byte_length = open_bracket.len() + close_bracket.len() + comma.len();

        for blob in &self.blobs {
            byte_length = byte_length + blob.len();
        }

        byte_length = byte_length + blob.len();

        //If new byte length is larger then current, send current messages into the tangle
        if byte_length > Block::LENGTH_MAX - epsilon {
            let mut blobs = self.blobs.clone();
            self.blobs = Vec::new();
            let node = self.node.clone();

            //If current message fits into block (no big market data)
            if blob.len() + open_bracket.len() + close_bracket.len() < Block::LENGTH_MAX - epsilon {

                    let mut data: Vec<u8> = vec![];

                    open_bracket.iter().for_each(|byte| {
                        data.push(byte.clone());
                    });
                    let blob = blobs.get(0).unwrap().clone();
                    let mut is_first = true;

                    while blobs.len() > 0 && data.len() + blob.len() + ",".as_bytes().len() + "]".as_bytes().len() < Block::LENGTH_MAX - epsilon {
                        if !is_first {
                            comma.iter().for_each(|byte| {
                                data.push(byte.clone());
                            });
                        } else {
                            is_first = false;
                        }
                        let popped_blob = blobs.pop().unwrap();
                        for byte in popped_blob {
                            data.push(byte.clone())
                        }
                    }

                    if !is_first {
                        close_bracket.iter().for_each(|byte| {
                            data.push(byte.clone());
                        });

                        let thread_data = data.clone();
                        let thread_node = node.clone();
                        let now = Instant::now();
                        println!("Attaching block...");
                        tokio::runtime::Builder::new_multi_thread()
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
                                        println!("Block send: {}", block.id());
                                        println!("Took {} seconds", now.elapsed().as_secs());
                                    }
                                    Err(err) => {
                                        println!("Couldn't send block chunk: {:?}", err)
                                    }
                                }
                            });
                    }

            } else {
                let blob = blobs.get(0).unwrap();

                let chunks = blob.chunks(Block::LENGTH_MAX - epsilon);
                for chunk in chunks {
                    let thread_node_blocks = node.clone();
                    tokio::runtime::Builder::new_multi_thread()
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
        self.blobs.push(blob);
    }
    pub fn read(&mut self) {

        let result = self.bus_reader.recv();
        match result {
            Ok(blob) => {
                self.attach(blob);
                //println!("Queue size: {}",  self.blobs.len());
            }
            Err(err) => {
                println!("Rec. error: {}", err);
            }
        }
    }
}
