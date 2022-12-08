use std::{collections::VecDeque, sync::Arc, mem, any::{TypeId, Any}};

use hecs::Entity;
use rustc_hash::FxHashMap;

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
    new_effects: VecDeque<(TypeId, StatusEffectObj)>,
    applied_effects: FxHashMap<TypeId, StatusEffectObj>,
}

impl StatusEffectsManager {
    pub fn reset(&mut self) {
        self.new_effects = VecDeque::new();
        self.applied_effects = FxHashMap::default();
    }
    pub fn get_effects(&self) -> &FxHashMap<TypeId, StatusEffectObj> {
        &self.applied_effects
    }
    pub fn add_effect<T: StatusEffect + 'static>(&mut self, effect: T) {
        self.new_effects.push_back((TypeId::of::<T>(), Box::new(effect)));
    }
    pub fn get_effect<T: StatusEffect + 'static>(&mut self) -> Option<&mut T> {
        let v = self.applied_effects.get_mut(&TypeId::of::<T>())?;
        let v = v as &mut dyn Any;
        v.downcast_mut::<T>()
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
            while let Some((id, mut effect)) = new_effects.pop_front() {
                effect.on_apply(game, server, entity)?;
                to_apply.push((id, effect));
            }
            let mut manager = game.ecs.get_mut::<StatusEffectsManager>(entity)?;
            for (id, effect) in to_apply {
                manager.applied_effects.insert(id, effect);
            }
            let mut to_remove = Vec::new();
            for (i, (id, effect)) in manager.applied_effects.iter().enumerate() {
                if effect.should_remove() {
                    to_remove.push(*id);
                }
            }
            let mut applied_effects = mem::take(&mut manager.applied_effects);
            drop(manager);
            for i in to_remove {
                let mut effect = applied_effects.remove(&i);
                if let Some(mut effect) = effect {
                    effect.on_remove(game, server, entity)?;
                }
            }
            let mut manager = game.ecs.get_mut::<StatusEffectsManager>(entity)?;
            manager.applied_effects = applied_effects;
            let mut effects = mem::take(&mut manager.applied_effects);
            drop(manager);
            for (id, effect) in effects.iter_mut() {
                effect.tick(game, server, entity)?;
            }
            let mut manager = game.ecs.get_mut::<StatusEffectsManager>(entity)?;
            manager.applied_effects = effects;
        }
        Ok(())
    }
}