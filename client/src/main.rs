use anyhow::Result;

use crate::game::Game;

mod engine;
mod game;
mod vertex;
mod map;
mod client_player;

#[tokio::main]
async fn main() -> Result<()> {
    let game = Game::new()?;
    game.run()?;
    
    Ok(())
}
