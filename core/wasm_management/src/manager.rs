extern crate wasmi;
use wasmi::{Module, ModuleInstance, ModuleRef, ModuleImportResolver, ImportsBuilder, ImportResolver, RuntimeValue};
use parity_wasm::elements::{Module as RawModule, ImportEntry};
//use parity_scale_codec::{Encode, Decode};
//use wasm3::{WasmArg, WasmArgs as TupleArgs, WasmType,Environment,Runtime,Module};
use sp_std::{vec};

use crate::module_id::{ModuleId};
//use crate::interrupt::{InputInterrupt,OutputInterrupt};
use hash_db::Hasher;
use core::str::{from_utf8};
use crate::error::{ManagerError};
use crate::host::{Host as HostTrait};
use crate::entry::{EntryBuilder};

pub trait ManagementHelper:Sized+Default {
    //type Resolver:ImportResolver;
    type Hash:Hasher;
    type Host:HostTrait;
    const ENTRY_FUNC:&'static str;
    const HOST:&'static str;
    const VERIFY_MODULE:&'static str;
    //const VERIFY_MODULE_ID: ModuleId<Self::Hash>;
    fn get_import_builder<'a>(&'a self) -> ImportsBuilder<'a>;
    //fn set_import_builder(&self,resolver:ImportsBuilder) -> Self;
    fn import(self, module_id:&ModuleId<Self::Hash>, module:ModuleRef) -> Self;
    fn read_bytes_of_id(&self, module_id:&ModuleId<Self::Hash>) -> Result<Option<vec::Vec<u8>>,ManagerError>;
    fn write_bytes_of_id(&self, module_id:&ModuleId<Self::Hash>, bytes:&[u8]) -> Result<(),ManagerError>;
    fn get_host(&self) -> &Self::Host;
    fn get_mut_host(&mut self) -> &mut Self::Host;
    fn set_host(self, host:Self::Host) -> Self;
}

pub struct ModuleManager<Helper:ManagementHelper> {
    helper:Helper
}

impl<Helper:ManagementHelper> ModuleManager<Helper> {

    /*pub fn new(helper:Helper,host:Helper::Host) -> Result<Self,ManagerError> {
        let imported = helper.set_host(host);
        Ok(Self {
            helper:imported
        })
    }

    pub fn add_module(&self,bytes:&[u8]) -> Result<(),ManagerError> {
        let module = Module::from_buffer(bytes).map_err(|e| ManagerError::InstanceError{error: e})?;
        let module_id:ModuleId<Helper::Hash> = ModuleId::hash(bytes);
        self.helper.write_bytes_of_id(&module_id, bytes)
    }

    pub fn call_module(&mut self,module_id:ModuleId<Helper::Hash>,args:&[RuntimeValue]) -> Result<Option<RuntimeValue>,ManagerError> {
        let bytes = self.helper.read_bytes_of_id(&module_id).unwrap().unwrap();
        let raw_module = RawModule::from_bytes(&bytes).unwrap();

        let imported_module_ids:vec::Vec<ModuleId<Helper::Hash>> = match raw_module.import_section() {
            None => vec::Vec::new(),
            Some(imports) => {
                imports
                .entries()
                .to_vec()
                .into_iter()
                .map(|entry| ModuleId::<Helper::Hash>::from(entry.module().as_bytes()))
                .collect::<vec::Vec<ModuleId<Helper::Hash>>>()
            }
        };
        let imported_module_refs = imported_module_ids
            .iter()
            .map(|id| self.helper.read_bytes_of_id(id).unwrap().unwrap())
            .map(|bytes| RawModule::from_bytes(bytes).unwrap())
            .map(|module| self.module2ref(module).unwrap())
            .collect::<vec::Vec<ModuleRef>>();
        
        let mut builder = ImportsBuilder::new();
        for (i,id) in imported_module_ids.into_iter().numerate() {
            let module_ref = imported_module_refs.get(i).unwrap();
            let module_vec = id.to_string_vec().unwrap();
            let module_str = from_utf8(&module_vec[..]).unwrap();
            builder.push_resolver(module_str, module_ref)
        }

        let module = Module::from_parity_wasm_module(raw_module).unwrap();
        let instance = ModuleInstance::new(&module,&builder)
            .map_err(|e| ManagerError::InstanceError{error: e})?
            .run_start(self.helper.get_mut_host())
            .map_err(|e| ManagerError::RunError{trap:e})?;
        instance.invoke_export(Helper::ENTRY_FUNC,args,self.helper.get_mut_host()).map_err(|e| ManagerError::InvokeError{error:e})
    }


    fn module2ref(&self,raw_module:RawModule)->Result<ModuleRef,ManagerError> {
        let imports = self.resolve_import(&raw_module).unwrap();
        let module = Module::from_parity_wasm_module(raw_module).unwrap();
        let not_started = ModuleInstance::new(&module, &imports).map_err(|e| ManagerError::RefError{error:e})?;
        Ok(not_started.assert_no_start())
    }*/



