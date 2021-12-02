pub mod configuration;
pub mod feather_tick_loop;
pub mod game;
pub mod logging;
pub mod network;
pub mod objects;
pub mod server;
pub mod world;
pub mod api;
pub mod ecs;
use configuration::CONFIGURATION;
use feather_tick_loop::TickLoop;
pub mod commands;
use anyhow::anyhow;
use std::io::Read;
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::setup_logging();
    let start = Instant::now();
    log::info!("Starting server version {} for Minecraft 1.2.5", VERSION);
    let _ = &configuration::CONFIGURATION.server_name;
    let mut systems = Systems::new();
    ecs::systems::default_systems(&mut systems);
    systems.add_system("packet_accept", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>().unwrap();
        game.accept_packets(&mut server)?;
        Ok(())
    });
    systems.add_system("poll_new_players", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        game.poll_new_players(&mut server)?;
        Ok(())
    });
    systems.add_system("chunk_loading", |game| {
        for (_, world) in game.worlds.iter_mut() {
            world.process_chunk_loads(&mut game);
        }
        Ok(())
    });
    let mut game = game::Game::new(
        systems,
    );
    let server = server::Server::bind().await?;
    server.register(&mut game);
    let obj = game.objects.clone();
    log::info!("Done! ({}ms) For command help, run \"help\".", start.elapsed().as_millis());
    run(game);
    loop {}
    println!("Hello, world!");
    Ok(())
}
/* fn load_plugins(manager: &mut PluginManager) {
    let mut faxvec: Vec<std::path::PathBuf> = Vec::new();
    std::fs::create_dir_all("plugins/").expect("Could not create plugins folder!");
    for element in std::path::Path::new(r"plugins/").read_dir().unwrap() {
        let path = element.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "so" {
                faxvec.push(path);
            }
        }
    }
    for plugin in faxvec {
        unsafe {
            if let Err(e) = manager.load_plugin(plugin) {
                log::error!("Error loading plugin: {:?}", e);
            }
        }
    }
} */
use std::panic::{self, AssertUnwindSafe};
use std::time::Instant;
use sysinfo::ProcessorExt;

use crate::ecs::systems::Systems;

//use plugins::PluginManager;
fn setup_tick_loop(mut game: game::Game) -> TickLoop {
    std::env::set_var("RUST_BACKTRACE", "1");
    use std::sync::mpsc::channel;
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");
    use std::time::{Duration, Instant};
    let mut tick_counter = 0;
    let mut last_tps_check = Instant::now();
    TickLoop::new(move || {
        if rx.try_recv().is_ok() {
            log::info!("Shutting down.");
            std::process::exit(0);
        }
        if let Err(_) = panic::catch_unwind(AssertUnwindSafe(|| {
            if last_tps_check + Duration::from_secs(5) < Instant::now() {
                //game.tps = tick_counter as f64 / 5.;
                tick_counter = 0;
                last_tps_check = std::time::Instant::now();
            }
            let systems = game.systems.clone();
            systems.borrow_mut().run(&mut game);
            tick_counter += 1;
        })) {
            //game.save_playerdata().unwrap();
            //game.world.get_world().to_file(&CONFIGURATION.level_name);
            println!("========================================");
            println!("\nPlease report this!\n");
            println!("========================================");
            println!("----- Hardware information:");
            use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt};
            let mut sys = System::new_all();
            sys.refresh_all();
            println!("total memory: {} KB", sys.total_memory());
            println!("used memory : {} KB", sys.used_memory());
            println!("System name:             {:?}", sys.name());
            println!("System kernel version:   {:?}", sys.kernel_version());
            println!("System OS version:       {:?}", sys.os_version());
            println!("CPU: {}", sys.global_processor_info().brand());
            println!("----- Game information:");
            //println!("online players: {}", game.players.0.lock().unwrap().len());
            println!("{} loaded chunks", game.loaded_chunks.0.len());
            println!("----- configuration info:\n{:?}", *CONFIGURATION);
            std::process::exit(1);
        };
        false
    })
}
fn run(game: game::Game) {
    let tick_loop = setup_tick_loop(game);
    tick_loop.run();
}
