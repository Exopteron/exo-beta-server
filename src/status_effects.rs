use std::{collections::VecDeque, sync::Arc, mem};

use hecs::Entity;

use crate::{game::Game, ecs::{systems::SysResult, EntityRef}, server::{Server, Client}};
pub mod fire;
pub mod poison;

pub trait StatusEffect: Sync + Send {
    fn on_apply(&mut self, game: &mut Game, server: &mut Server, entity: Entity) -> SysResult;
    fn on_remove(&mut self, game: &mut Game, server: &mut Server, entity: Entity) -> SysResult;
    fn tick(&mut self, game: &mut Game, server: &mut Server, entity: Entity) -> SysResult;
    fn show_client(&self, us: &EntityRef, client: &Client) -> SysResult;
    fn should_remove(&self) -> bool;
}
pub type StatusEffectObj = Box<dyn StatusEffect>;
#[derive(Default)]
pub struct StatusEffectsManager {
    new_effects: VecDeque<StatusEffectObj>,
    applied_effects: Vec<StatusEffectObj>,
}

impl StatusEffectsManager {
    pub fn reset(&mut self) {
        self.new_effects = VecDeque::new();
        self.applied_effects = Vec::new();
    }
    pub fn get_effects(&self) -> &Vec<StatusEffectObj> {
        &self.applied_effects
    }
    pub fn add_effect(&mut self, effect: impl StatusEffect + 'static) {
        self.new_effects.push_back(Box::new(effect));
    }
    pub fn system(game: &mut Game, server: &mut Server) -> SysResult {
        let mut to_check = Vec::new();
        for (entity, _) in game.ecs.query::<&StatusEffectsManager>().iter() {
            to_check.push(entity);
        }
        for entity in to_check {
            let mut manager = game.ecs.get_mut::<StatusEffectsManager>(entity)?;
            let mut new_effects = mem::take(&mut manager.new_effects);
            let mut to_apply = Vec::new();
            drop(manager);
            while let Some(mut effect) = new_effects.pop_front() {
                effect.on_apply(game, server, entity)?;
                to_apply.push(effect);
            }
            let mut manager = game.ecs.get_mut::<StatusEffectsManager>(entity)?;
            for effect in to_apply {
                manager.applied_effects.push(effect);
            }
            let mut to_remove = Vec::new();
            for (i, effect) in manager.applied_effects.iter().enumerate() {
                if effect.should_remove() {
                    to_remove.push(i);
                }
            }
            let mut applied_effects = mem::take(&mut manager.applied_effects);
            drop(manager);
            for i in to_remove {
                if applied_effects.len() >= i {
                    let mut effect = applied_effects.remove(i);
                    effect.on_remove(game, server, entity)?;
                }
            }
            let mut manager = game.ecs.get_mut::<StatusEffectsManager>(entity)?;
            manager.applied_effects = applied_effects;
            let mut effects = mem::take(&mut manager.applied_effects);
            drop(manager);
            for effect in effects.iter_mut() {
                effect.tick(game, server, entity)?;
            }
            let mut manager = game.ecs.get_mut::<StatusEffectsManager>(entity)?;
            manager.applied_effects = effects;
        }
        Ok(())
    }
}