    pub fn new(helper:Helper,host:Helper::Host) -> Result<Self,ManagerError> {
        let imported = helper.set_host(host);
        Ok(Self {
            helper:imported
        })
    }

    pub fn add_module(self,bytes:&[u8]) -> Result<Self,ManagerError> {
        let module = Module::from_buffer(bytes).map_err(|e| ManagerError::InstanceError{error: e})?;
        let module_ref = self.module2ref(&module)?;
        let module_id:ModuleId<Helper::Hash> = ModuleId::from(bytes);
        //let module_vec = id.to_string_vec().map_err(|e| ManagerError::HexConvertError{error:e})?;
        //let module_str = from_utf8(&module_vec[..]).map_err(|e| ManagerError::Utf8ConvertError{error:e})?;
        self.helper.write_bytes_of_id(&module_id, bytes);
        let imported = self.helper.import(&module_id, module_ref);
        Ok(Self {
            helper:imported        
        })
    }

    pub fn call_module(&mut self,module_id:&ModuleId<Helper::Hash>,args:&[RuntimeValue]) -> Result<Option<RuntimeValue>,ManagerError> {
        let bytes = self.helper.read_bytes_of_id(module_id).unwrap().unwrap();
        let entry_module = Module::from_buffer(bytes).map_err(|e| ManagerError::InstanceError{error: e})?;
        let instance = ModuleInstance::new(&entry_module,&self.helper.get_import_builder())
            .map_err(|e| ManagerError::InstanceError{error: e})?
            .run_start(self.helper.get_mut_host())
            .map_err(|e| ManagerError::RunError{trap:e})?;
        instance.invoke_export(Helper::ENTRY_FUNC,args,self.helper.get_mut_host()).map_err(|e| ManagerError::InvokeError{error:e})
    }

    pub fn call_module_with_verification(&mut self,module_id:ModuleId<Helper::Hash>,args:&[RuntimeValue]) -> Result<(Option<RuntimeValue>,Option<RuntimeValue>),ManagerError> {
        let module_hash = module_id.as_wasm_values();
        let result1 = self.call_module(&module_id,args)?;
        let result1_vec = match result1 {
            Some(val) => vec![val],
            None => vec::Vec::new()
        };
        let args2:vec::Vec<RuntimeValue> = [&module_hash,args,&result1_vec].concat();
        let verify_id = ModuleId::from(Helper::VERIFY_MODULE.as_bytes());
        let result2 = self.call_module(&verify_id,&args2)?;
        Ok((result1, result2))
    }

    fn module2ref(&self,module:&Module)->Result<ModuleRef,ManagerError> {
        let not_started = ModuleInstance::new(module, &self.helper.get_import_builder()).map_err(|e| ManagerError::RefError{error:e})?;
        Ok(not_started.assert_no_start())
    }
    /*
    pub fn call_module_with_verification(&mut self,module_id:ModuleId<Helper::Hash>,args:&[RuntimeValue]) -> Result<(Option<RuntimeValue>,Option<RuntimeValue>),ManagerError> {
        let module_hash = module_id.as_wasm_values();
        let result1 = self.call_module(module_id,args)?;
        let result1_vec = match result1 {
            Some(val) => vec![val],
            None => vec::Vec::new()
        };
        let args2:vec::Vec<RuntimeValue> = [&module_hash,args,&result1_vec].concat();
        let verify_id = ModuleId::from(Helper::VERIFY_MODULE.as_bytes());
        let result2 = self.call_module(verify_id,&args2)?;
        Ok((result1, result2))
    }

    fn module2ref(&self,module:&Module)->Result<ModuleRef,ManagerError> {
        let not_started = ModuleInstance::new(module, &self.helper.get_import_builder()).map_err(|e| ManagerError::RefError{error:e})?;
        Ok(not_started.assert_no_start())
    }*/
}


