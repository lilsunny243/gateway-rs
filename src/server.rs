use crate::{
    api::LocalServer,
    beaconer, gateway, region_watcher, router,
    settings::{self, Settings},
    Result,
};
use slog::{info, Logger};

pub async fn run(shutdown: &triggered::Listener, settings: &Settings, logger: &Logger) -> Result {
    let (gateway_tx, gateway_rx) = gateway::message_channel();
    let (router_tx, router_rx) = router::message_channel();
    let (beacon_tx, beacon_rx) = beaconer::message_channel();

    let mut region_watcher = region_watcher::RegionWatcher::new(settings);
    let region_rx = region_watcher.watcher();

    let mut beaconer =
        beaconer::Beaconer::new(settings, beacon_rx, region_rx.clone(), gateway_tx.clone());
    let mut router =
        router::Router::new(settings, router_rx, region_rx.clone(), gateway_tx.clone());
    let mut gateway = gateway::Gateway::new(
        settings,
        gateway_rx,
        region_rx.clone(),
        router_tx,
        beacon_tx,
    )
    .await?;
    let api = LocalServer::new(region_rx.clone(), settings)?;
    info!(logger,
        "starting server";
        "version" => settings::version().to_string(),
        "key" => settings.keypair.public_key().to_string(),
    );
    tokio::try_join!(
        region_watcher.run(shutdown, logger),
        beaconer.run(shutdown, logger),
        gateway.run(shutdown, logger),
        router.run(shutdown, logger),
        api.run(shutdown, logger),
    )
    .map(|_| ())
}
