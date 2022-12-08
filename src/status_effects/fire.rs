use std::sync::Arc;

use crate::{
    aabb::AABBSize,
    ecs::{
        entities::{living::Health, player::Gamemode, item::ItemEntity},
        systems::SysResult, EntityRef,
    },
    entities::metadata::{EntityBitMask, META_INDEX_ENTITY_BITMASK},
    game::{DamageType, Position},
    item::item::ItemRegistry,
    network::{ids::NetworkID, metadata::Metadata},
    server::Server,
};

use super::StatusEffect;

pub struct FireEffect {
    pub time_ticks: u128,
    timer: i32,
    registry: Arc<ItemRegistry>,
}
impl StatusEffect for FireEffect {
    fn on_apply(
        &mut self,
        game: &mut crate::game::Game,
        server: &mut Server,
        entity: hecs::Entity,
    ) -> SysResult {
        let mut metadata = game.ecs.get_mut::<Metadata>(entity)?;
        metadata.flags.set(EntityBitMask::ON_FIRE, true);
        let entityref = game.ecs.entity(entity)?;
        let network_id = *entityref.get::<NetworkID>()?;
        server.broadcast_nearby_with(*entityref.get::<Position>()?, |client| {
            client.send_entity_metadata(true, network_id, metadata.clone());
        });
        Ok(())
    }

    fn on_remove(
        &mut self,
        game: &mut crate::game::Game,
        server: &mut Server,
        entity: hecs::Entity,
    ) -> SysResult {
        let mut metadata = game.ecs.get_mut::<Metadata>(entity)?;
        metadata.flags.set(EntityBitMask::ON_FIRE, false);
        let entityref = game.ecs.entity(entity)?;
        let network_id = *entityref.get::<NetworkID>()?;
        server.broadcast_nearby_with(*entityref.get::<Position>()?, |client| {
            client.send_entity_metadata(true, network_id, metadata.clone());
        });
        Ok(())
    }

    fn tick(
        &mut self,
        game: &mut crate::game::Game,
        server: &mut Server,
        entity: hecs::Entity,
    ) -> SysResult {
        if self.time_ticks > 0 {
            self.time_ticks -= 1;
            if self.timer <= 0 {
                self.timer = 20;
                let pos = game.ecs.get::<Position>(entity)?;
                let aabb = game.ecs.get::<AABBSize>(entity)?;
                let world = game
                    .worlds
                    .get(&pos.world)
                    .ok_or(anyhow::anyhow!("No world"))?;
                if world.collides_with(&aabb, &pos, self.registry.get_block(51).unwrap()) {
                    self.timer = 10;
                }
                if world.collides_with(&aabb, &pos, self.registry.get_block(8).unwrap())
                    || world.collides_with(&aabb, &pos, self.registry.get_block(9).unwrap())
                {
                    self.time_ticks = 0;
                    return Ok(());
                }

                let is_creative = game.ecs.get::<Gamemode>(entity).map(|v| matches!(*v, Gamemode::Creative)).unwrap_or(false);
                
                let amount_of_damage = if game.ecs.get::<ItemEntity>(entity).is_ok() {
                    20
                } else {
                    1
                };
                if !is_creative {
                    let mut health = game.ecs.get_mut::<Health>(entity)?;
                    health.damage(amount_of_damage, DamageType::Fire);
                }
            } else {
                self.timer -= 1;
            }
        }
        Ok(())
    }

    fn should_remove(&self) -> bool {
        self.time_ticks == 0
    }

    fn show_client(
        &self,
        entityref: &EntityRef,
        client: &crate::server::Client,
    ) -> SysResult {
        let mut metadata = entityref.get_mut::<Metadata>()?;
        metadata.flags.set(EntityBitMask::ON_FIRE, true);
        let network_id = *entityref.get::<NetworkID>()?;
        client.send_entity_metadata(true, network_id, metadata.clone());
        Ok(())
    }
}
impl FireEffect {
    pub fn new(time: u128) -> Self {
        Self {
            time_ticks: time,
            timer: 0,
            registry: ItemRegistry::global(),
        }
    }
}
pub fn is_water(id: u8) -> bool {
    id == 8 || id == 9
}
