use once_cell::sync::Lazy;
#[derive(serde_derive::Deserialize, Debug)]
pub struct ServerConfig {
    pub listen_address: String,
    pub listen_port: u16,
    pub server_name: String,
    pub server_motd: String,
    pub max_players: u32,
    pub level_name: String,
    pub chunk_distance: i32,
    pub chunk_generator: String,
    pub tps: i32,
    pub world_seed: Option<u64>,
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

# Server name (unused)
server_name = "Hello!"

# Server MOTD (unused)
server_motd = "there!"

# Max players.
max_players = 32

# World file location. Currently a strange NBT format I cooked up at 2am. dw about it
level_name = "world"

# Player chunk load distance
chunk_distance = 8

# Chunk generator, can be: (flat, noise). Noise is quite slow (working on it).
chunk_generator = "noise"

# Server TPS (Ticks Per Second), probably shouldn't change it. But who's stopping you?
tps = 20

# World seed, optional. If not specified the server will use a random seed. Must be an unsigned 64 bit number. (u64 type)
# world_seed = 420

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
// this will be in no way an implementation of ECS. (bcz idk how)
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