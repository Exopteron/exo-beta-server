pub mod aabb;
pub mod api;
pub mod block_entity;
pub mod configuration;
pub mod ecs;
pub mod entities;
pub mod events;
pub mod feather_tick_loop;
pub mod game;
pub mod item;
pub mod logging;
pub mod network;
pub mod objects;
pub mod player_count;
pub mod plugins;
pub mod protocol;
pub mod server;
pub mod translation;
pub mod world;
pub mod status_effects;
pub mod physics;
mod player_dat;
use configuration::CONFIGURATION;
use feather_tick_loop::TickLoop;
pub mod commands;
use anyhow::anyhow;
use logging::file::LogManager;
use std::cell::RefCell;
use std::io::Read;
pub mod jvm;
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn main() -> anyhow::Result<()> {
    std::fs::create_dir_all("local/")?;
    let appender = logging::setup_logging();
    let start = Instant::now();
    let mut manager = PluginManager::new();
    let mut loaders = BlockEntityNBTLoaders::default();
    let mut recipes = Solver::new();
    let mut item_registry = ItemRegistry::new(recipes);
    manager.register_items(&mut item_registry);
    default::register_items(&mut item_registry);
    item_registry.apply_loaders(&mut loaders);
    item_registry.set();
    let mut recipes = ItemRegistry::global().deref().clone();
    item::recipes::register(&mut recipes.solver);
    item::recipes::register_furnace(&mut recipes.furnace);
    recipes.set();
    if let Some(str) = &CONFIGURATION.vanilla_jar {
        JVMSetup::setup(str).unwrap();
    }
    let mut generators = WorldgenRegistry::default();
    crate::world::generation::default_world_generators(&mut generators);
    generators.set();
    log::info!("Starting server version {} for Minecraft b1.8.1", VERSION);
    let _ = &configuration::CONFIGURATION.max_players;
    let translation = TranslationManager::initialize()?;
    let mut systems = SystemExecutor::<Game>::new();
    systems.add_system(|game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>().unwrap();
        game.accept_packets(&mut server)?;
        Ok(())
    });
    systems.add_system(|game| {
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        game.poll_new_players(&mut server)?;
        Ok(())
    });
    /*     systems.add_system("chunk_loading", |game| {
        for (_, world) in game.worlds.iter_mut() {
            world.process_chunk_loads(&mut game);
        }
        Ok(())
    }); */
    let mut game = game::Game::new(manager);
    game.insert_object(Scheduler::new());
    game.insert_object(OpManager::new());
    ecs::systems::default_systems(&mut game, &mut systems);
    let plugins = game.plugins.clone();
    let mut plugins = plugins.borrow_mut();
    plugins.load_all(&mut game, &mut systems);
    let server = server::Server::bind().await?;
    server.register(&mut game);
    game.insert_object(translation);
    let systems_list: Vec<&str> = systems.system_names().collect();
    log::info!("---SYSTEMS---\n{:#?}\n", systems_list);
    game.systems = Arc::new(RefCell::new(systems));
    game.insert_object(BlockUpdateManager::new());
    game.insert_object(loaders);
    log::info!(
        "Done! ({}ms) For command help, run \"help\".",
        start.elapsed().as_millis()
    );
    run(game, appender);
    loop {}
    Ok(())
}
use std::ops::Deref;
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
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::ProcessorExt;

use crate::block_entity::BlockEntityNBTLoaders;
use crate::configuration::OpManager;
use crate::ecs::entities::player::{Username, Player};
use crate::ecs::systems::world::block::update::BlockUpdateManager;
use crate::ecs::systems::SystemExecutor;
use crate::game::{Game, Scheduler};
use crate::item::crafting::{Solver, ShapelessRecipe};
use crate::item::default;
use crate::item::item::ItemRegistry;
use crate::item::stack::ItemStack;
use crate::jvm::JVMSetup;
use crate::player_dat::PlayerDat;
use crate::plugins::PluginManager;
use crate::server::Server;
use crate::translation::TranslationManager;
use crate::world::generation::WorldgenRegistry;
use crate::world::generation::mcvanillagenerator::VanillaWorldGenerator;
pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

