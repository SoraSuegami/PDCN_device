extern crate wasmi;
use wasmi::{ModuleImportResolver, Externals, MemoryRef, MemoryInstance};
use wasmi::memory_units::Pages;
use core::marker::PhantomData;


pub trait Host:ModuleImportResolver+Externals {
    fn new(memory:MemoryRef) -> Self;
    fn get_memory(&self) -> Option<MemoryRef>;
}

pub struct HostBuilder<'a, H:Host> {
    memory:Option<&'a MemoryRef>,
    host:PhantomData<H>
}

impl<'a, H:Host> HostBuilder<'a, H> {
    const MEMORYSIZE:usize = 65536;

    pub fn new() -> Self {
        Self {
            memory: None,
            host:PhantomData
        }
    }

    pub fn memory(self, mem:&'a MemoryRef) -> Self {
        HostBuilder {
            memory: Some(mem),
            host:PhantomData
        }
    }

    pub fn build(self) -> H {
        let empty_mem = MemoryInstance::alloc(Pages(0), Some(Pages(Self::MEMORYSIZE))).unwrap();
        let mem = match self.memory {
            Some(m) => m,
            None => &empty_mem
        };
        H::new(mem.clone())
    }
}


