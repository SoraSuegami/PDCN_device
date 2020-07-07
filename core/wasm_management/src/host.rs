extern crate wasmi;
use wasmi::{ModuleImportResolver, Externals, MemoryRef, MemoryInstance};
use wasmi::memory_units::Pages;
use core::marker::PhantomData;
use pdcn_system_crypto::Sha256Base;
use id::{ModuleId};

pub trait Host:ModuleImportResolver+Externals {
    type Hash:Sha256Base;
    fn new(id:ModuleId<Self::Hash>,mem:MemoryRef) -> Self;
    fn get_memory(&self) -> Option<MemoryRef>;
}
pub struct HostBuilder<'a, H:Host+'a> {
    id:Option<&'a [u8]>,
    memory:Option<&'a MemoryRef>,
    host:PhantomData<H>
}

impl<'a, H:Host> HostBuilder<'a, H> {
    const MEMORYSIZE:usize = 65536;

    pub fn new() -> Self {
        Self {
            id: None,
            memory: None,
            host: PhantomData
        }
    }

    pub fn module_id(self, id:&'a [u8]) -> Self {
        HostBuilder {
            id: Some(id),
            memory: self.memory,
            host:PhantomData
        }
    }

    pub fn memory(self, mem:&'a MemoryRef) -> Self {
        HostBuilder {
            id: self.id,
            memory: Some(mem),
            host:PhantomData
        }
    }

    pub fn build(self) -> H {
        let empty_slice:[u8;0] = [];
        let slice:&[u8] = match self.id {
            Some(i) => i,
            None => &empty_slice
        };
        let id = ModuleId::<<H as Host>::Hash>::from(slice);
        let empty_mem = MemoryInstance::alloc(Pages(0), Some(Pages(Self::MEMORYSIZE))).unwrap();
        let mem = match self.memory {
            Some(m) => m,
            None => &empty_mem
        };
        H::new(id,mem.clone())
    }
}
