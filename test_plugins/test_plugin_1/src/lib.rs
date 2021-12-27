use exo_beta_server::{
    commands::Command,
    declare_plugin,
    ecs::entities::player::{Chatbox, Username, Player},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Position},
    item::item::block::Block,
    plugins::{hecs::Entity, Plugin, PluginVTable},
    server::Client,
    world::chunks::BlockState,
    PluginVTable_static,
};
#[derive(Debug, Default)]
pub struct TestPlugin1;
impl Plugin for TestPlugin1 {
    fn name(&self) -> &'static str {
        "TestPlugin1"
    }
    fn register_items(&self, item_registry: &mut exo_beta_server::item::item::ItemRegistry) {
        item_registry.register_block(CactusBlock);
    }

    fn on_unload(&self) {}

    fn register_commands(&self, command_registry: &mut exo_beta_server::commands::CommandSystem) {
        command_registry.register(Command::new(
            "testcommand",
            "test",
            4,
            vec![],
            Box::new(|game, server, executor, args| {
                game.broadcast_chat("Sup! I run");
                Ok(0)
            }),
        ));
    }

    fn on_load(&self, game: &mut Game, systems: &mut SystemExecutor<Game>) {
        systems.add_system(|game| {
            let mut positions = Vec::new();
            let mut q = game.ecs.query::<(&Position, &Player)>();
            for (_, (pos, _)) in q.iter() {
                println!("Got pos {:?}", pos);
                positions.push(pos.clone());
            }
            drop(q);
            for pos in positions {
                println!("Setting {}", BlockPosition::new(pos.x as i32, pos.y as i32, pos.z as i32));
                game.set_block(
                    BlockPosition::new(pos.x as i32, pos.y as i32, pos.z as i32),
                    BlockState::from_id(7),
                    0,
                );
            }
            Ok(())
        });
    }
}

pub struct CactusBlock;
impl Block for CactusBlock {
    fn id(&self) -> exo_beta_server::item::item::BlockIdentifier {
        81
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn place(
        &self,
        game: &mut exo_beta_server::game::Game,
        placer: Entity,
        item: exo_beta_server::item::stack::ItemStack,
        mut position: exo_beta_server::game::BlockPosition,
        face: exo_beta_server::protocol::packets::Face,
        world: i32,
    ) -> Option<exo_beta_server::events::block_interact::BlockPlacementEvent> {
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
