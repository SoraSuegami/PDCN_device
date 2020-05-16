extern crate wasmi;
use wasmi::{Module, ModuleInstance, ModuleRef, ImportsBuilder, RuntimeValue, MemoryInstance, MemoryRef};
use wasmi::memory_units::*;
use parity_wasm::elements::{Module as RawModule};
use sp_std::{vec};
use sp_std::str::from_utf8;
use crate::module_id::{ModuleId};
use hash_db::Hasher;
use crate::error::{ManagerError};
use crate::host::{Host as HostTrait};

pub trait ManagementHelper:Sized+Default {
    type Hash:Hasher;
    type Host:HostTrait;
    const ENTRY_FUNC:&'static str;
    const ENTRY_MEMORY:&'static str;
    const HOST_MODULE:&'static str;
    const VERIFY_MODULE:&'static str;
    const HOST_OBJECT:Self::Host;
    fn import(self, module_id:ModuleId<Self::Hash>, module:ModuleRef) -> Self;
    fn get_ref_of_id(&self, module_id:&ModuleId<Self::Hash>) -> Option<&ModuleRef>;
}

pub struct ModuleManager<Helper:ManagementHelper> {
    helper:Helper
}

impl<Helper:ManagementHelper> ModuleManager<Helper> {
    pub fn new(helper:Helper) -> Self {
        Self {
            helper:helper
        }
    }

    pub fn add_module(self,bytes:&[u8]) -> Result<Self,ManagerError> {
        let raw_module = RawModule::from_bytes(bytes).unwrap();
        let imported_module_ids:vec::Vec<ModuleId<Helper::Hash>> = match raw_module.import_section() {
            None => vec::Vec::new(),
            Some(imports) => {
                imports
                .entries()
                .to_vec()
                .into_iter()
                .filter(|entry| entry.module()!=Helper::HOST_MODULE)
                .map(|entry| ModuleId::<Helper::Hash>::from(entry.module().as_bytes()))
                .collect::<vec::Vec<ModuleId<Helper::Hash>>>()
            }
        };
        let imported_module_refs = imported_module_ids
            .iter()
            .map(|id| self.helper.get_ref_of_id(id).unwrap())
            .collect::<vec::Vec<&ModuleRef>>();
        
        let host_object = Helper::HOST_OBJECT;
        let mut builder = ImportsBuilder::new().with_resolver(Helper::HOST_MODULE, &host_object);
        for (i,id) in imported_module_ids.into_iter().enumerate() {
            let module_ref = imported_module_refs.get(i).unwrap().clone();
            let module_vec = id.to_string_vec().unwrap();
            let module_str = from_utf8(&module_vec[..]).unwrap();
            builder.push_resolver(module_str, module_ref)
        }
        let module = Module::from_buffer(bytes).map_err(|e| ManagerError::InstanceError{error: e})?;
        let module_id:ModuleId<Helper::Hash> = ModuleId::from(bytes);
        let module_ref = ModuleInstance::new(&module,&builder)
            .map_err(|e| ManagerError::InstanceError{error: e})?
            .run_start(&mut Helper::HOST_OBJECT)
            .map_err(|e| ManagerError::RunError{trap:e})?;
        let imported = self.helper.import(module_id, module_ref);
        Ok(Self {
            helper:imported        
        })
    }

    pub fn call_module(& self,module_id:&ModuleId<Helper::Hash>,runtime_args:&[RuntimeValue],memory_args:&[u8]) -> Result<(Option<RuntimeValue>,vec::Vec<u8>),ManagerError> {
        /*let (runtime_args, pushed_memory) = args.into_iter().fold((runtime_args,memory,0), |(all_args, all_mem, offset), arg|{
            match arg {
                Arg::RuntimeValue => {
                    all_args.push(arg);
                    (all_args, all_mem, offset)
                },

                Arg::B => {
                    let size = arg.len();
                }
            }
        });*/
        let module_ref = self.helper.get_ref_of_id(module_id).unwrap();
        let externals = module_ref.export_by_name(Helper::ENTRY_MEMORY).unwrap();
        let memory = externals.as_memory().unwrap();
        memory.set(0, memory_args).unwrap();
        let results = module_ref.invoke_export(Helper::ENTRY_FUNC,runtime_args,&mut Helper::HOST_OBJECT).map_err(|e| ManagerError::InvokeError{error:e})?;
        let size = memory.current_size().0;
        let memory_vec = memory.get(0, size).unwrap();
        Ok((results, memory_vec))
    }

    pub fn call_module_with_verification(&mut self,module_id:ModuleId<Helper::Hash>,runtime_args:&[RuntimeValue],memory_args:&[u8]) -> Result<[(Option<RuntimeValue>,vec::Vec<u8>);2],ManagerError> {
        let module_hash = module_id.as_slice();
        let (rersult1_values,result1_memory) = self.call_module(&module_id,runtime_args,memory_args)?;
        let args_length = RuntimeValue::from(runtime_args.len() as i32);
        let result1_values_vec:vec::Vec<RuntimeValue> = match rersult1_values {
            Some(val) => vec![val],
            None => vec::Vec::new()
        };
        let memory_length = RuntimeValue::from(memory_args.len() as i32);
        let args2:&[RuntimeValue] = &[vec![args_length,memory_length],runtime_args.to_vec(),result1_values_vec].concat()[..];
        let memory_args2 = [memory_args, &result1_memory[..]].concat();
        let verify_id = ModuleId::from(Helper::VERIFY_MODULE.as_bytes());
        let result2 = self.call_module(&verify_id,args2,&memory_args2[..])?;
        Ok([(rersult1_values,result1_memory), result2])
    }
}
