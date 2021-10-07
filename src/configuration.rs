use once_cell::sync::Lazy;
pub struct ServerConfig {
    pub listen_address: String,
    pub listen_port: u16,
    pub server_name: String,
    pub server_motd: String,
    // generic configuration, max players etc
}
// this will be in no way an implementation of ECS. (bcz idk how)
pub static CONFIGURATION: Lazy<ServerConfig> = Lazy::new(|| {
    ServerConfig {listen_address: "127.0.0.1".to_string(), listen_port: 25565, server_name: "Hello".to_string(), server_motd: "there!".to_string()}
});