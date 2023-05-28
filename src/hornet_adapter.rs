use std::collections::VecDeque;
use std::thread;
use std::time::{Duration, Instant};
use dotenv::dotenv;
use iota_wallet::iota_client::block::{Block, BlockId};
use iota_wallet::iota_client::Client;
use iota_wallet::iota_client::crypto::ciphers::aes_kw::BLOCK;
use json::JsonValue;
use serde_json::{json, Value};

struct Hornet {
    node: Client,
    messages: VecDeque<IRC27>,
    timestamp: Instant
}

impl Default for Hornet{
    fn default() -> Self {
        dotenv().expect(".env file not found");
        let node_url = hornet::connect_to_node(std::env::var("NODE_URL").unwrap());
            
        Hornet{
            node: connect_to_node(node_url),
            messages: VecDeque::new(),
            timestamp: Instant::now()
        }
    }
}

impl Hornet {
    pub fn attach(mut self, data: String){
        let irc27 = generate_irc27_from_json(json::parse(data.as_str()).unwrap());
        self.messages.push(irc27);
        let elapsed = self.timestamp.elapsed();
        if elapsed >= Duration::from_secs(10) {
            self.timestamp == Instant::now();
            //TODO Attach block
            //TODO If "market" block then do not delete timestamp. Market block should be treated differently

            thread::spawn(move ||async move{
                let mut data : Vec<u8> = vec![];

                while self.messages.len() > 0 && data.len() + self.messages.get(0).unwrap() + data.len() < Block::LENGTH_MAX{
                    let byte_array = self.messages.pop_front().unwrap().get_json().as_str().unwrap().as_bytes();
                    for byte in byte_array {
                        data.push(byte.clone())
                    }
                    index = index + 1;
                }

                if self.messages.len() > 0 && self.messages.get(0).unwrap().get_json().as_str().unwrap().as_bytes().len() > Block::LENGTH_MAX{
                    let date = self.messages.get(0).unwrap().get_json().as_str().unwrap().as_bytes();
                    let chunks = data.chunks(Block::LENGTH_MAX - 1);
                    for chunk in chunks {
                        let result = self.node.block()
                            .with_tag("EDCAS".as_bytes().to_vec())
                            .with_data(Vec::from(chunk))
                            .finish().await;


                        match result {
                            Ok(block) => {
                                println!("Block chunk send: {}", block.id())
                            }
                            Err(err) => {
                                println!("Couldn't send block chunk: {:?}", err)
                            }
                        }
                    }
                }

                let result = self.node.block()
                    .with_tag("EDCAS".as_bytes().to_vec())
                    .with_data(data)
                    .finish().await;


                match result {
                    Ok(block) => {
                        println!("Block send: {}", block.id())
                    }
                    Err(err) => {
                        println!("Couldn't send block: {:?}", err)
                    }
                }
            });
        }
    }
}

pub fn connect_to_node(url : String) -> Client {
    Client::builder()
        .with_node(url.as_str()).expect("Unable to connect to node")
        .finish().unwrap()
}

pub async fn send_data_in_blocks(node : &Client, data : &Vec<u8>,tag : String){
    match node.block()
        .with_data(data.to_vec())
        .with_tag(tag.as_bytes().to_vec())
        .finish().await
    {
        Ok(block) => {
            let result = node.post_block(&block).await;
            match result {
                Ok(_) => {}
                Err(err) => {
                    let local_node = node.clone();
                    let local_data = data.to_vec().clone();
                    let block_id = block.id().clone();
                    thread::spawn(move ||async move{
                        println!("{}",String::from_utf8(local_data).unwrap_or_default());
                        println!("{:?}",err);
                        let _ = local_node.retry_until_included(&block_id, None, None).await;
                    });
                }
            }
        },
        Err(_) => {
            send_data_in_block_group(&node, &data,tag).await;
        }
    }
}

pub async fn send_data_in_block_group(node : &Client, data : &Vec<u8>,tag : String) -> Option<Block>{
    let chunks = data.chunks(Block::LENGTH_MAX);
    let chunks_len = chunks.len();

    let mut block_vector: Vec<BlockId> = Vec::new();

    for chunk in chunks{
        let block_result = node.block()
            .with_tag(tag.as_bytes().to_vec())
            .with_data(chunk.to_vec())
            .finish().await;
        match block_result {
            Ok(block) => {block_vector.push(block.id());}
            Err(err) => {println!("Error send_data_in_block_group for loop: {:?}", err);}
        }
    }

    // 8 Is the maximum number of parents
    if block_vector.len() > 8 {
        println!("Too many parents: {}", block_vector.len());
        println!("Byte size: {}", data.len() );
        return None
    }

    //Creating block with the data chunks included and some additional information
    let mut ids = String::new();
    ids.push_str("{\"messages\":[");
    for block_id in &block_vector {
        ids.push_str("\"");
        ids.push_str(block_id.to_string().as_str());
        ids.push_str("\",")
    }
    let _ = ids.pop();
    ids.push_str("],\"totalLength\":");
    ids.push_str(String::from(data.len().to_string().as_str()).as_str());
    ids.push_str(",");
    ids.push_str("\"size\":");
    ids.push_str(chunks_len.to_string().as_str());
    ids.push_str("}");

    let parents_result = node.block()
        .with_tag("DataChunkPack".as_bytes().to_vec())
        .with_data(ids.into_bytes())
        .with_parents(block_vector);

    return match parents_result {
        Ok(parents) => {
            let block_result = parents.finish().await;

            return match block_result {
                Ok(block) => {Some(block)}
                Err(_) => {None}
            }
        }
        Err(_) => {None}
    }

}


// https://docs.opensea.io/docs/metadata-standards
struct IRC27 {
    standard: String,
    version: String,
    description: String,
    media_type: String,
    uri: String,
    name: String,
    collection_name: String,
    attributes: Vec<Attribute>,
}

struct Attribute {
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
    for entry in json.entries() {
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