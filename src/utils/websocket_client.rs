use thiserror::Error;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};

use std::net::TcpStream;

#[derive(Error, Debug)]
pub enum WebsocketError {
    #[error("Connection error")]
    Connection(String),
}

pub trait WebsocketClient {
    fn connect(&mut self, server_url: &str) -> Result<(), WebsocketError>;
}

#[derive(Default)]
pub struct TachyonClient {
    websocket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl TachyonClient {
    pub fn new() -> TachyonClient {
        TachyonClient { websocket: None }
    }
}

impl WebsocketClient for TachyonClient {
    fn connect(&mut self, server_url: &str) -> Result<(), WebsocketError> {
        let (socket, _response) = connect(server_url)
            .map_err(|e| WebsocketError::Connection(format!("Connection error: {:?}", e)))?;

        self.websocket = Some(socket);

        Ok(())
    }
}