//use plugins::PluginManager;
fn setup_tick_loop(mut game: game::Game, appender: LogManager) -> TickLoop {
    std::env::set_var("RUST_BACKTRACE", "1");
    ctrlc::set_handler(move || {
        log::info!("Got ctrlc");
        SHUTDOWN.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");
    use std::time::{Duration, Instant};
    let mut tick_counter = 0;
    let mut last_tps_check = Instant::now();
    TickLoop::new(move || {
        game.ticks += 1;
        if SHUTDOWN.load(Ordering::Relaxed) {
            log::info!("Shutting down.");
            let mut entities = Vec::new();
            for (entity, (_, _)) in game.ecs.query::<(&Player, &Username)>().iter() {
                entities.push(entity);
            }
            let closure: Box<dyn FnOnce() -> anyhow::Result<()>> = Box::new(|| {
                for entity in entities {
                    let entity_ref = game.ecs.entity(entity)?;
                    let world = game.worlds.get(&0).unwrap();
                    let mut pd_dir = world.world_dir.clone();
                    let mut username = entity_ref.get::<Username>()?.0.clone();
                    username = username.replace("\\", "");
                    username = username.replace("/", "");
                    username = username.replace("..", "");
                    pd_dir.push("players/".to_owned() + &username + ".dat");
                    PlayerDat::from_entity(&entity_ref)?.to_file(pd_dir)?;
                    drop(entity_ref);
                }
                Ok(())
            });
            closure().unwrap();
            let translation = game.objects.get::<TranslationManager>().unwrap();
            for (_, client) in game
                .objects
                .get::<Server>()
                .expect("No server")
                .clients
                .iter()
            {
                client.disconnect(
                    &translation.translate("multiplayer.disconnect.server_shutdown", None),
                );
            }
            let world = game.worlds.get(&0).unwrap();
            let mut path = world.world_dir.clone();
            path.push("level.dat");
            world.level_dat.lock().to_file(path).unwrap();
            for (id, world) in game.worlds.iter_mut() {
                let mut positions = Vec::new();
                for chunk in world.chunk_map.iter_chunks() {
                    let pos = chunk.read().pos.clone();
                    positions.push(pos);
                    //drop(chunk);
                }
                log::info!("Unloading DIM:{} ({} chunks)", id, positions.len());
                for pos in positions {
                    //log::info!("Unloading chunk {} in {}", pos, id);
                    if let Err(e) = world.unload_chunk(&mut game.ecs, &pos) {
                        log::error!("Error saving chunk {}: {:?}", pos, e);
                    }
                }
                world.drop_chunk_sender();
                loop {
                    if world.get_shutdown().load(Ordering::Relaxed) {
                        break;
                    }
                }
            }
            appender.close();
            std::process::exit(0);
        }
        if let Err(_) = panic::catch_unwind(AssertUnwindSafe(|| {
            if last_tps_check + Duration::from_secs(5) < Instant::now() {
                game.tps = tick_counter as f64 / 5.;
                tick_counter = 0;
                last_tps_check = std::time::Instant::now();
            }
            //let mut start = Instant::now();
            let systems = game.systems.clone();
            systems.borrow_mut().run(&mut game);
            //log::info!("Time taken: {}ms", start.elapsed().as_millis());
            tick_counter += 1;
            game.run_scheduler();
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
            //println!("{} loaded chunks", game.loaded_chunks.0.len());
            println!("----- configuration info:\n{:?}", *CONFIGURATION);
            std::process::exit(1);
        };
        false
    })
}
fn run(game: game::Game, appender: LogManager) {
    let tick_loop = setup_tick_loop(game, appender);
    tick_loop.run();
}