/*pub trait ManagerHelper:Sized+Default {
    const SIZE:u32;
    const ENTRY_FUNC:&'static str;
    const VERIFY_MODULE:&'static str;
    type Hash:Hasher;
    fn read_wasm(&self,module_id:&ModuleId<Self::Hash>) -> Result<Option<vec::Vec<u8>>,ManagerError>;
    fn is_registered(&self,module_id:&ModuleId<Self::Hash>) -> Result<bool,ManagerError>;
    fn write_wasm(&self,module_id:&ModuleId<Self::Hash>,bytes:&[u8]) -> Result<(),ManagerError>;
}

pub struct ModuleManager<Helper:ManagerHelper> {
    env:Environment,
    runtime:Runtime,
    helper:Helper
}

impl<Helper:ManagerHelper> ModuleManager<Helper> {
    pub fn new(helper:Helper) -> Result<Self,ManagerError> {
        let env = Environment::new().expect("Unable to create environment");
        let rt = env
            .create_runtime(Helper::SIZE)
            .expect("Unable to create runtime");
        Ok(Self {
            env:env,
            runtime:rt,
            helper:helper
        })
    }

    pub fn call_module<Args:WasmArg,Ret:WasmType>(&self,module_id:&ModuleId<Helper::Hash>,args:Args) -> Result<Option<Ret>,ManagerError> {
        if self.helper.is_registered(module_id).unwrap() == false {
            panic!("");
        }
        let env = &self.env;
        let rt = &self.runtime;
        let bytes = self.helper.read_wasm(module_id).unwrap().unwrap();
        let module = Module::parse(env, &bytes[..])
            .expect("Unable to parse module");
        let module = rt.load_module(module).expect("Unable to load module");
        let func = module
            .find_function::<Args, Ret>(Helper::ENTRY_FUNC)
            .expect("Unable to find function");
        Ok(Some(func.call(args).unwrap()))
        /*let module_vec = module_id.to_string_vec().map_err(|e| ManagerError::HexConvertError{error:e})?;
        let module_str = from_utf8(&module_vec[..]).map_err(|e| ManagerError::Utf8ConvertError{error:e})?;
        let entry_raw_module = EntryBuilder::new()
            .set_module(module_str,Helper::ENTRY_FUNC)
            .build();
        let entry_module = Module::from_parity_wasm_module(entry_raw_module).map_err(|e| ManagerError::InstanceError{error: e})?;
        //let host = self.helper.get_host();
        let instance = ModuleInstance::new(&entry_module,&self.helper.get_import_builder())
            .map_err(|e| ManagerError::InstanceError{error: e})?
            .run_start(self.helper.get_mut_host())
            .map_err(|e| ManagerError::RunError{trap:e})?;
        instance.invoke_export(Helper::ENTRY_FUNC,args,self.helper.get_mut_host()).map_err(|e| ManagerError::InvokeError{error:e})*/
    }

    pub fn add_module(&self,bytes:&[u8]) -> Result<(),ManagerError> {
        let env = &self.env;
        let rt = &self.runtime;
        let module = Module::parse(env, &bytes[..])
            .expect("Unable to parse module");
        let module_id:ModuleId<Helper::Hash> = ModuleId::from(bytes);
        self.helper.write_wasm(&module_id, bytes);
        Ok(())
        //let module_vec = id.to_string_vec().map_err(|e| ManagerError::HexConvertError{error:e})?;
        //let module_str = from_utf8(&module_vec[..]).map_err(|e| ManagerError::Utf8ConvertError{error:e})?;
        /*self.helper.import(module_str, module_ref);
        Ok(())*/
        //let imported = self.helper.import(module_str, module_ref);
    }

}*/

