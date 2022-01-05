use hecs::Entity;

use crate::{protocol::packets::{client::EntityAction, EntityActionKind}, game::Game, ecs::{entities::player::{Sneaking, Sprinting}, systems::SysResult}, events::{SneakEvent, SprintEvent}};

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
            
        }
        EntityActionKind::StartSprinting | EntityActionKind::StopSprinting => {
            let start_sprinting = matches!(packet.action_id, EntityActionKind::StartSprinting);
            let is_sprinting = game.ecs.get_mut::<Sprinting>(player)?.0;
            if is_sprinting != start_sprinting {
                game.ecs
                    .insert_entity_event(player, SprintEvent::new(start_sprinting))?;
                game.ecs.get_mut::<Sprinting>(player)?.0 = start_sprinting;
            }
        }
    }

    Ok(())
}
