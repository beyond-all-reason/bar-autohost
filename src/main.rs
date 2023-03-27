mod autohost;
mod config;
mod environment;
mod http_client;
mod server;
mod spring;

use autohost::{Autohost, AutohostError};
use config::AutohostConfig;
use environment::AutohostEnvironment;
use http_client::TeiHttpClient;
use server::{Server, TeiServer};
use spring::SpringHeadless;

#[tokio::main]
async fn main() -> Result<(), AutohostError> {
    let environment = AutohostEnvironment::new();
    let config = AutohostConfig::build()?;
    let spring = SpringHeadless::new();
    let http_client = TeiHttpClient::new();

    let autohost = Autohost::new(&config, &spring, &environment);

    let mut server = TeiServer::new(&config, &http_client);
    server.start_session().await?;

    autohost.start_game()?;

    Ok(())
}
