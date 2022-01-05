use std::sync::Arc;

use j4rs::{ClasspathEntry, Jvm, JvmBuilder};


pub struct JVMSetup;

impl JVMSetup {
    pub fn setup(mc_jar: &str) -> anyhow::Result<()> {
        let jvm = JvmBuilder::new()
            .classpath_entry(ClasspathEntry::new(mc_jar))
            .build()?;
        jvm.invoke_static("com.exopteron.RustInterface", "init", &[])?;
        Ok(())
    }
}
