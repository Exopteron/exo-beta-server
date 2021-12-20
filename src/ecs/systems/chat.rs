// Feather license at FEATHER_LICENSE.md
use hecs::EntityBuilder;

use crate::{ecs::{systems::SystemExecutor, entities::player::Chatbox}, game::Game, server::Server, network::ids::NetworkID};

use super::SysResult;

struct Console;
pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    let mut console = EntityBuilder::new();
    console.add(Console).add(Chatbox::new());
    game.ecs.spawn(console.build());
    systems.add_system(flush_console_chat_box);
    systems.group::<Server>().add_system(flush_chat_boxes);
}

/// Flushes players' chat mailboxes and sends the needed packets.
fn flush_chat_boxes(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, (&client_id, mailbox)) in game.ecs.query::<(&NetworkID, &mut Chatbox)>().iter() {
        if let Some(client) = server.clients.get(&client_id) {
            for message in mailbox.drain() {
                client.send_chat_message(message);
            }
        }
    }

    Ok(())
}

/// Prints chat messages to the console.
fn flush_console_chat_box(game: &mut Game) -> SysResult {
    for (_, (_console, mailbox)) in game.ecs.query::<(&Console, &mut Chatbox)>().iter() {
        for message in mailbox.drain() {
            log::info!("[CHAT]: {}", message.0);
        }
    }

    Ok(())
}