use std::collections::VecDeque;
use std::thread;

use bus::Bus;
use iota_sdk::client::constants::{IOTA_COIN_TYPE};
use iota_sdk::client::secret::{SecretManage, SecretManager};
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::crypto::keys::bip44::Bip44;
use iota_sdk::Wallet;
use iota_sdk::wallet::ClientOptions;
use rustc_hex::ToHex;
use tokio::time::Instant;

use crate::eddn_adapter::EddnAdapter;
use crate::hornet_adapter::Hornet;

mod hornet_adapter;
mod eddn_adapter;

fn main() {
    let workers: usize = std::env::var("NUM_OF_WORKERS").unwrap().parse().unwrap();
    let node_url = std::env::var("NODE_URL").unwrap();
    let wallet_path = std::env::var("WALLET_PATH").unwrap_or("wallet.stronghold".to_string());
    let wallet_password = std::env::var("WALLET_PASSWORD").unwrap().to_string();

    let mut hornet_bus: Bus<Vec<u8>> = Bus::new(1000);
    let bus_reader = hornet_bus.add_rx();

    println!("Loading wallet");

    let client_options = ClientOptions::new()
        .with_local_pow(true)
        .with_pow_worker_count(workers)
        .with_node(node_url.as_str()).unwrap();

    let secret_manager = StrongholdSecretManager::builder()
        .password(wallet_password.clone())
        .build(wallet_path.clone()).unwrap();

    let stronghold = SecretManager::Stronghold(secret_manager);

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

            println!("Syncing balance");
            //let balance = account.sync(None).await.unwrap();

            //println!("[Total: {} : Available: {}]", balance.base_coin().total(), balance.base_coin().available());
            //println!("[NFTS Count: {}]", balance.nfts().len());
            //println!("[Req. storage deposit (basic): {}]", balance.required_storage_deposit().basic());

            println!("Balance synced");

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

    assert_eq!(&bech32_hrp, "edcas");

    let mut sig = "0x".to_string();
    sig.push_str(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating thread")
        .block_on(async {
            let secret_manager = StrongholdSecretManager::builder()
                .password(wallet_password.clone())
                .build(wallet_path.clone()).unwrap();
            let sig: String =
                secret_manager.sign_ed25519(&[0, 1],
                                            Bip44::new(IOTA_COIN_TYPE)
                                                .with_account(0)
                                                .with_change(false as _)
                                                .with_address_index(0),
                ).await.unwrap().public_key().to_bytes().to_hex();
            return sig;
        }).as_str());

    println!("Public key: {}", sig);
    println!("Bech32: {}", &bech32_hrp);
    println!("Done loading wallet!");
    println!("Starting threads");
    let mut hornet = Hornet {
        node: account.client().clone(),
        account,
        stronghold: StrongholdSecretManager::builder()
            .password(wallet_password.clone())
            .build(wallet_path.clone()).unwrap(),
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