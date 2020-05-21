extern crate wasmi;
use wasmi::{ModuleImportResolver, Externals, MemoryRef};

pub trait Host:ModuleImportResolver+Externals {
    fn get_memory(&self) -> Option<MemoryRef>;
}
