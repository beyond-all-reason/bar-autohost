mod autohost;
mod config;
mod environment;
mod spring;

use autohost::{Autohost, AutohostError};
use config::AutohostConfig;
use environment::AutohostEnvironment;
use spring::SpringHeadless;

fn main() -> Result<(), AutohostError> {
    let environment = AutohostEnvironment::new();
    let config = AutohostConfig::build()?;
    let spring = SpringHeadless::new();

    let autohost = Autohost::new(&config, &spring, &environment);

    autohost.start_game()?;

    Ok(())
}
