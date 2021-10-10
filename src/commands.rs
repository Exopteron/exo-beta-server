use crate::game::Game;
use std::sync::Arc;
use std::any::Any;
pub trait CommandExecutor {
    fn as_any(&mut self) -> &mut dyn Any;
}
#[derive(Clone)]
pub enum CommandArgumentTypes {
    String,
    Int,
}
pub trait CommandArgument {
    fn as_any(&mut self) -> &mut dyn Any;
    fn display(&self) -> String;
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
#[derive(Clone)]
pub struct Command {
    root: String,
    description: String,
    arguments: Vec<CommandArgumentTypes>,
    function: Arc<Box<dyn Fn(&mut Game, &mut dyn CommandExecutor, Vec<Box<dyn CommandArgument>>) -> anyhow::Result<usize>>>,
}
pub struct CommandSystem {
    commands: Vec<Command>,
}
impl Command {
    pub fn new(root: &str, description: &str, arguments: Vec<CommandArgumentTypes>, function: Box<dyn Fn(&mut Game, &mut dyn CommandExecutor, Vec<Box<dyn CommandArgument>>) -> anyhow::Result<usize>>) -> Self {
        Self { root: root.to_string(), description: description.to_string(), arguments, function: Arc::new(function) }
    }
}
impl CommandSystem {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }
    pub fn register(&mut self, command: Command) {
        self.commands.push(command);
    }
    pub fn execute(&mut self, game: &mut Game, executor: &mut dyn CommandExecutor, command: &str) -> anyhow::Result<usize> {
        let command = command.split(" ").collect::<Vec<&str>>();
        if command.len() < 1 {
            return Ok(1);
        }
        let mut cmd: Option<Command> = None;
        for registered in &self.commands {
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
        let mut args: Vec<Box<dyn CommandArgument>> = Vec::new();
        for argument in &cmd.arguments {
            match argument {
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
        (cmd.function)(game, executor, args)
    }
}