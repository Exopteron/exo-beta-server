#![feature(specialization)]
//pub mod error;
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
    systems.add_system(|game| {
        game.ticks += 1;
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::sync_positions(game, &mut server)?;
        //systems::update_local_health(game, &mut server)?;
        systems::tick_entities(game, &mut server)?;
        systems::tick_players(game, &mut server)?;
        //systems::check_dead(game, &mut server)?;
        systems::rem_old_clients(game, &mut server)?;
        //systems::spawn_players(game, &mut server)?;
        systems::entity_positions(game, &mut server)?;
        systems::update_positions(game, &mut server)?;
        /*         systems::chat_msgs(game, &mut server)?; */
        systems::ping(game, &mut server)?;
        systems::cull_players(game, &mut server)?;
        systems::time_update(game, &mut server)?;
        systems::block_updates(game, &mut server)?;
        systems::check_loaded_chunks(game, &mut server)?;
        let players = game.players.0.borrow().clone();
        for player in players.iter() {
            /*             systems::check_inv(game, &mut server, player.1)?;
            systems::sync_inv(game, &mut server, player.1)?; */
        }
        obj.get_mut::<game::events::EventHandler>()?
            .handle_events(game);
        Ok(())
    });
    let mut game = game::Game::new(systems);
    let server = server::Server::bind().await?;
    server.register(&mut game);
    run(game);
    loop {}
    println!("Hello, world!");
    Ok(())
} // this is my error handling which is all i have so far because i'm scared to write any more.
use std::panic::{self, AssertUnwindSafe};
use sysinfo::ProcessorExt;
fn setup_tick_loop(mut game: game::Game) -> TickLoop {
    std::env::set_var("RUST_BACKTRACE", "1");
    TickLoop::new(move || {
        if let Err(_) = panic::catch_unwind(AssertUnwindSafe(|| {
            let systems = game.systems.clone();
            systems.borrow_mut().run(&mut game);
        })) {
            println!("Please report this!");
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
