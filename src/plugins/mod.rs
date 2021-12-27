use std::{any::Any, ffi::OsStr};

pub use libloading::{Library, Symbol};
pub use vtable as vtable;
use vtable::{vtable as vtable_macro, VRef, VBox};
pub use hecs;
use crate::{ecs::entities::player::{Player, Chatbox}, server::Client, item::item::ItemRegistry, game::Game, commands::CommandSystem};

#[vtable_macro]
#[repr(C)]
pub struct PluginVTable {
    name: fn(VRef<PluginVTable>) -> &'static str,
    on_load: fn(VRef<PluginVTable>, &mut Game),
    on_unload: fn(VRef<PluginVTable>),
    register_items: fn(VRef<PluginVTable>, &mut ItemRegistry),
    register_commands: fn(VRef<PluginVTable>, &mut CommandSystem),
    drop: fn(VRefMut<PluginVTable>),
}
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        use $crate::game::Game;
        use $crate::item::item::ItemRegistry;
        use $crate::commands::CommandSystem;
        use $crate::plugins::vtable::*;
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> VBox<exo_beta_server::plugins::PluginVTable> {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;
            PluginVTable_static!(static PL_VT for $plugin_type);
            VBox::<PluginVTable>::new(constructor())
        }
    };
}

pub struct PluginManager {
    plugins: Vec<VBox<PluginVTable>>,
    loaded_libraries: Vec<Library>,
}

impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
        }
    }
    pub fn register_commands(&mut self, registry: &mut CommandSystem) {
        for plugin in &mut self.plugins {
            plugin.register_commands(registry);
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
        type PluginCreate = unsafe fn() -> VBox<PluginVTable>;

        let lib = Library::new(filename.as_ref()).or_else(|_| Err(anyhow::anyhow!("Unable to load the plugin")))?;

        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create")
        .or_else(|_| Err(anyhow::anyhow!("The `_plugin_create` symbol wasn't found.")))?;
        let plugin = constructor();
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