use std::collections::HashMap;
use std::{
    any::{type_name, Any, TypeId},
    cell::{Ref, RefCell, RefMut},
};

use ahash::AHashMap;
#[derive(Default)]
pub struct Resources {
    objects: AHashMap<TypeId, RefCell<Box<dyn Any>>>
}
impl Resources {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert<T: 'static>(&mut self, resource: T) -> Option<T> {
        self.objects
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(resource)))
            .map(|resource| *resource.into_inner().downcast::<T>().unwrap())
    }
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.objects
            .remove(&TypeId::of::<T>())
            .map(|resource| *resource.into_inner().downcast::<T>().unwrap())
    }
    pub fn get<T: 'static>(&self) -> anyhow::Result<Ref<T>> {
        let resource = self
            .objects
            .get(&TypeId::of::<T>())
            .ok_or_else(|| anyhow::anyhow!("{}", type_name::<T>()))?;

        resource
            .try_borrow()
            .map_err(|_| anyhow::anyhow!("{}", type_name::<T>()))
            .map(|b| Ref::map(b, |b| b.downcast_ref::<T>().unwrap()))
    }
    pub fn get_mut<T: 'static>(&self) -> anyhow::Result<RefMut<T>> {
        let resource = self
            .objects
            .get(&TypeId::of::<T>())
            .ok_or_else(|| anyhow::anyhow!("{}", type_name::<T>()))?;

        resource
            .try_borrow_mut()
            .map_err(|_| anyhow::anyhow!("{}", type_name::<T>()))
            .map(|b| RefMut::map(b, |b| b.downcast_mut::<T>().unwrap()))
    }
}