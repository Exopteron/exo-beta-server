use hecs::Entity;

use crate::{
    ecs::{
        entities::player::{Sneaking, Sprinting, Sleeping},
        systems::SysResult,
    },
    entities::metadata::EntityBitMask,
    events::{SneakEvent, SprintEvent},
    game::Game,
    network::metadata::Metadata,
    protocol::packets::{client::EntityAction, EntityActionKind},
};

///  From [wiki](https://wiki.vg/Protocol#Entity_Action)
///  Sent by the client to indicate that it has performed certain actions:
///  *) sneaking (crouching),
///  *) sprinting,
///  *) exiting a bed,
///  *) jumping with a horse,
///  *) opening a horse's inventory while riding it.

pub fn handle_entity_action(game: &mut Game, player: Entity, packet: EntityAction) -> SysResult {
    match packet.action_id {
        EntityActionKind::StartSneaking => {
            let is_sneaking = game.ecs.get_mut::<Sneaking>(player)?.0;
            if !is_sneaking {
                game.ecs
                    .insert_entity_event(player, SneakEvent::new(true))?;
                game.ecs.get_mut::<Sneaking>(player)?.0 = true;
            }
        }
        EntityActionKind::StopSneaking => {
            let is_sneaking = game.ecs.get_mut::<Sneaking>(player)?.0;
            if is_sneaking {
                game.ecs
                    .insert_entity_event(player, SneakEvent::new(false))?;
                game.ecs.get_mut::<Sneaking>(player)?.0 = false;
            }
        }
        EntityActionKind::LeaveBed => {
            println!("Leave bed");
            let mut sleeping = game.ecs.get_mut::<Sleeping>(player)?;
            sleeping.unset_sleeping();
        }
        EntityActionKind::StartSprinting | EntityActionKind::StopSprinting => {
            let start_sprinting = matches!(packet.action_id, EntityActionKind::StartSprinting);
            let is_sprinting = game.ecs.get_mut::<Sprinting>(player)?.0;
            if is_sprinting != start_sprinting {
                game.ecs
                    .insert_entity_event(player, SprintEvent::new(start_sprinting))?;
                game.ecs.get_mut::<Sprinting>(player)?.0 = start_sprinting;
                let mut meta = game.ecs.get_mut::<Metadata>(player)?;
                meta.flags.set(EntityBitMask::SPRINTING, start_sprinting);
                meta.dirty = true;
            }
        }
    }

    Ok(())
}
