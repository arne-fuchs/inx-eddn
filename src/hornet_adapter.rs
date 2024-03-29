use std::sync::mpsc::Receiver;
use base64::Engine;
use base64::engine::general_purpose;

use iota_sdk::client::Client;
use iota_sdk::client::constants::IOTA_COIN_TYPE;
use iota_sdk::client::secret::SecretManage;
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::crypto::keys::bip44::Bip44;
use serde_json::json;
use rustc_hex::ToHex;

pub struct Hornet {
    pub node: Client,
    pub stronghold: StrongholdSecretManager,
    pub bus_reader: Receiver<Vec<u8>>,
}

impl Hornet {
    pub fn attach(&mut self, blob: Vec<u8>) {
        let thread_data = blob.clone();
        let thread_node = self.node.clone();
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {

                let bip44_chain = Bip44::new(IOTA_COIN_TYPE)
                    .with_account(0)
                    .with_change(false as _)
                    .with_address_index(0);

                let ed255195_signature = self.stronghold
                    .sign_ed25519(&thread_data, bip44_chain)
                    .await.unwrap();

                let mut signature: String = "0x".to_string();
                let tmp_sig: String = ed255195_signature.signature().to_bytes().to_hex();
                signature.push_str(tmp_sig.as_str());

                let mut public_key: String = "0x".to_string();
                let tmp_pk: String = ed255195_signature.public_key().to_bytes().to_hex();
                public_key.push_str(tmp_pk.as_str());

                let message_data = general_purpose::STANDARD.encode(thread_data);


                let message = json!(
                                    {
                                        "message": message_data,
                                        "signature": signature,
                                        "public_key": public_key
                                    }
                                );

                let result = thread_node.build_block()
                    .with_tag("EDDN".as_bytes().to_vec())
                    .with_data(message.to_string().as_bytes().to_vec())
                    .finish()
                    .await;

                match result {
                    Ok(_block) => {
                        //println!("{}",_block.id());
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
            }
            Err(err) => {
                println!("Rec. error: {}", err);
            }
        }
    }
}
