use std::{collections::HashSet, fmt::Debug, sync::Arc};

use ahash::AHashSet;
use once_cell::sync::Lazy;
use rand::RngCore;
use serde::{Deserialize, de::{Visitor, self}, Deserializer};

use crate::world::generation::{WorldGenerator, FlatWorldGenerator, TerrainWorldGenerator, MountainWorldGenerator, CustomWorldGenerator, WorldgenRegistry};
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
struct SerializeOPS {
    ops: Vec<String>,
}
#[derive(serde_derive::Deserialize, Debug)]
pub struct ServerConfig {
    pub listen_address: String,
    pub listen_port: u16,
    pub server_motd: String,
    pub max_players: u32,
    pub level_name: String,
    pub chunk_distance: u32,
    pub chunk_generator: WorldgenData,
    pub light_prop_per_tick: usize,
    pub translation_file: String,
    pub default_gamemode: u8,
    pub tps: i32,
    pub world_seed: Option<u64>,
    pub world_border: i32,
    pub custom_generation: GenConfig,
    pub logging: LoggingConfig,
    // generic configuration, max players etc
}


#[derive(serde_derive::Deserialize, Debug)]
pub struct LoggingConfig {
    pub chunk_load: bool,
    pub chunk_unload: bool,
    pub chunk_gen: bool,
    pub slow_ticks: bool,
    pub packet_transfer: bool,
    pub packet_transfer_exclusion: AHashSet<String>,
}
#[derive(serde_derive::Deserialize, Debug)]
pub struct GenConfig {
    pub x_step: f64,
    pub y_step: f64,
    pub multiplication: i64,
}
const DEFAULT_CONFIG: &str = r#"# Default config

# Listen address
listen_address = "127.0.0.1"

# Listen port
listen_port = 25565

# Server MOTD 
server_motd = "A Ton of Rocks server"

# Max players.
max_players = 32

# World file location. Minecraft region file format.
level_name = "world"

# Player chunk load distance
chunk_distance = 8

# Chunk generator, can be: (flat, terrain, mountain, custom).
chunk_generator = "terrain"

# How many requests should the light propagator propagate per tick?
light_prop_per_tick = 50

# Translation file path.
translation_file = "translation.json"

# Default gamemode.
default_gamemode = 0

# Server TPS (Ticks Per Second), probably shouldn't change it. But who's stopping you?
tps = 20

# World seed, optional. If not specified the server will use a random seed. Must be an unsigned 64 bit number. (u64 type)
# world_seed = 420

# World border. How many blocks out can a player go from spawn?
world_border = 16000

# Custom worldgen options. These take effect when chunk_generator is "custom".

[custom_generation]

x_step = -0.01
y_step = 0.01
multiplication = 25

# Logging options

[logging]

# Log chunk loads
chunk_load = false

# Log chunk unloads
chunk_unload = false

# Log chunk generation
chunk_gen = false

# Log ticks that take too long
slow_ticks = true

# Log packet transfer
packet_transfer = false

# Packets excluded from the log
packet_transfer_exclusion = ["PreChunk", "ChunkData", "TimeUpdate", "PlayerMovement", "PlayerLook", "PlayerPosition", "PlayerPositionAndLook"]
"#;

