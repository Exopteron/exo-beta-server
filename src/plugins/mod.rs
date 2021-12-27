use std::{any::Any, ffi::OsStr, panic::AssertUnwindSafe};

pub use libloading::{Library, Symbol};
pub use vtable as vtable;
use vtable::{vtable as vtable_macro, VRef, VBox};
pub use hecs;
use crate::{ecs::entities::player::{Player, Chatbox}, server::Client, item::item::ItemRegistry, game::Game, commands::CommandSystem};
use crate::ecs::systems::SystemExecutor;
use crate::server::Server;


pub trait RustPlugin {
    fn name(&self) -> &'static str;
    fn on_load(&self, game: &mut Game, systems: &mut SystemExecutor<Game>) {

    }
    fn on_unload(&self) {

    }
    fn register_items(&self, registry: &mut ItemRegistry) {

    }
    fn register_commands(&self, registry: &mut CommandSystem) {

    }
}

impl<T> InnerPlugin for T
where T: RustPlugin {
    fn name(&self) ->  & 'static str {
        RustPlugin::name(self)
    }


    fn on_load(&self,_1: &mut Game,_2: &mut SystemExecutor<Game>) {
        let e = std::panic::catch_unwind(AssertUnwindSafe(move || {
            RustPlugin::on_load(self, _1, _2);
        }));
        if let Err(e) = e {
            eprintln!("Error: {:?}", e);
        }
    }


    fn on_unload(&self) {
        let e = std::panic::catch_unwind(AssertUnwindSafe(move || {
            RustPlugin::on_unload(self);
        }));
        if let Err(e) = e {
            eprintln!("Error: {:?}", e);
        }
    }


    fn register_items(&self,_1: &mut ItemRegistry) {
        let e = std::panic::catch_unwind(AssertUnwindSafe(move || {
            RustPlugin::register_items(self, _1);
        }));
        if let Err(e) = e {
            eprintln!("Error: {:?}", e);
        }
    }


    fn register_commands(&self,_1: &mut CommandSystem) {
        let e = std::panic::catch_unwind(AssertUnwindSafe(move || {
            RustPlugin::register_commands(self, _1);
        }));
        if let Err(e) = e {
            eprintln!("Error: {:?}", e);
        }
    }

}
#[vtable_macro]
#[repr(C)]
pub struct InnerPluginVTable {
    name: fn(VRef<InnerPluginVTable>) -> &'static str,
    on_load: fn(VRef<InnerPluginVTable>, &mut Game, &mut SystemExecutor<Game>),
    on_unload: fn(VRef<InnerPluginVTable>),
    register_items: fn(VRef<InnerPluginVTable>, &mut ItemRegistry),
    register_commands: fn(VRef<InnerPluginVTable>, &mut CommandSystem),
    drop: fn(VRefMut<InnerPluginVTable>),
}
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        use $crate::game::Game;
        use $crate::item::item::ItemRegistry;
        use $crate::commands::CommandSystem;
        use $crate::plugins::vtable::*;
        use $crate::ecs::systems::SystemExecutor;
        use $crate::plugins::InnerPluginVTable;
        use $crate::plugins::InnerPlugin;
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> VBox<$crate::plugins::InnerPluginVTable> {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;
            $crate::InnerPluginVTable_static!(static PL_VT for $plugin_type);
            VBox::<InnerPluginVTable>::new(constructor())
        }
    };
}

pub struct PluginManager {
    plugins: Vec<VBox<InnerPluginVTable>>,
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
    pub fn load_all(&mut self, game: &mut Game, systems: &mut SystemExecutor<Game>) {
        for plugin in &mut self.plugins {
            plugin.on_load(game, systems);
        }
    }
    pub unsafe fn load_plugin<P: AsRef<OsStr>>(&mut self, filename: P) -> anyhow::Result<()> {
        type PluginCreate = unsafe fn() -> VBox<InnerPluginVTable>;

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