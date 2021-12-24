use std::io::{self, BufRead};

use flume::Receiver;

use crate::{game::Game, server::Server};

use super::{SystemExecutor, SysResult, chat::Console};

pub struct AsyncStdinAcceptor {
    pub receiver: Receiver<String>
}
impl AsyncStdinAcceptor {
    pub fn start() -> Self {
        let (sender, receiver) = flume::unbounded();
        std::thread::Builder::new().name("stdin-thread".to_string()).spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                if let Ok(line) = line {
                    sender.send(line).expect("stdin channel send failure");
                }
            }
        }).expect("Error creating stdin thread");
        Self { receiver }
    }
}
pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems.group::<Server>().add_system(accept_stdin);
    game.insert_object(AsyncStdinAcceptor::start());
}

pub fn accept_stdin(game: &mut Game, server: &mut Server) -> SysResult {
    let stdin = game.objects.get::<AsyncStdinAcceptor>()?;
    let mut console = None;
    for (entity, _) in game.ecs.query::<&Console>().iter() {
        console = Some(entity);
        break;
    }
    let recieved = stdin.receiver.try_iter().collect::<Vec<String>>();
    drop(stdin);
    for string in recieved {
        let res = game.execute_command(server, &string, console.unwrap());
        if let Ok(c) = res {
            if let Some(message) = crate::commands::code_to_message(c) {
                log::info!("{}", message);
            }
        } else if let Err(e) = res {
            log::info!("§cAn internal error occured.");
            log::error!("Command error: {:?}", e);
        }
    }
    Ok(())
}