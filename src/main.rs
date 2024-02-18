use std::sync::mpsc::channel;
use std::{env, fs, thread};
use std::net::{IpAddr, Ipv4Addr};
use std::process::exit;
use std::str::FromStr;
use iota_sdk::client::Client;

use iota_sdk::client::constants::{IOTA_COIN_TYPE};
use iota_sdk::client::secret::{SecretManage, SecretManager};
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::crypto::keys::bip44::Bip44;
use iota_sdk::Wallet;
use iota_sdk::wallet::ClientOptions;
use rustc_hex::ToHex;

use crate::eddn_adapter::EddnAdapter;
use crate::hornet_adapter::Hornet;

mod hornet_adapter;
mod eddn_adapter;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut workers: usize = env::var("NUM_OF_WORKERS").unwrap().parse().unwrap_or(4);
    let mut node_url = env::var("NODE_URL").unwrap_or("127.0.0.1".to_string());
    let mut wallet_path = env::var("WALLET_PATH").unwrap_or("wallet.stronghold".to_string());
    let mut wallet_password = env::var("WALLET_PASSWORD").unwrap_or("".to_string());
    let mut hrp = env::var("HRP").unwrap_or("edcas".to_string());

    let length = args.len();
    for i in 1..length {
        match args[i].as_str() {
            "-a" => {
                node_url = args[i+1].clone();
            }
            "-w" => {
                workers = usize::from_str(args[i+1].as_str()).unwrap();
            }
            "-p" => {
                wallet_password = args[i+1].clone();
            }
            "-f" => {
                wallet_path = args[i+1].clone();
            }
            "-h" => {
                hrp = args[i+1].clone();
            }
            &_ => {}
        }
    }

    println!("Getting secret manager");

    let mut sig = "0x".to_string();
    let (secret_manager, suffix) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating threads")
        .block_on(async {
            let secret_manager = StrongholdSecretManager::builder()
                .password(wallet_password.clone())
                .build(wallet_path.clone()).unwrap();

            let mnemonic = Client::generate_mnemonic().unwrap();
            match secret_manager.store_mnemonic(mnemonic).await {
                Ok(_) => {}
                Err(err) => {
                    println!("{}",err);
                }
            }

            let sig: String =
                secret_manager.sign_ed25519(&[0, 1],
                                            Bip44::new(IOTA_COIN_TYPE)
                                                .with_account(0)
                                                .with_change(false as _)
                                                .with_address_index(0),
                ).await.unwrap().public_key().to_bytes().to_hex();
            return (secret_manager,sig);
        });
    sig.push_str(suffix.as_str());
    let stronghold = SecretManager::Stronghold(secret_manager);

    println!("Public key: {}", sig);

    if let Some(arg) = args.get(1){
        if arg == "--saveKey" {
            let mut prefix = "EDDN_PUBLIC_KEY=".to_string();
            prefix.push_str(&sig);
            let key_location = env::var("KEY_SAVE_LOCATION").unwrap();
            println!("Save key argument provided -> saving key to location {}", key_location);
            fs::write(key_location,prefix).unwrap();
            exit(0);
        }else {
            println!("Unknown argument: {}", arg);
        }
    }


    let (hornet_bus, bus_reader) = channel::<Vec<u8>>();

    println!("Loading wallet");

    let client_options = ClientOptions::new()
        .with_local_pow(true)
        .with_pow_worker_count(workers)
        .with_node(node_url.as_str()).unwrap();

    let account =  tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let wallet = Wallet::builder()
                .with_secret_manager(stronghold)
                .with_client_options(client_options)
                .with_coin_type(IOTA_COIN_TYPE)
                .finish().await.unwrap();

            let account = match wallet
                .get_account("User").await {
                Ok(account) => {account}
                Err(err) => {
                    println!("{}",err);

                    let mnemonic = wallet.generate_mnemonic().unwrap();
                     match wallet.store_mnemonic(mnemonic).await {
                         Ok(_) => {}
                         Err(err) => {
                             println!("{}",err);
                         }
                     }

                    // Create a new account
                    wallet
                        .create_account()
                        .with_alias("User")
                        .finish().await.unwrap()
                }
            };

            println!("Bech32: {}", account.client().get_bech32_hrp().await.unwrap());

            //FIXME Crashes for no reason
            //println!("Syncing balance");

            //let balance = account.sync(None).await.unwrap();

            //println!("[Total: {} : Available: {}]", balance.base_coin().total(), balance.base_coin().available());
            //println!("[NFTS Count: {}]", balance.nfts().len());
            //println!("[Req. storage deposit (basic): {}]", balance.required_storage_deposit().basic());

            //println!("Balance synced");

            return account;
        });

    println!("Wallet loaded");
    println!("Creating address");

    //get address one time so it doesn't have to be created each time
    let address = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating addresses")
        .block_on(async {
            let address = account.addresses().await.unwrap()[0].address().to_string();
            return address;
        });

    println!("Address created: {}", address);
    println!("Getting Hrp");

    let bech32_hrp = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed getting hrp")
        .block_on(async {
            return account.client().get_bech32_hrp().await.unwrap();
        });

    println!("Hrp: {}", bech32_hrp.to_string());

    assert_eq!(&bech32_hrp.to_string(), hrp.as_str());

    println!("Bech32: {}", &bech32_hrp);
    println!("Done loading wallet!");
    println!("Starting threads");
    let mut hornet = Hornet {
        node: account.client().clone(),
        stronghold: StrongholdSecretManager::builder()
            .password(wallet_password.clone())
            .build(wallet_path.clone()).unwrap(),
        bus_reader,
    };
    let eddn = EddnAdapter {
        hornet_bus
    };
    thread::spawn(move || {
        loop {
            hornet.read();
        }
    });
    println!("Ready!");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            eddn.subscribe_to_eddn().await;
        });
}