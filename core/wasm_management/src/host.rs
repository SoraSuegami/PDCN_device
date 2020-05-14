extern crate wasmi;
use wasmi::{ModuleImportResolver, Externals};

pub trait Host:ModuleImportResolver+Externals {}
