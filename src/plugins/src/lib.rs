use std::ffi::OsStr;
use std::any::Any;
use libloading::{Library, Symbol};

pub trait Plugin: Any + Send + Sync {
    fn info(&self) -> PluginInfo;
}
pub struct PluginInfo {
    pub name: &'static str,
    pub version: &'static str,
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
    pub unsafe fn load_plugin<P: AsRef<OsStr>>(&mut self, filename: P) -> anyhow::Result<()> {
        type PluginCreate = unsafe fn() -> *mut dyn Plugin;
    
        let lib = Library::new(filename.as_ref())?;
    
        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);
    
        let lib = self.loaded_libraries.last().unwrap();
    
        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create")?;
        let boxed_raw = constructor();
    
        let plugin = Box::from_raw(boxed_raw);
        log::info!("Loaded plugin: {}", plugin.info().name);
        self.plugins.push(plugin);
    
    
        Ok(())
    }
}
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut dyn $crate::Plugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<dyn $crate::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}
