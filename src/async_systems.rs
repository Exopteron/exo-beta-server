use super::*;
use crate::{commands::CommandExecutor, game::*};
use flume::{Receiver, Sender};
use std::{any::Any, io::Write, sync::Arc};
pub enum AsyncGameCommand {
    ScheduleSyncTask {
        func: Arc<Box<dyn Fn(&mut Game) -> Option<()> + Sync + Send>>,
    },
}
pub struct ConsoleCommandExecutor {}
impl CommandExecutor for ConsoleCommandExecutor {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn send_message(&mut self, message: Message) {
        log::info!("[Console CHAT] {}", message.message);
    }
    fn permission_level(&self) -> u8 {
        5
    }
    fn username(&self) -> String {
        String::from("CONSOLE")
    }
}
pub async fn setup_async_systems(command_sender: Sender<AsyncGameCommand>) {
    let sender_2 = command_sender.clone();
    std::thread::spawn(move || {
        log::info!("[Console thread] Started");
        loop {
            let mut line = String::new();
            if let Err(e) = std::io::stdin().read_line(&mut line) {
                log::error!("[Console thread] Error reading line from stdin: {:?}", e);
            }
            if let Err(e) = sender_2.send(AsyncGameCommand::ScheduleSyncTask {
                func: Arc::new(Box::new(move |game| {
                    if let Ok(code) =
                        game.execute_command(&mut ConsoleCommandExecutor {}, line.clone().trim())
                    {
                        if let Some(msg) = crate::commands::code_to_message(code) {
                            log::info!("{}", msg);
                        }
                        /*                     match code {
                            0 => {}
                            1 => {
                                log::info!("§7Bad syntax.");
                            }
                            4 => {
                                log::info!("§7Unknown command.");
                            }
                            5 => {
                                log::info!("§4Insufficient permission. (Somehow?)");
                            }
                            3 => {}
                            res => {
                                log::info!("§7Command returned code {}.", res);
                            }
                        } */
                        std::io::stdout().write(b"> ").expect("handle later");
                        std::io::stdout().flush().expect("handle later");
                    } else {
                        log::error!("Error executing command");
                    }
                    None
                })),
            }) {
                log::error!("[Console thread] Error executing command: {:?}", e);
            }
        }
    });
    /*     tokio::spawn(async move {
        loop {
            command_sender.send_async(AsyncGameCommand::ScheduleSyncTask { func: Arc::new(Box::new(|game| {
                log::info!("Sugmanuts");
                None
            }))}).await.expect("Impossible!");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }); */
}
pub mod chat {
    use std::{
        collections::HashMap,
        hash::{Hash, Hasher},
        sync::Arc,
    };

    use flume::{Receiver, Sender};

    use crate::{
        game::Message,
        network::packet::{ChatMessage, ClientPacket, ServerPacket},
    };

    use super::AsyncGameCommand;

    pub enum AsyncChatCommand {
        RegisterUser { user: AsyncChatClient, name: String },
        RemoveUser { name: String },
        ChatToUser { name: String, message: Message },
    }
    pub struct AsyncChatClient {
        pub sender: Sender<ServerPacket>,
        pub receiver: Receiver<ClientPacket>,
    }
    pub struct AsyncChatManager {
        game_sender: Sender<AsyncGameCommand>,
        recv: Receiver<AsyncChatCommand>,
        clients: HashMap<String, AsyncChatClient>,
    }
    impl AsyncChatManager {
        pub fn new(
            game_sender: Sender<AsyncGameCommand>,
            recv: Receiver<AsyncChatCommand>,
        ) -> Self {
            Self {
                game_sender,
                recv,
                clients: HashMap::new(),
            }
        }
        pub async fn run(mut self) {
            tokio::spawn(async move {
                loop {
                    self.handle_connections().await;
                    self.handle_chat().await;
                    self.gc().await;
                    tokio::time::sleep(std::time::Duration::from_millis(15)).await;
                }
            });
        }
        pub async fn gc(&mut self) {
            self.clients.retain(|_, client| {
                if client.sender.is_disconnected() || client.receiver.is_disconnected() {
                    return false;
                }
                true
            });
        }
        pub async fn handle_connections(&mut self) {
            for command in self.recv.try_iter() {
                match command {
                    AsyncChatCommand::RegisterUser { user, name } => {
                        self.clients.insert(name, user);
                    }
                    AsyncChatCommand::RemoveUser { name } => {
                        self.clients.remove(&name);
                    }
                    AsyncChatCommand::ChatToUser { name, message } => {
                        if let Some(client) = self.clients.get(&name) {
                            if let Err(e) = client.sender.send(ServerPacket::ChatMessage {
                                message: message.message,
                            }) {
                                log::info!("Error sending async chat message: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
        pub async fn handle_chat(&mut self) {
            let mut messages = Vec::new();
            for (name, client) in self.clients.iter() {
                for packet in client.receiver.try_iter() {
                    match packet {
                        ClientPacket::ChatMessage(mut packet) => {
                            if packet.message.starts_with("/") {
                                packet.message.remove(0);
                                let packet = packet.clone();
                                let name = name.clone();
                                self.game_sender
                                    .send_async(AsyncGameCommand::ScheduleSyncTask {
                                        func: Arc::new(Box::new(move |game| {
                                            let mut player =
                                                game.players.get_player(&(name.clone()))?;
                                            log::info!(
                                                "{} issued server command \"/{}\".",
                                                player.get_username(),
                                                packet.message
                                            );
                                            //log::debug!("A");
                                            let res = game
                                                .execute_command(&mut player, &packet.message)
                                                .ok()?;
                                            //log::debug!("B");
                                            if let Some(msg) = crate::commands::code_to_message(res)
                                            {
                                                log::info!("{}", msg);
                                                player.send_message(Message::new(&msg));
                                            }
                                            None
                                        })),
                                    })
                                    .await
                                    .expect("this shouldn't happen");
                            } else {
                                let message =
                                    Message::new(&format!("<{}> {}", name, packet.message));
                                log::info!("[Async Chat Task] {}", message.message);
                                messages.push(message);
                            }
                        }
                        _ => {
                            log::info!("Recieved other");
                        }
                    }
                }
            }
            for message in messages {
                for (_, client) in self.clients.iter() {
                    //log::info!("Sending");
                    if let Err(e) = client.sender.send(ServerPacket::ChatMessage {
                        message: message.message.to_string(),
                    }) {
                        log::error!("Error handling async chat event: {:?}", e);
                    }
                }
            }
        }
    }
}
