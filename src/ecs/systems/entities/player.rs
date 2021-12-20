use crate::ecs::{
    entities::player::{Player, Username},
    systems::Systems,
};

pub fn init_systems(s: &mut Systems) {
/*     s.add_system("check_disconnected", |game| {
        let mut to_despawn = Vec::new();
        let mut to_check = Vec::new();
        for (p, _) in game.ecs.query::<&Player>().iter() {
            to_check.push(p);
        }
        for e in to_check {
            if game
                .get_client(e)?
                .borrow()
                .recv_packets_recv
                .is_disconnected()
            {
                to_despawn.push(e);
            }
        }
        for e in to_despawn {
            log::info!(
                "Disconnecting plr {}",
                game.ecs.entity(e)?.get::<Username>()?.0
            );
            game.despawn(e);
        }
        Ok(())
    }); */
}
