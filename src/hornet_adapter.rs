use std::time::Instant;

use bus::BusReader;
use iota_wallet::account::AccountHandle;
use iota_wallet::iota_client::block::Block;
use iota_wallet::iota_client::Client;

pub struct Hornet {
    pub node: Client,
    pub account: AccountHandle,
    pub bus_reader: BusReader<Vec<u8>>,
}

impl Hornet {
    pub fn attach(&mut self, blob: Vec<u8>) {
        let thread_data = blob.clone();
        let thread_node = self.node.clone();
        let now = Instant::now();
        println!("Attaching block...");
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let result = thread_node.block()
                    .with_tag("EDDN".as_bytes().to_vec())
                    .with_data(thread_data)
                    .finish()
                    .await;

                match result {
                    Ok(block) => {
                        println!("------------------------------------------");
                        println!("Block send: {}", block.id());
                        println!("Took {} seconds", now.elapsed().as_secs());
                        println!("------------------------------------------");
                    }
                    Err(err) => {
                        println!("Couldn't send block: {:?}", err)
                    }
                }
            });
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
