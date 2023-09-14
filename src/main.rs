use std::collections::VecDeque;
use std::fs::File;
use std::path::PathBuf;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use bus::Bus;
use dotenv::dotenv;
use iota_sdk::client::constants::SHIMMER_COIN_TYPE;
use iota_sdk::client::generate_mnemonic;
use iota_sdk::client::secret::{SecretManage, SecretManager};
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::crypto::keys::bip44::Bip44;
use iota_sdk::types::block::address::ToBech32Ext;
use iota_sdk::Wallet;
use iota_sdk::wallet::ClientOptions;
use rustc_hex::ToHex;
use tokio::time::Instant;

use crate::eddn_adapter::EddnAdapter;
use crate::hornet_adapter::Hornet;

mod hornet_adapter;
mod eddn_adapter;

fn main() {
    dotenv().expect(".env file not found");

    let mut hornet_bus: Bus<Vec<u8>> = Bus::new(1000);
    let bus_reader = hornet_bus.add_rx();

    println!("Loading wallet");

    let workers: usize = std::env::var("NUM_OF_WORKERS").unwrap().parse().unwrap();

    let client_options = ClientOptions::new()
        .with_local_pow(true)
        .with_pow_worker_count(workers)
        .with_node(std::env::var("NODE_URL").unwrap().as_str()).unwrap();

    let secret_manager = StrongholdSecretManager::builder()
        .password(std::env::var("WALLET_PASSWORD").unwrap().to_string())
        .build("wallet.stronghold").unwrap();

    //create stronghold account
    let wallet_file_result = File::open("wallet.stronghold");


    let account = match wallet_file_result {
        Ok(file) => {
            println!("{:?}", file);
            println!("Stronghold file found");
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let stronghold = SecretManager::Stronghold(secret_manager);

                    let wallet = Wallet::builder()
                        .with_secret_manager(stronghold)
                        .with_client_options(client_options)
                        .with_coin_type(SHIMMER_COIN_TYPE)
                        .finish().await.unwrap();

                    let account = wallet
                        .get_account("User").await.unwrap();

                    println!("{:?}", account.client());

                    println!("Bech32: {}", account.client().get_bech32_hrp().await.unwrap());

                    let mut balance_result = account.sync(None).await;
                    while balance_result.is_err() {
                        println!("{}", balance_result.err().unwrap());
                        sleep(Duration::from_secs(2));
                        balance_result = account.sync(None).await;
                    }

                    let balance = balance_result.unwrap();

                    println!("[Total: {} : Available: {}]", balance.base_coin().total(), balance.base_coin().available());
                    println!("[NFTS Count: {}]", balance.nfts().len());
                    println!("[Req. storage deposit (basic): {}]", balance.required_storage_deposit().basic());

                    return account;
                })
        }
        Err(err) => {
            println!("{}", &err);
            println!("{}", err);
            println!("Stronghold file not found -> creating");
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
                    let wallet = Wallet::builder()
                        .with_secret_manager(SecretManager::Stronghold(secret_manager))
                        .with_client_options(client_options)
                        .with_coin_type(SHIMMER_COIN_TYPE)
                        .finish().await.unwrap();

                    // The mnemonic only needs to be stored the first time
                    let mnemonic = wallet.generate_mnemonic().unwrap();
                    wallet.store_mnemonic(mnemonic).await.unwrap();

                    // Create a new account
                    wallet
                        .create_account()
                        .with_alias("User".to_string())
                        .finish()
                        .await.unwrap()
                })
        }
    };

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            return account.sync(None).await.unwrap();
        });

    //get address one time so it doesn't have to be created each time
    let _ = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating addresses")
        .block_on(async {
            let address = account.addresses().await.unwrap()[0].address().to_string();
            println!("Address: {}", &address);
            return address;
        });

    let bech32_hrp = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating addresses")
        .block_on(async {
            return account.client().get_bech32_hrp().await.unwrap();
        });

    assert_eq!(&bech32_hrp, "edcas");

    let mut sig = "0x".to_string();
    sig.push_str(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating thread")
        .block_on(async {
            let secret_manager = StrongholdSecretManager::builder()
                .password(std::env::var("WALLET_PASSWORD").unwrap().to_string())
                .build("wallet.stronghold").unwrap();
            let sig: String =
                secret_manager.sign_ed25519(&[0, 1],
                                            Bip44::new(SHIMMER_COIN_TYPE)
                                                .with_account(0)
                                                .with_change(false as _)
                                                .with_address_index(0),
                ).await.unwrap().public_key().to_bytes().to_hex();
            return sig;
        }).as_str());

    println!("Public key: {}", sig);
    println!("Bech32: {}", &bech32_hrp);
    println!("Done loading wallet!");
    let mut hornet = Hornet {
        node: account.client().clone(),
        account,
        stronghold: StrongholdSecretManager::builder()
            .password(std::env::var("WALLET_PASSWORD").unwrap().to_string())
            .build("wallet.stronghold").unwrap(),
        bus_reader,
    };
    let eddn = EddnAdapter {
        hornet_bus,
        queue: VecDeque::new(),
        timestamp: Instant::now(),
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