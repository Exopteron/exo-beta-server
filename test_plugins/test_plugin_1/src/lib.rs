use exo_beta_server::{plugins::Plugin, declare_plugin, ecs::entities::player::Chatbox, server::Client, item::item::block::Block, events::block_interact::BlockPlacementEvent};
use hecs::Entity;

#[derive(Debug, Default)]
pub struct TestPlugin1;
impl Plugin for TestPlugin1 {
    fn name(&self) -> &'static str {
        "TestPlugin1"
    }
    fn on_load(&self, _game: &mut exo_beta_server::game::Game) {
        
    }
    fn register_items(&self, item_registry: &mut exo_beta_server::item::item::ItemRegistry) {
        item_registry.register_block(CactusBlock);
    }
}

pub struct CactusBlock;
impl Block for CactusBlock {
    fn id(&self) -> exo_beta_server::item::item::BlockIdentifier {
        81
    }

    fn item_stack_size(&self) -> i8 {
        println!("Called");
        64
    }
    fn place(&self, game: &mut exo_beta_server::game::Game, placer: Entity, item: exo_beta_server::item::stack::ItemStack, mut position: exo_beta_server::game::BlockPosition, face: exo_beta_server::protocol::packets::Face, world: i32) -> Option<exo_beta_server::events::block_interact::BlockPlacementEvent> {
        println!("Placement");
        log::info!("Yoo!");
        //game.ecs.get_mut::<Chatbox>(placer).unwrap().send_message("Balls".into());
        game.broadcast_chat("Sup losers!");
        position = face.offset(position);
        Some(BlockPlacementEvent {
            held_item: item,
            location: position,
            face,
            world,
        })
    }
}


declare_plugin!(TestPlugin1, TestPlugin1::default);