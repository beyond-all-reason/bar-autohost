use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Session start error")]
    SessionStart(String),
    #[error("Session end error")]
    SessionEnd(String),
}
