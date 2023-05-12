use bar_autohost::server_coms::server::{Server, TeiServer};
use bar_autohost::utils::config::AutohostConfig;
use bar_autohost::utils::environment::AutohostEnvironment;
use bar_autohost::utils::http_client::TeiHttpClient;
use bar_autohost::utils::websocket_client::TachyonClient;

use bar_autohost::autohost::lobby::{Lobby, LobbyError};
use bar_autohost::autohost::spring::SpringHeadless;

#[tokio::main]
async fn main() -> Result<(), LobbyError> {
    let environment = AutohostEnvironment::new();
    let config = AutohostConfig::build()?;
    let spring = SpringHeadless::new();
    let http_client = TeiHttpClient::new();
    let mut socket_client = TachyonClient::new();

    let lobby = Lobby::new(&config, &spring, &environment);

    let mut server = TeiServer::new(&config, &http_client, &mut socket_client);
    server.start_session().await?;

    lobby.start_game()?;

    server.end_session().await?;

    Ok(())
}
