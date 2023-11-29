use std::io;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::time::Duration;

use bus::Bus;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use tokio::time::Instant;

pub struct EddnAdapter {
    pub hornet_bus: Bus<Vec<u8>>,
    pub queue: VecDeque<Vec<u8>>,
    pub timestamp: Instant,
}

impl EddnAdapter {
    pub async fn subscribe_to_eddn(mut self){

        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        subscriber.connect(std::env::var("ZEROMQ_URL").unwrap().as_str()).expect("Failed to connect to ZeroMQ Server");
        subscriber.set_subscribe(b"").expect("Failed to subscribe to channel");

        tokio::time::sleep(Duration::from_millis(1)).await;

        let mut update_nbr : u32 = 0;
        loop {

            let data = subscriber
                .recv_bytes(0)
                .expect("Failed receiving update");

            let message = decode_reader(data).unwrap();

            if message == "END" {
                break;
            }

            let mut blob = Vec::new();
            // Create a ZlibEncoder and write the compressed data to the buffer
            let mut encoder = ZlibEncoder::new(&mut blob, Compression::best());
            encoder.write_all(message.as_bytes()).unwrap();
            encoder.finish().unwrap();

            self.queue.push_back(blob);
            loop{
                if !self.queue.is_empty(){
                    let result = self.hornet_bus.try_broadcast(self.queue.pop_front().unwrap());
                    match result {
                        Ok(_) => {}
                        Err(blob) => {
                            self.queue.push_front(blob);
                            if self.queue.len() % 100 == 0{
                                println!("------------------------------------------");
                                println!("Queue size: {}", self.queue.len());
                                println!("Updates per second: {}",100.0/self.timestamp.elapsed().as_secs() as f64);
                                println!("------------------------------------------");
                                self.timestamp = Instant::now();
                            }
                            break;
                        }
                    }
                }else {
                    break;
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

