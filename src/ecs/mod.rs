// feather
use hecs::{Component, ComponentError, DynamicBundle, Entity, MissingComponent, NoSuchEntity, Query, QueryBorrow, Ref, RefMut, World};
pub mod entities;
pub mod systems;
use std::io::BufRead;
pub struct Ecs {
    pub world: World,
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
        }
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
