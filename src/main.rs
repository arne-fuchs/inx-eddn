use std::collections::VecDeque;
use std::fs::File;
use std::path::PathBuf;
use std::thread;

use bus::Bus;
use dotenv::dotenv;
use iota_wallet::ClientOptions ;
use iota_wallet::account_manager::AccountManager;
use iota_wallet::iota_client::constants::SHIMMER_COIN_TYPE;
use iota_wallet::iota_client::generate_mnemonic;
use iota_wallet::secret::stronghold::StrongholdSecretManager;
use tokio::time::Instant;

use crate::eddn_adapter::EddnAdapter;
use crate::hornet_adapter::Hornet;

mod hornet_adapter;
mod eddn_adapter;

fn main(){
    dotenv().expect(".env file not found");

    let mut hornet_bus: Bus<Vec<u8>> = Bus::new(1000);
    let bus_reader = hornet_bus.add_rx();

    println!("Loading wallet");

    let client_options = ClientOptions::new()
        .with_local_pow(true)
        .with_pow_worker_count(std::env::var("NUM_OF_WORKERS").unwrap().parse().unwrap())
        .with_node(std::env::var("NODE_URL").unwrap().as_str()).unwrap();

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
                    // Create the account managero
                    let manager = AccountManager::builder()
                        .with_client_options(client_options)
                        .with_coin_type(SHIMMER_COIN_TYPE)
                        .finish().await.unwrap();

                    // Set the stronghold password
                    manager
                        .set_stronghold_password(std::env::var("WALLET_PASSWORD").unwrap().as_str())
                        .await.unwrap();

                    // Get the account we generated with `01_create_wallet`
                    let account = manager.get_account("User").await.unwrap();

                    let balance = account.sync(None).await.unwrap();

                    println!("[Total: {} : Available: {}]",balance.base_coin.total,balance.base_coin.available);
                    println!("[NFTS Count: {}]",balance.nfts.len());
                    println!("[Req. storage deposit (basic): {}]",balance.required_storage_deposit.basic());

                    return account;
                })
        }
        Err(err) => {
            println!("{}", &err);
            println!("{}",err);
            println!("Stronghold file not found -> creating");
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    // Setup Stronghold secret_manager
                    let mut secret_manager = StrongholdSecretManager::builder()
                        .password(std::env::var("WALLET_PASSWORD").unwrap().as_str())
                        .build(PathBuf::from("wallet.stronghold")).unwrap();

                    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
                    let mnemonic = generate_mnemonic().unwrap();

                    // The mnemonic only needs to be stored the first time
                    secret_manager.store_mnemonic(mnemonic).await.unwrap();

                    let manager = AccountManager::builder()
                        .with_secret_manager(iota_wallet::secret::SecretManager::Stronghold(secret_manager))
                        .with_client_options(client_options)
                        .with_coin_type(SHIMMER_COIN_TYPE)
                        .finish()
                        .await.unwrap();

                    // Create a new account
                    manager
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
            let address = account.addresses().await.unwrap()[0].address().to_bech32();
            println!("Address: {}",&address);
            return address;
        });

    let bech32_hrp = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed creating addresses")
        .block_on(async {
            return account.client().get_bech32_hrp().await.unwrap();
        });

    println!("Bech32: {}",&bech32_hrp);
    println!("Done loading wallet!");
    let mut hornet = Hornet {
        node: account.client().clone(),
        account,
        bus_reader,
    };
    let eddn = EddnAdapter{
        hornet_bus,
        queue: VecDeque::new(),
        timestamp: Instant::now(),
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