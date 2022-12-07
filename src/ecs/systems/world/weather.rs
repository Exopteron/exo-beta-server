use crate::{ecs::{systems::{SystemExecutor, SysResult}, entities::player::Player}, game::{Game, Position}, server::Server, events::{WeatherChangeEvent}, network::ids::NetworkID};

pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems.group::<Server>().add_system(weather_system);
}

fn weather_system(game: &mut Game, server: &mut Server) -> SysResult {

    let mut new_weather = vec![];
    for (_, event) in game.ecs.query::<&WeatherChangeEvent>().iter() {
        new_weather.push(*event);
    }
    for weather in new_weather {
        for (_, (id, pos, _)) in game.ecs.query::<(&NetworkID, &Position, &Player)>().iter() {
            if pos.world == weather.world {
                if let Some(cl) = server.clients.get(id) {
                    cl.set_is_raining(weather.is_raining);
                }
            }
        }
    }
    Ok(())
}