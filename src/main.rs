use iota_wallet::Error;

mod hornet_adapter;
mod eddn_adapter;

#[tokio::main]
async fn main() -> Result<(),Error>{
    eddn::subscribe_to_eddn().await;
    Ok(())
}