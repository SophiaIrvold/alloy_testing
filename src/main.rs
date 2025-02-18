use std::error::Error;

use alloy::{primitives::b256, providers::Provider, rpc::types::Filter, sol_types::*};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (quit_tx, mut quit_rx) = tokio::sync::oneshot::channel();
    let mut quit_tx = Some(quit_tx);


    // just dying seems to make the eth node think we still have the log subscription going for a while
    ctrlc::set_handler(move || {
        if let Some(quit_tx) = quit_tx.take() {
            quit_tx.send(()).unwrap();
        }
        println!("exiting gracefully");
    }).unwrap();


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

    loop {
        tokio::select! {
            log = log_stream.next() => {
                if log.is_none() {
                    break;
                }
                let log = log.unwrap();
                println!(
                    "just printed some money on token {} going to {}, amount {}",
                    log.inner.address, log.inner.to, log.inner.amount
                );
                dbg!(log);
            }
            _ = &mut quit_rx => {
                break
            }
        }
    }

    Ok(())
}
