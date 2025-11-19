use std::{collections::HashMap, sync::LazyLock};

use shared::{Player, TileManager};
use tokio::sync::RwLock;

pub static PLAYERS: LazyLock<RwLock<HashMap<String, Player>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub static TILE_MANAGER: LazyLock<RwLock<TileManager>> = LazyLock::new(|| RwLock::new(TileManager::new([0,0])));