//pub mod error;
pub mod configuration;
pub mod network;
pub mod server;
pub mod game;
pub mod logging;
pub mod objects;
pub mod feather_tick_loop;
pub mod systems;
pub mod world;
pub mod entities;
use systems::Systems;
use feather_tick_loop::TickLoop;
pub mod commands;
use anyhow::anyhow;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::setup_logging();
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
        let obj = game.objects.clone();
        let mut server = obj.get_mut::<server::Server>()?;
        systems::sync_positions(game, &mut server)?;
        //systems::update_local_health(game, &mut server)?;
        systems::tick_players(game, &mut server)?;
        //systems::check_dead(game, &mut server)?;
        systems::rem_old_clients(game, &mut server)?;
        systems::spawn_players(game, &mut server)?;
        systems::update_positions(game, &mut server)?;
/*         systems::chat_msgs(game, &mut server)?; */
        systems::ping(game, &mut server)?;
        systems::cull_players(game, &mut server)?;
        systems::time_update(game, &mut server)?;
        systems::block_updates(game, &mut server)?;
        let players = game.players.0.borrow().clone();
        for player in players.iter() {
/*             systems::check_inv(game, &mut server, player.1)?;
            systems::sync_inv(game, &mut server, player.1)?; */
        }
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

fn setup_tick_loop(mut game: game::Game) -> TickLoop {
    TickLoop::new(move || {
        let systems = game.systems.clone();
        systems.borrow_mut().run(&mut game);
        false
    })
}
fn run(game: game::Game) {
    let tick_loop = setup_tick_loop(game);
    tick_loop.run();
}