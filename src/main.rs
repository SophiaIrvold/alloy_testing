use std::error::Error;

use alloy::{primitives::b256, providers::Provider, rpc::types::Filter, sol_types::*};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let provider = "wss://eth-mainnet.ws.alchemyapi.io/ws/demo";
    let provider = alloy::providers::WsConnect::new(provider);
    let provider = alloy::providers::ProviderBuilder::new()
        .on_ws(provider)
        .await?;

    let latest_block = provider.get_block_number().await?;

    alloy::sol! {
        #[derive(Debug)]
        event Transfer(address indexed from, address indexed to, uint256 amount);
    };

    let filter = Filter::new()
        .from_block(latest_block)
        .event_signature(Transfer::SIGNATURE_HASH)
        .topic1(b256!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));

    let mut log_stream = provider
        .subscribe_logs(&filter)
        .await?
        .into_stream()
        .filter_map(|l: alloy::rpc::types::Log| l.log_decode::<Transfer>().ok());

    while let Some(log) = log_stream.next().await {
        println!(
            "just printed some money on token {} going to {}, amount {}",
            log.inner.address, log.inner.to, log.inner.amount
        );
        dbg!(log);
    }
    Ok(())
}