pub static CONFIGURATION: Lazy<ServerConfig> = Lazy::new(|| {
    //ServerConfig {listen_address: "127.0.0.1".to_string(), listen_port: 25565, server_name: "Hello".to_string(), server_motd: "there!".to_string()}
    let mut cfg = get_options();
    if cfg.world_seed == None {
        cfg.world_seed = Some(rand::thread_rng().next_u64());
    }
    cfg
});
pub fn get_options() -> ServerConfig {
    let file = if let Ok(f) = std::fs::read_to_string("config.toml") {
        f
    } else {
        log::info!("Generating configuration file.");
        std::fs::write("config.toml", DEFAULT_CONFIG).unwrap();
        DEFAULT_CONFIG.to_string()
    };
    let c = toml::from_str(&file);
    let config: ServerConfig = if let Ok(c) = c {
        c
    } else if let Err(c) = c {
        log::error!("Invalid configuration file! Details: {}", c);
        std::process::exit(1);
    } else {
        unreachable!();
    };
    config
}
pub struct OpManager {
    ops: AHashSet<String>,
}
impl OpManager {
    pub fn new() -> Self {
        let ops = get_ops();
        let mut set = AHashSet::new();
        for op in ops {
            set.insert(op);
        }
        Self { ops: set }
    }
    pub fn is_op(&self, name: &str) -> bool {
        self.ops.contains(name)
    }
    pub fn add_op(&mut self, name: String) {
        add_op(&name);
        self.ops.insert(name);
    }
    pub fn remove_op(&mut self, name: &str) {
        remove_op(name);
        self.ops.remove(name);
    }
}
pub fn get_ops() -> Vec<String> {
    let file = if let Ok(f) = std::fs::read_to_string("ops.toml") {
        f
    } else {
        log::info!("Generating ops file.");
        std::fs::write("ops.toml", r#"ops = []"#).unwrap();
        r#"ops = []"#.to_string()
    };
    let config: SerializeOPS = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid ops file!");
        std::process::exit(1);
    };
    config.ops
}
pub fn add_op(username: &str) {
    let file = if let Ok(f) = std::fs::read_to_string("ops.toml") {
        f
    } else {
        log::info!("Generating ops file.");
        std::fs::write("ops.toml", r#"ops = []"#).unwrap();
        r#"ops = []"#.to_string()
    };
    let mut config: SerializeOPS = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid ops file!");
        std::process::exit(1);
    };
    let mut doit = true;
    for name in &config.ops {
        if name == username {
            doit = false;
        }
    }
    if doit {
        config.ops.push(username.to_string());
        std::fs::write("ops.toml", toml::to_string(&config).unwrap()).unwrap();
    }
}
pub fn remove_op(username: &str) {
    let file = if let Ok(f) = std::fs::read_to_string("ops.toml") {
        f
    } else {
        log::info!("Generating ops file.");
        std::fs::write("ops.toml", r#"ops = []"#).unwrap();
        r#"ops = []"#.to_string()
    };
    let mut config: SerializeOPS = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid ops file!");
        std::process::exit(1);
    };
    config.ops.retain(|name| name != username);
    std::fs::write("ops.toml", toml::to_string(&config).unwrap()).unwrap();
}


pub struct WorldgenData {
    name: Option<String>,
    generator: Arc<dyn WorldGenerator>,
}
impl Debug for WorldgenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldgenData").field("generator", &self.name).finish()
    }
}
pub struct StringVisitor;

impl<'de> Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a String")
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
            E: serde::de::Error, {
            Ok(v)
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
            E: serde::de::Error, {
        Ok(v.to_string())
    }
}
impl<'de> Deserialize<'de> for WorldgenData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let mut us = WorldgenData { name: None, generator: Arc::new(FlatWorldGenerator {})};
        if let Ok(str) = deserializer.deserialize_string(StringVisitor) {
            if !us.set_self(&str) {
                return Err(de::Error::custom(format!("unknown world generator \"{}\"", str)));
            }
        }
        Ok(us)
    }
}
impl WorldgenData {
    fn set_self(&mut self, name: &str) -> bool {
        let registry = WorldgenRegistry::get();
        let generator = registry.get_generator(name);
        if let Some(generator) = generator {
            self.generator = generator;
            self.name = Some(name.to_string());
            return true;
        }
        false
    }
    pub fn get(&self) -> Arc<dyn WorldGenerator> {
        Arc::clone(&self.generator)
    }
    pub fn name(&self) -> String {
        match &self.name {
            Some(name) => name.clone(),
            None => String::from("null")
        }
    }
}