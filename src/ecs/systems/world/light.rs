use std::collections::VecDeque;

use crate::{
    block_entity::{BlockEntity, BlockEntityLoader},
    configuration::CONFIGURATION,
    ecs::{
        entities::player::Username,
        systems::{SysResult, SystemExecutor},
    },
    entities::EntityInit,
    events::block_change::BlockChangeEvent,
    game::{BlockPosition, Game, Position, ChunkCoords},
    item::item::ItemRegistry,
    protocol::packets::Face,
    server::Server,
    world::{
        chunks::{BlockState, SECTION_VOLUME},
        light::{GameCommand, LightPropagator, LightThreadManager, LightThreadSync},
    },
};
use ahash::AHashMap;
#[derive(Debug)]
pub enum LightPropagationRequest {
    ChunkSky {
        position: ChunkCoords,
        world: i32,
    }
}
pub struct LightPropagationManager {
    manager: LightThreadSync,
    queue: VecDeque<GameCommand>,
}
impl LightPropagationManager {
    pub fn new() -> Self {
        let (manager, sync) = LightThreadManager::new();
        rayon::spawn(move || manager.run());
        Self {
            manager: sync,
            queue: VecDeque::new(),
        }
    }
    pub fn push(&mut self, request: LightPropagationRequest) {
        self.manager.sender.send(request).expect("handle later");
    }
    // pub fn pop(&mut self, mut amount: usize) -> Vec<LightPropagationRequest> {
    //     let mut requests = Vec::new();
    //     let len = self.requests.len();
    //     if amount > len {
    //         amount = len;
    //     }
    //     for _ in 0..amount {
    //         if let Some(r) = self.requests.pop_front() {
    //             requests.push(r);
    //         }
    //     }
    //     requests
    // }
    // pub fn push(&mut self, request: LightPropagationRequest) {
    //     self.requests.push_front(request);
    // }
}
pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    game.insert_object(LightPropagationManager::new());
    systems.add_system(handle_commands);
}
pub fn handle_commands(game: &mut Game) -> SysResult {
    let obj = game.objects.clone();
    let mut propagator = obj.get_mut::<LightPropagationManager>()?;
    for _ in 0..2500 {
        let command;
        if let Ok(cmd) = propagator.manager.receiver.try_recv() {
            command = cmd;
        } else if let Some(v) = propagator.queue.pop_front() {
            command = v;
        } else {
            break;
        }
        match command {
            GameCommand::GetBlock { position, recv } => {
                //log::info!("Getblock");
                let block = game.block(position, position.world);
                recv.send(block).expect("handle later");
            }
            GameCommand::SetBlockLight {
                position,
                state,
                recv,
            } => {
                //log::info!("Sbl");
                let block = game.block(position, position.world);
                if let Some(mut block) = block {
                    block.b_light = state.b_light;
                    block.b_skylight = state.b_skylight;
                    game.set_block_nb(position, block, position.world, false, true);
                }
                recv.send(()).expect("later");
            }
            GameCommand::GetChunk { position, recv } => {
                //log::info!("Got get chunk");
                let world = game.worlds.get(&position.world).unwrap();
                recv.send(world.chunk_map.chunk_handle_at(position))
                    .expect("handle later");
            }
        }
    }
    Ok(())
}
// pub fn propagate_lighting(game: &mut Game) -> SysResult {
//     let mut propagator = game.objects.get_mut::<LightPropagationManager>()?;
//     let requests: Vec<LightPropagationRequest> = Vec::new(); // propagator.pop(CONFIGURATION.light_prop_per_tick);
//     drop(propagator);
//     let mut propagator = LightPropagator::new(true); // TODO: change in future for regular lighting
//     for request in requests {
//         let block_id = game.block_id_at(request.position.clone());
//         //log::info!("Propagating {:?} which is a {}", request, block_id);
//         propagator.increase_light(request.world, game, request.position, request.level);
//     }
//     Ok(())
// }
