
use std::collections::VecDeque;

use ahash::AHashMap;
use crate::{ecs::{systems::{SystemExecutor, SysResult}, entities::player::Username}, game::{Game, Position, BlockPosition}, server::Server, world::{chunks::{SECTION_VOLUME, BlockState}, light::LightPropagator}, events::block_change::BlockChangeEvent, protocol::packets::Face, item::item::ItemRegistry, block_entity::{BlockEntity, BlockEntityLoader}, entities::EntityInit, configuration::CONFIGURATION};
#[derive(Debug)]
pub struct LightPropagationRequest {
    pub position: BlockPosition,
    pub world: i32,
    pub level: u8,
    pub skylight: bool
}
#[derive(Default)]
pub struct LightPropagationManager {
    requests: VecDeque<LightPropagationRequest>
}
impl LightPropagationManager {
    pub fn pop(&mut self, mut amount: usize) -> Vec<LightPropagationRequest> {
        let mut requests = Vec::new();
        let len = self.requests.len();
        if amount > len {
            amount = len;
        }
        for _ in 0..amount {
            if let Some(r) = self.requests.pop_front() {
                requests.push(r);
            }
        }
        requests
    }
    pub fn push(&mut self, request: LightPropagationRequest) {
        self.requests.push_front(request);
    }
}
pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    game.insert_object(LightPropagationManager::default());
    systems.add_system(propagate_lighting);
}

pub fn propagate_lighting(game: &mut Game) -> SysResult {
    let mut propagator = game.objects.get_mut::<LightPropagationManager>()?;
    let requests = propagator.pop(CONFIGURATION.light_prop_per_tick);
    drop(propagator);
    let mut propagator = LightPropagator::new(true); // TODO: change in future for regular lighting
    for request in requests {
        let block_id = game.block_id_at(request.position.clone());
        //log::info!("Propagating {:?} which is a {}", request, block_id);
        propagator.increase_light(request.world, game, request.position, request.level);
    }
    Ok(())
}