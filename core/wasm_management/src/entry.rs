
//extern crate wasmi;
//use wasmi::{Module, ModuleInstance, ModuleRef, ImportsBuilder, Externals, Signature, Error};
extern crate parity_wasm;
use parity_wasm::builder::ModuleBuilder;
use parity_wasm::builder::module;
use parity_wasm::elements::Module;
//use parity_scale_codec::{Encode, Decode};
//use crate::interrupt::{InputInterrupt,OutputInterrupt};
//use no_std_compat::marker::PhantomData;


pub struct EntryBuilder {
    module_builder:ModuleBuilder,
}

impl EntryBuilder {
    //const MAIN:&'static str = "main";

    pub fn new() -> Self {
        Self {
            module_builder:module()
        }
    }

    pub fn set_module(self,module_name:&str,func_name:&str) -> Self {
        let new_builder = self.module_builder
            .import()
                .path(module_name,func_name)
                .build()
            .export()
                .field(func_name)
                .build();
        Self {
            module_builder:new_builder
        }
    }

    pub fn build(self) -> Module {
        self.module_builder.build()
    }
}

/*pub trait EntryBuilder:Sized {
    type H:Hasher;


    fn import_

    fn build_unit_function(&self,params:vec::Vec<ValueType>,returns:vec::Vec<ValueType>) -> Result<Self,ManagerError>{
        build_function()
            .body()

    }
}*/