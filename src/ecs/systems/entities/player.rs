use crate::ecs::{entities::player::{NetworkManager, Player, Username}, systems::Systems};

pub fn init_systems(s: &mut Systems) {
    s.add_system("check_disconnected", |game| {
        let mut to_despawn = Vec::new();
        for (p, (_, net)) in game.ecs.query::<(&Player, &NetworkManager)>().iter() {
            if net.recv_packets_recv.is_disconnected() {
                to_despawn.push(p);
            }
        }
        for e in to_despawn {
            log::info!("Disconnecting plr {}", game.ecs.entity(e)?.get::<Username>()?.0);
            game.ecs.world.despawn(e)?;
        }
        Ok(())
    });
}