/*
pub trait ManagementHelper:Sized+Default {
    //type Resolver:ImportResolver;
    type Hash:Hasher;
    type Host:HostTrait;
    const ENTRY_FUNC:&'static str;
    const HOST:&'static str;
    const VERIFY_MODULE:&'static str;
    //const VERIFY_MODULE_ID: ModuleId<Self::Hash>;
    fn get_import_builder<'a>(&'a self) -> ImportsBuilder<'a>;
    //fn set_import_builder(&self,resolver:ImportsBuilder) -> Self;
    fn import(self, module_str:&str, module:ModuleRef) -> Self;
    fn get_host(&self) -> &Self::Host;
    fn get_mut_host(&mut self) -> &mut Self::Host;
    fn set_host(self, host:Self::Host) -> Self;
}

pub struct ModuleManager<Helper:ManagementHelper> {
    helper:Helper
}

impl<Helper:ManagementHelper> ModuleManager<Helper> {

    pub fn add_module(self,buffer:&[u8]) -> Result<Self,ManagerError> {
        let module = Module::from_buffer(buffer).map_err(|e| ManagerError::InstanceError{error: e})?;
        let module_ref = self.module2ref(&module)?;
        let id:ModuleId<Helper::Hash> = ModuleId::from(buffer);
        let module_vec = id.to_string_vec().map_err(|e| ManagerError::HexConvertError{error:e})?;
        let module_str = from_utf8(&module_vec[..]).map_err(|e| ManagerError::Utf8ConvertError{error:e})?;
        /*self.helper.import(module_str, module_ref);
        Ok(())*/
        let imported = self.helper.import(module_str, module_ref);
        Ok(Self {
            helper:imported        
        })
    }

    pub fn new(helper:Helper,host:Helper::Host) -> Result<Self,ManagerError> {
        let imported = helper.set_host(host);
        Ok(Self {
            helper:imported
        })
    }

    pub fn call_module(&mut self,module_id:ModuleId<Helper::Hash>,args:&[RuntimeValue]) -> Result<Option<RuntimeValue>,ManagerError> {
        let module_vec = module_id.to_string_vec().map_err(|e| ManagerError::HexConvertError{error:e})?;
        let module_str = from_utf8(&module_vec[..]).map_err(|e| ManagerError::Utf8ConvertError{error:e})?;
        let entry_raw_module = EntryBuilder::new()
            .set_module(module_str,Helper::ENTRY_FUNC)
            .build();
        let entry_module = Module::from_parity_wasm_module(entry_raw_module).map_err(|e| ManagerError::InstanceError{error: e})?;
        //let host = self.helper.get_host();
        let instance = ModuleInstance::new(&entry_module,&self.helper.get_import_builder())
            .map_err(|e| ManagerError::InstanceError{error: e})?
            .run_start(self.helper.get_mut_host())
            .map_err(|e| ManagerError::RunError{trap:e})?;
        instance.invoke_export(Helper::ENTRY_FUNC,args,self.helper.get_mut_host()).map_err(|e| ManagerError::InvokeError{error:e})
    }

    pub fn call_module_with_verification(&mut self,module_id:ModuleId<Helper::Hash>,args:&[RuntimeValue]) -> Result<(Option<RuntimeValue>,Option<RuntimeValue>),ManagerError> {
        let module_hash = module_id.as_wasm_values();
        let result1 = self.call_module(module_id,args)?;
        let result1_vec = match result1 {
            Some(val) => vec![val],
            None => vec::Vec::new()
        };
        let args2:vec::Vec<RuntimeValue> = [&module_hash,args,&result1_vec].concat();
        let verify_id = ModuleId::from(Helper::VERIFY_MODULE.as_bytes());
        let result2 = self.call_module(verify_id,&args2)?;
        Ok((result1, result2))
    }

    fn module2ref(&self,module:&Module)->Result<ModuleRef,ManagerError> {
        let not_started = ModuleInstance::new(module, &self.helper.get_import_builder()).map_err(|e| ManagerError::RefError{error:e})?;
        Ok(not_started.assert_no_start())
    }
}*/

