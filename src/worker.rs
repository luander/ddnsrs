use crate::{cf, cli::Cli, ip, Message};
use anyhow::Result;
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{error, info};

#[tracing::instrument(skip(config))]
pub async fn run(config: Cli) -> Result<()> {
    // channel to reeive public IP address updates
    let (message_sender, message_receiver) = broadcast::channel::<Message>(10);
    let (notify_shutdown, _) = broadcast::channel::<bool>(1);

    // spawn the task to retrive the public IP address
    let ip_handle = tokio::spawn(ip::run(
        config.clone(),
        message_sender,
        notify_shutdown.subscribe(),
    ));

    // spawn the task to update the Cloudflare DNS record
    let cf_handle = tokio::spawn(cf::run(
        config,
        message_receiver,
        notify_shutdown.subscribe(),
    ));

    // spawn the task to listen for shutdown signal
    tokio::spawn(shutdown_task(notify_shutdown.clone()));

    if let Err(e) = tokio::try_join!(flatten(ip_handle), flatten(cf_handle)) {
        error!("Error: {:?}", e);
        notify_shutdown.send(true).ok();
        return Err(e);
    }

    Ok(())
}

async fn flatten<T>(handle: JoinHandle<Result<T>>) -> Result<T> {
    handle.await?
}

async fn shutdown_task(notify_shutdown: broadcast::Sender<bool>) {
    let _ = tokio::signal::ctrl_c().await;
    match notify_shutdown.send(true) {
        Ok(_) => info!("Shutting down..."),
        Err(e) => error!("Error: {:?}", e),
    }
}
