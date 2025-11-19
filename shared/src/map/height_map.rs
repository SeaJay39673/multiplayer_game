use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use anyhow::{Result, anyhow};
use noise::{NoiseFn, Perlin};

static PERLIN: LazyLock<RwLock<Perlin>> = LazyLock::new(|| RwLock::new(Perlin::new(0)));

pub fn generate_heightmap(postion: &[i64; 2], size: u8) -> Result<HashMap<(i64, i64), i64>> {
    let scale = 0.025;

    let mut height_map: HashMap<(i64, i64), i64> = HashMap::new();
    for y in -(size as i64)..=size as i64 {
        for x in -(size as i64)..=size as i64 {
            let pos = [(x + postion[0]), (y + postion[1])];
            let noise = PERLIN
                .read()
                .map_err(|e| anyhow!("Could not get PERLIN to read: {e}"))?
                .get([pos[0] as f64 * scale, pos[1] as f64 * scale]);
            height_map.insert((pos[0], pos[1]), (((noise + 1.0) * 0.5) * 5.0) as i64);
        }
    }

    Ok(height_map)
}
