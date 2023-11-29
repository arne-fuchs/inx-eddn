use std::io;
use std::io::{Read, Write};
use std::sync::mpsc::Sender;
use std::time::Duration;

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

pub struct EddnAdapter {
    pub hornet_bus: Sender<Vec<u8>>
}

impl EddnAdapter {
    pub async fn subscribe_to_eddn(self) {
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        subscriber.connect(std::env::var("ZEROMQ_URL").unwrap().as_str()).expect("Failed to connect to ZeroMQ Server");
        subscriber.set_subscribe(b"").expect("Failed to subscribe to channel");

        tokio::time::sleep(Duration::from_millis(1)).await;

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


            let result = self.hornet_bus.send(blob);
            match result {
                Ok(_) => {}
                Err(err) => {
                    println!("{}",err);
                }
            }
        }
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