/*let imports = entry_raw_module.import_section();
        let interrupts:vec::Vec<I> = match imports {
            None => vec::Vec::new(),
            Some(imports) => {
                imports.entries()
                .iter()
                .filter_map(|val| self.helper.interrupt_of_name(val.module()))
                .collect::<vec::Vec<I>>()
            }
        };*/

        
        
        /*entry_raw_module
            .import_section()
            .ok_or(ManagerError::InterruptNotFoundError)?
            .entries()
            .iter()
            .map(|val| self.helper.interrupt_of_name(val.module()))
            .collect::<&str>();
        let instance = ModuleInstance::new(&entry_module,self.helper.get_import_builder())
            .map_err(|e| ManagerError::InstanceError{error: e})?
            .run_start()*/

/*
pub trait ModuleManager:Sized {
    type H:Hasher;
    type I:Interrupt;
    const CORE:vec::Vec<u8>;
    const SPECIFIC:vec::Vec<u8>;

    fn get_import_builder(&self) -> ImportsBuilder;
    fn set_import_builder(&self,resolver:&ImportsBuilder) -> Self;
    fn get_entry_module(&self) -> RawModule;

    fn add_module(&self,buffer:&[u8]) -> Result<Self,ManagerError> {
        let module = Module::from_buffer(buffer).map_err(|e| ManagerError::InstanceError{error: e})?;
        let module_ref = self.module2ref(&module)?;
        let id:ModuleId<Self::H> = ModuleId::from(buffer);
        let module_vec = id.to_string_vec().map_err(|e| ManagerError::HexConvertError{error:e})?;
        let module_str = from_utf8(&module_vec[..]).map_err(|e| ManagerError::Utf8ConvertError{error:e})?;
        let resolver = self.get_import_builder().with_resolver(module_str, &module_ref);
        Ok(self.set_import_builder(&resolver))
    }

    fn add_interrupt(&self,interrupt:Self::I) -> Result<Self,ManagerError> {
        let name_vec = Self::name_mapping(&interrupt)?;
        let name_str = from_utf8(&name_vec[..]).map_err(|e| ManagerError::Utf8ConvertError{error:e})?;
        let resolver = self.get_import_builder().with_resolver(name_str, &interrupt);
        Ok(self.set_import_builder(&resolver))
    }

    fn module2ref(&self,module:&Module)->Result<ModuleRef,ManagerError> {
        let not_started = ModuleInstance::new(module, &self.get_import_builder()).map_err(|e| ManagerError::RefError{error:e})?;
        Ok(not_started.assert_no_start())
    }

    fn name_mapping(interrupt:&Self::I) -> Result<vec::Vec<u8>, ManagerError>;
}*/
/*
impl<H:Hasher> Externals for ModuleManager<T> {

}

impl<I:ImportResolver,E:Externals,H:Hasher> ModuleManager<I,E,H> {
    const DEFAULT_PREFIX:[u8;0] = [];
    const START_FUNC:&'static str = "main";

    pub fn call_module(&self,module_hash:ModuleId<H>,signature:&Signature) -> Result<u64,ModuleCallError> {
        let module_vec = module_hash.to_string_vec().map_err(|e| ModuleCallError::GetError{error:ModuleGetError::HexConvertError{error:e}})?;
        let module_str = from_utf8(&module_vec[..]).map_err(|e|ModuleCallError::GetError{error:ModuleGetError::Utf8ConvertError{error:e}})?;
        let func = self.import.resolve_func(module_str, Self::START_FUNC, signature).map_err(|e| ModuleCallError::GetError{error:ModuleGetError::FuncError{error:e}})?;
        let result = func.signature()
        Ok(0)
    }
}*/

/*pub struct ModuleManager<T:ImportResolver,I:InputInterrupt,O:OutputInterrupt> {
    pure_modules:vec::Vec<Module<T>>,
    input_modules:vec::Vec<InputModule<T>>,
    output_modules:vec::Vec<OutputModule<T>>,
    input_interrupts:vec::Vec<I>,
    output_interrupts:vec::Vec<O>,
    modules:vec::Vec<PureModule<T>>
}*/

/*struct InterruptManager {

}
*/