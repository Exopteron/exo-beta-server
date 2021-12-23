use hecs::Entity;

use crate::game::Game;
use crate::server::Server;
use std::any::Any;
use std::sync::Arc;
/*
Command codes:
1 = bad syntax
3 = generic error
4 = unknown command
5 = bad permissions
*/
#[derive(Clone, Debug)]
pub enum CommandArgumentTypes {
    StringRest,
    String,
    Int,
}
pub struct PermissionLevel(pub u8);
pub trait CommandArgument {
    fn as_any(&mut self) -> &mut dyn Any;
    fn display(&self) -> String;
    fn as_int(&mut self) -> i32 {
        match self.as_any().downcast_ref() {
            Some(i) => *i,
            None => {
                panic!("Not int");
            }
        }
    }
}
impl CommandArgument for String {
    fn display(&self) -> String {
        format!("{}", self)
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
impl CommandArgument for i32 {
    fn display(&self) -> String {
        format!("{}", self)
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
impl CommandArgument for Vec<String> {
    fn display(&self) -> String {
        format!("{:?}", self)
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
#[derive(Clone)]
pub struct Command {
    pub root: String,
    pub description: String,
    pub arguments: Vec<CommandArgumentTypes>,
    pub perm_level: u8,
    function: Arc<
        Box<
            dyn Fn(
                &mut Game,
                &mut Server,
                Entity,
                Vec<Box<dyn CommandArgument>>,
            ) -> anyhow::Result<usize>,
        >,
    >,
}
pub struct CommandSystem {
    pub commands: Vec<Command>,
}
impl Command {
    pub fn new(
        root: &str,
        description: &str,
        perm_level: u8,
        arguments: Vec<CommandArgumentTypes>,
        function: Box<
            dyn Fn(
                &mut Game,
                &mut Server,
                Entity,
                Vec<Box<dyn CommandArgument>>,
            ) -> anyhow::Result<usize>,
        >,
    ) -> Self {
        Self {
            root: root.to_string(),
            description: description.to_string(),
            arguments,
            function: Arc::new(function),
            perm_level,
        }
    }
}
pub fn code_to_message(res: usize) -> Option<String> {
    //log::debug!("B");
    match res {
        0 => {}
        1 => {
            return Some(String::from("§7Bad syntax."));
            //player.send_message(Message::new(&format!("§7Bad syntax.")));
        }
        4 => {
            return Some(String::from("§7Unknown command."));
           // player.send_message(Message::new(&format!("§7Unknown command.")));
        }
        5 => {
            return Some(String::from("§4Insufficient permission."));
            //log::info!("§4Insufficient permission.");
            //player.send_message(Message::new(&format!("§4Insufficient permission.")));
        }
        3 => {}
        res => {
            return Some(format!("§7Command returned code {}.", res));
            //player.send_message(Message::new(&format!("§7Command returned code {}.", res)));
        }
    }
    None
}
impl CommandSystem {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
    pub fn register(&mut self, command: Command) {
        self.commands.push(command);
    }
    pub fn execute(
        &mut self,
        game: &mut Game,
        server: &mut Server,
        executor: Entity,
        command: &str,
    ) -> anyhow::Result<usize> {
        let command = command.split(" ").collect::<Vec<&str>>();
        if command.len() < 1 {
            return Ok(1);
        }
        let mut cmd: Option<Command> = None;
        for registered in &self.commands {
            //log::info!("cmd {} {}", registered.root, command[0]);
            if registered.root == command[0] {
                //log::info!("is");
                cmd = Some(registered.clone());
                break;
            }
        }
        if cmd.is_none() {
            //log::info!("none");
            return Ok(4);
        }
        let mut argselect = 1;
        let cmd = cmd.unwrap();
        if game.ecs.get::<PermissionLevel>(executor)?.0 < cmd.perm_level {
            return Ok(5);
        }
        let mut args: Vec<Box<dyn CommandArgument>> = Vec::new();
        for argument in &cmd.arguments {
            match argument {
                CommandArgumentTypes::StringRest => {
                    let mut epic_args = Vec::new();
                    loop {
                        if let Some(arg) = command.get(argselect) {
                            epic_args.push(arg.to_string());
                            argselect += 1;
                        } else {
                            break;
                        }
                    }
                    args.push(Box::new(epic_args));
                }
                CommandArgumentTypes::String => {
                    if let Some(arg) = command.get(argselect) {
                        args.push(Box::new(arg.to_string()));
                        argselect += 1;
                    } else {
                        return Ok(1);
                    }
                }
                CommandArgumentTypes::Int => {
                    if let Some(arg) = command.get(argselect) {
                        args.push(Box::new(match i32::from_str_radix(arg, 10) {
                            Ok(int) => int,
                            Err(_) => {
                                return Ok(3);
                            }
                        }));
                        argselect += 1;
                    } else {
                        return Ok(1);
                    }
                }
            }
        }
        (cmd.function)(game, server, executor, args)
    }
}
