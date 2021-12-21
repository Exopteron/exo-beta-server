use once_cell::sync::Lazy;
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
    pub chunk_generator: String,
    pub translation_file: String,
    pub tps: i32,
    pub world_seed: Option<u64>,
    pub autosave_interval: i64,
    pub logging: LoggingConfig,
    // generic configuration, max players etc
}
#[derive(serde_derive::Deserialize, Debug)]
pub struct LoggingConfig {
    pub chunk_load: bool,
    pub chunk_unload: bool,
    pub chunk_gen: bool,
    pub slow_ticks: bool,
    pub profiler: bool,
}
const DEFAULT_CONFIG: &str = r#"# Default config

# Listen address
listen_address = "127.0.0.1"

# Listen port
listen_port = 25565

# Server MOTD 
server_motd = "there!"

# Max players.
max_players = 32

# World file location. Minecraft region file format.
level_name = "world"

# Player chunk load distance
chunk_distance = 8

# Chunk generator, can be: (flat, noise, mountain). Noise is quite slow (working on it). CURRENTLY HAS NO EFFECT.
chunk_generator = "noise"

# Translation file path.
translation_file = "translation.json"

# Server TPS (Ticks Per Second), probably shouldn't change it. But who's stopping you?
tps = 20

# World seed, optional. If not specified the server will use a random seed. Must be an unsigned 64 bit number. (u64 type)
# world_seed = 420

# Autosave interval (in ticks)
autosave_interval = 1200

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

# Profiler
profiler = false
"#;

pub static CONFIGURATION: Lazy<ServerConfig> = Lazy::new(|| {
    //ServerConfig {listen_address: "127.0.0.1".to_string(), listen_port: 25565, server_name: "Hello".to_string(), server_motd: "there!".to_string()}
    get_options()
});
pub fn get_options() -> ServerConfig {
    let file = if let Ok(f) = std::fs::read_to_string("config.toml") {
        f
    } else {
        log::info!("Generating configuration file.");
        std::fs::write("config.toml", DEFAULT_CONFIG).unwrap();
        DEFAULT_CONFIG.to_string()
    };
    let config: ServerConfig = if let Ok(c) = toml::from_str(&file) {
        c
    } else {
        log::error!("Invalid configuration file!");
        std::process::exit(1);
    };
    config
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
