// Derived from feather-rs. License can be found in FEATHER_LICENSE.md
use hecs::{
    Component, ComponentError, DynamicBundle, Entity, MissingComponent, NoSuchEntity, Query,
    QueryBorrow, Ref, RefMut, World,
};
pub mod entities;
pub mod event;
pub mod systems;
use std::{any::type_name, io::BufRead, marker::PhantomData, sync::Arc};

use crate::objects::Resources;

use self::event::EventTracker;
pub struct Ecs {
    world: World,
    event_tracker: EventTracker,
}
pub struct EntityRef<'a>(hecs::EntityRef<'a>);

impl<'a> EntityRef<'a> {
    /// Borrows the component of type `T` from this entity.
    pub fn get<T: Component>(&self) -> Result<Ref<'a, T>, ComponentError> {
        self.0
            .get()
            .ok_or_else(|| ComponentError::MissingComponent(MissingComponent::new::<T>()))
    }

    /// Uniquely borrows the component of type `T` from this entity.
    pub fn get_mut<T: Component>(&self) -> Result<RefMut<'a, T>, ComponentError> {
        self.0
            .get_mut()
            .ok_or_else(|| ComponentError::MissingComponent(MissingComponent::new::<T>()))
    }
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            event_tracker: EventTracker::default(),
        }
    }
    /// Creates an event not related to any entity. Use
    /// `insert_entity_event` for events regarding specific
    /// entities (`PlayerJoinEvent`, `EntityDamageEvent`, etc...)
    pub fn insert_event<T: Component>(&mut self, event: T) {
        let entity = self.world.spawn((event,));
        self.event_tracker.insert_event(entity);
    }
    /// Sets the index of the currently executing system,
    /// used for event tracking.
    pub fn set_current_system_index(&mut self, index: usize) {
        self.event_tracker.set_current_system_index(index);
    }
    /// Adds a component to an entity.
    ///
    /// Do not use this function to add events. Use [`Ecs::insert_event`]
    /// instead.
    pub fn insert(
        &mut self,
        entity: Entity,
        component: impl Component,
    ) -> Result<(), NoSuchEntity> {
        self.world.insert_one(entity, component)
    }
    /// Defers removing an entity until before the next time this system
    /// runs, allowing it to be observed by systems one last time.
    pub fn defer_despawn(&mut self, entity: Entity) {
        // a bit of a hack - but this will change once
        // hecs allows taking out components of a despawned entity
        self.event_tracker.insert_event(entity);
    }
    /// Adds an event component to an entity and schedules
    /// it to be removed immeditately before the current system
    /// runs again. Thus, all systems have exactly one chance
    /// to observe the event before it is dropped.
    pub fn insert_entity_event<T: Component>(
        &mut self,
        entity: Entity,
        event: T,
    ) -> Result<(), NoSuchEntity> {
        self.insert(entity, event)?;
        self.event_tracker.insert_entity_event::<T>(entity);
        Ok(())
    }
    /// Should be called before each system runs.
    pub fn remove_old_events(&mut self) {
        self.event_tracker.remove_old_events(&mut self.world);
    }

    pub fn spawn(&mut self, c: impl DynamicBundle) -> Entity {
        self.world.spawn(c)
    }
    /// Returns an `EntityRef` for an entity.
    pub fn entity(&self, entity: Entity) -> Result<EntityRef, NoSuchEntity> {
        self.world.entity(entity).map(EntityRef)
    }

    /// Gets a component of an entity.
    pub fn get<T: Component>(&self, entity: Entity) -> Result<Ref<T>, ComponentError> {
        self.world.get(entity)
    }

    /// Mutably gets a component of an entity.
    pub fn get_mut<T: Component>(&self, entity: Entity) -> Result<RefMut<T>, ComponentError> {
        self.world.get_mut(entity)
    }

    /// Returns an iterator over all entities that match a query parameter.
    pub fn query<Q: Query>(&self) -> QueryBorrow<Q> {
        self.world.query()
    }
}

/// A type containing an `Ecs`.
pub trait HasEcs {
    fn ecs(&self) -> &Ecs;

    fn ecs_mut(&mut self) -> &mut Ecs;
}

impl HasEcs for Ecs {
    fn ecs(&self) -> &Ecs {
        self
    }

    fn ecs_mut(&mut self) -> &mut Ecs {
        self
    }
}

/// A type containing a `Resources`.
pub trait HasResources {
    fn resources(&self) -> Arc<Resources>;
}
