#![allow(missing_docs)]

use eyre::Result;
use iroha::{client, data_model::prelude::*};
use iroha_telemetry::metrics::Status;
use iroha_test_network::*;
use tokio::task::spawn_blocking;

fn status_eq_excluding_uptime_and_queue(lhs: &Status, rhs: &Status) -> bool {
    lhs.peers == rhs.peers
        && lhs.blocks == rhs.blocks
        && lhs.blocks_non_empty == rhs.blocks_non_empty
        && lhs.txs_approved == rhs.txs_approved
        && lhs.txs_rejected == rhs.txs_rejected
        && lhs.view_changes == rhs.view_changes
}

async fn check(client: &client::Client, blocks: u64) -> Result<()> {
    let status_json = reqwest::get(client.torii_url.join("/status").unwrap())
        .await?
        .json()
        .await?;

    let status_scale = {
        let client = client.clone();
        spawn_blocking(move || client.get_status()).await??
    };

    assert!(status_eq_excluding_uptime_and_queue(
        &status_json,
        &status_scale
    ));
    assert_eq!(status_json.blocks_non_empty, blocks);

    Ok(())
}

#[tokio::test]
async fn json_and_scale_statuses_equality() -> Result<()> {
    let network = NetworkBuilder::new().start().await?;
    let client = network.client();

    check(&client, 1).await?;

    {
        let client = client.clone();
        spawn_blocking(move || {
            client.submit_blocking(Register::domain(Domain::new("looking_glass".parse()?)))
        })
    }
    .await??;
    network.ensure_blocks(2).await?;

    check(&client, 2).await?;

    Ok(())
}

#[tokio::test]
async fn get_server_version() -> eyre::Result<()> {
    let network = NetworkBuilder::new().start().await?;
    let client = network.client();
    let response =
        tokio::task::spawn_blocking(move || client.get_server_version().unwrap()).await?;
    assert!(response.version.starts_with("2.0.0"));
    assert!(!response.git_sha.is_empty());
    Ok(())
}
