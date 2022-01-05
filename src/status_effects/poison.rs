use crate::{game::Position, network::ids::NetworkID, protocol::packets::EntityEffectKind};

use super::StatusEffect;

pub struct PoisonEffect {
    duration: i16,
    amplifier: i8,
}
impl StatusEffect for PoisonEffect {
    fn on_apply(
        &mut self,
        game: &mut crate::game::Game,
        server: &mut crate::server::Server,
        entity: hecs::Entity,
    ) -> crate::ecs::systems::SysResult {
        let pos = *game.ecs.get::<Position>(entity)?;
        let id = *game.ecs.get::<NetworkID>(entity)?;
        server.broadcast_nearby_with(pos, |cl| {
            cl.send_entity_effect(id, EntityEffectKind::Poison, self.amplifier, self.duration);
        });
        Ok(())
    }

    fn on_remove(
        &mut self,
        game: &mut crate::game::Game,
        server: &mut crate::server::Server,
        entity: hecs::Entity,
    ) -> crate::ecs::systems::SysResult {
        Ok(())
    }

    fn tick(
        &mut self,
        game: &mut crate::game::Game,
        server: &mut crate::server::Server,
        entity: hecs::Entity,
    ) -> crate::ecs::systems::SysResult {
        if self.duration > 0 {
            self.duration -= 1;
        }
        panic!("AHhj!");
        Ok(())
    }

    fn show_client(
        &self,
        us: &crate::ecs::EntityRef,
        client: &crate::server::Client,
    ) -> crate::ecs::systems::SysResult {
        let id = us.get::<NetworkID>()?;
        client.send_entity_effect(*id, EntityEffectKind::Poison, self.amplifier, self.duration);
        Ok(())
    }

    fn should_remove(&self) -> bool {
        self.duration == 0
    }
}

impl PoisonEffect {
    pub fn new(amplifier: i8, duration: i16) -> Self {
        Self {
            amplifier,
            duration
        }
    }
}