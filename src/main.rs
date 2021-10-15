#![feature(specialization)]
//pub mod error;
pub mod async_systems;
pub mod chunks;
pub mod configuration;
pub mod feather_tick_loop;
pub mod game;
pub mod logging;
pub mod network;
pub mod objects;
pub mod server;
pub mod systems;
pub mod world;
use configuration::CONFIGURATION;
use feather_tick_loop::TickLoop;
use systems::Systems;
pub mod commands;
use anyhow::anyhow;
use std::io::Read;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::setup_logging();
    let _ = &configuration::CONFIGURATION.server_name;
    let mut systems = Systems::new();
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
    systems.add_system("tick_game_ticks", |game| {
        game.ticks += 1;
        Ok(())
    });
    systems.add_system("sync_positions", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::sync_positions(game, &mut server)?;
        Ok(())
    });
    systems.add_system("tick_entities", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::tick_entities(game, &mut server)?;
        Ok(())
    });
    systems.add_system("tick_players", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::tick_players(game, &mut server)?;
        Ok(())
    });
    systems.add_system("rem_old_clients", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::rem_old_clients(game, &mut server)?;
        Ok(())
    });
    systems.add_system("entity_positions", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::entity_positions(game, &mut server)?;
        Ok(())
    });
    systems.add_system("ping", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::ping(game, &mut server)?;
        Ok(())
    });
    systems.add_system("cull_players", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::cull_players(game, &mut server)?;
        Ok(())
    });
    systems.add_system("time_update", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::time_update(game, &mut server)?;
        Ok(())
    });
    systems.add_system("block_updates", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::block_updates(game, &mut server)?;
        Ok(())
    });
    systems.add_system("check_loaded_chunks", |game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::check_loaded_chunks(game, &mut server)?;
        Ok(())
    });
    systems.add_system("random_ticks", |game| {
        game.random_ticks();
        Ok(())
    });
    systems.add_system("tile_entity_ticks", |game| {
        game.tile_entity_ticks();
        Ok(())
    });
    systems.add_system("world_block_updates", |game| {
        let players = game.players.clone();
        game.world.send_block_updates(players);
        Ok(())
    });
    systems.add_system("handle_events", |game| {
        let obj = game.objects.clone();
        obj.get_mut::<game::events::EventHandler>()?
            .handle_events(game);
        Ok(())
    });
    systems.add_system("handle_async_scheduled_tasks", |game| {
        game.handle_async_commands();
        Ok(())
    });
    let mut manager = PluginManager::new();
    load_plugins(&mut manager);
    let (async_channel_send, async_channel_recv) = flume::unbounded();
    async_systems::setup_async_systems(async_channel_send.clone()).await;
    let (async_chat_send, async_chat_recv) = flume::unbounded();
    if CONFIGURATION.experimental.async_chat {
        let chat_manager =
            async_systems::chat::AsyncChatManager::new(async_channel_send.clone(), async_chat_recv);
        chat_manager.run().await;
    }
    let mut game = game::Game::new(
        systems,
        manager,
        async_channel_recv,
        async_chat_send.clone(),
    );
    let server = server::Server::bind(async_chat_send).await?;
    server.register(&mut game);
    run(game);
    loop {}
    println!("Hello, world!");
    Ok(())
}
fn load_plugins(manager: &mut PluginManager) {
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
}
use std::panic::{self, AssertUnwindSafe};
use sysinfo::ProcessorExt;

use plugins::PluginManager;
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
            game.save_playerdata().unwrap();
            let plrs = game.players.0.borrow().clone();
            for player in plrs.iter() {
                player.1.disconnect("Server closed".to_string());
            }
            game.world.to_file(&CONFIGURATION.level_name);
            std::process::exit(0);
        }
        if let Err(_) = panic::catch_unwind(AssertUnwindSafe(|| {
            if last_tps_check + Duration::from_secs(5) < Instant::now() {
                game.tps = tick_counter as f64 / 5.;
                tick_counter = 0;
                last_tps_check = std::time::Instant::now();
            }
            let systems = game.systems.clone();
            systems.borrow_mut().run(&mut game);
            tick_counter += 1;
        })) {
            game.save_playerdata().unwrap();
            game.world.to_file(&CONFIGURATION.level_name);
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
            println!("online players: {}", game.players.0.borrow().len());
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
