use std::{any::Any, ffi::OsStr};

use libloading::{Library, Symbol};

use crate::{ecs::entities::player::{Player, Chatbox}, server::Client, item::item::ItemRegistry, game::Game};

pub trait Plugin: Any + Send + Sync {
    fn name(&self) -> &'static str;

    fn on_load(&self, _game: &mut Game) {}
    fn on_unload(&self) {}
    fn register_items(&self, _item_registry: &mut ItemRegistry) {}
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut $crate::plugins::Plugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<$crate::plugins::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}


pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    loaded_libraries: Vec<Library>,
}

impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
        }
    }
    pub fn register_items(&mut self, registry: &mut ItemRegistry) {
        for plugin in &mut self.plugins {
            plugin.register_items(registry);
        }
    }
    pub fn load_all(&mut self, game: &mut Game) {
        for plugin in &mut self.plugins {
            plugin.on_load(game);
        }
    }
    pub unsafe fn load_plugin<P: AsRef<OsStr>>(&mut self, filename: P) -> anyhow::Result<()> {
        type PluginCreate = unsafe fn() -> *mut Plugin;

        let lib = Library::new(filename.as_ref()).or_else(|_| Err(anyhow::anyhow!("Unable to load the plugin")))?;

        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create")
        .or_else(|_| Err(anyhow::anyhow!("The `_plugin_create` symbol wasn't found.")))?;
        let boxed_raw = constructor();

        let plugin = Box::from_raw(boxed_raw);
        log::info!("Loaded plugin: {}", plugin.name());
        self.plugins.push(plugin);


        Ok(())
    }

    pub fn unload(&mut self) {
        log::debug!("Unloading plugins");

        for plugin in self.plugins.drain(..) {
            log::trace!("Firing on_plugin_unload for {:?}", plugin.name());
            plugin.on_unload();
        }

        for lib in self.loaded_libraries.drain(..) {
            drop(lib);
        }
    }
}
impl Drop for PluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() || !self.loaded_libraries.is_empty() {
            self.unload();
        }
    }
}