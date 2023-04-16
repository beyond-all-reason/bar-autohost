mod config;
mod environment;
mod http_client;
mod lobby;
mod server;
mod spring;

use config::AutohostConfig;
use environment::AutohostEnvironment;
use http_client::TeiHttpClient;
use lobby::{Lobby, LobbyError};
use server::{Server, TeiServer};
use spring::SpringHeadless;

#[tokio::main]
async fn main() -> Result<(), LobbyError> {
    let environment = AutohostEnvironment::new();
    let config = AutohostConfig::build()?;
    let spring = SpringHeadless::new();
    let http_client = TeiHttpClient::new();

    let lobby = Lobby::new(&config, &spring, &environment);

    let mut server = TeiServer::new(&config, &http_client);
    server.start_session().await?;

    lobby.start_game()?;

    server.end_session().await?;

    Ok(())
}
