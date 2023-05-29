use std::thread;
use bus::Bus;
use dotenv::dotenv;
use iota_wallet::Error;
use json::JsonValue;
use tokio::macros::support::thread_rng_n;
use crate::hornet_adapter::Hornet;
use crate::eddn_adapter::EddnAdapter;

mod hornet_adapter;
mod eddn_adapter;

fn main(){
    dotenv().expect(".env file not found");

    let mut hornet_bus: Bus<JsonValue> = Bus::new(100);
    let bus_reader = hornet_bus.add_rx();

    let node = hornet_adapter::connect_to_node(std::env::var("NODE_URL").unwrap());
    let mut hornet = Hornet {
        node,
        messages: Vec::new(),
        bus_reader,
    };
    let eddn = EddnAdapter{
        hornet_bus
    };
    thread::spawn(move ||{
        loop{
            hornet.read();
        }
    });
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            eddn.subscribe_to_eddn().await;
        });
}