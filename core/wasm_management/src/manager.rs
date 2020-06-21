extern crate wasmi;
use wasmi::{Module, ModuleInstance, ModuleRef, ImportsBuilder, RuntimeValue, ValueType};
use wasmi::memory_units::*;
use wasmi::nan_preserving_float::{F32, F64};
use parity_wasm::elements::{Module as RawModule};
use parity_wasm::builder::{ModuleBuilder};
use sp_std::{vec};
use sp_std::str::from_utf8;
use id::{ModuleId};
use pdcn_system_crypto::Sha256Base;
use crate::error::{ManagerError};
use crate::host::{Host as HostTrait, HostBuilder};
use crate::define::bytes_of_id;
use core::marker::PhantomData;

pub trait ManagementHelper:Sized {
    type Hash:Sha256Base;
    const ENTRY_FUNC:&'static str;
    const ENTRY_MEMORY:&'static str;
    const HOST_MODULE:&'static str;
    const VERIFY_MODULE:&'static str;
    //fn import_module(self, module_id:ModuleId<Self::Hash>, module:ModuleRef) -> Self;
    //fn get_ref_of_id(&self, module_id:&ModuleId<Self::Hash>) -> Option<&ModuleRef>;
}

pub struct ModuleManager<Helper:ManagementHelper,Host:HostTrait> {
    helper:Helper,
    host:PhantomData<Host>
}

impl<Helper:ManagementHelper,Host:HostTrait> ModuleManager<Helper,Host> {
    pub fn new(helper:Helper) -> Self {
        Self {
            helper:helper,
            host:PhantomData
        }
    }

    /*pub fn add_module<'a>(mut self,bytes:&[u8]) -> Result<Self,ManagerError> {
        let raw_module:RawModule = ModuleBuilder::new()
            .with_module(RawModule::from_bytes(bytes).unwrap())
            .build();
        
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

        let mut host = HostBuilder::<'a, Host>::new().build();
        
        let mut builder = ImportsBuilder::new().with_resolver(Helper::HOST_MODULE, &host);
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
            .run_start(&mut host)
            .map_err(|e| ManagerError::RunError{trap:e})?;
        let imported = self.helper.import_module(module_id, module_ref);
        Ok(Self {
            helper:imported,
            host:self.host
        })
    }*/

    pub fn call_module<'a>(&mut self,module_id:&ModuleId<Helper::Hash>,runtime_args:&[RuntimeValue],memory_args:&[u8]) -> Result<(Option<RuntimeValue>,vec::Vec<u8>),ManagerError> {
        let bytes = bytes_of_id(module_id).unwrap();
        let mut host = HostBuilder::<'a, Host>::new().build();
        let builder = ImportsBuilder::new().with_resolver(Helper::HOST_MODULE, &host);
        let module = Module::from_buffer(bytes).map_err(|e| ManagerError::InstanceError{error: e})?;
        let module_ref = ModuleInstance::new(&module,&builder)
            .map_err(|e| ManagerError::InstanceError{error: e})?
            .run_start(&mut host)
            .map_err(|e| ManagerError::RunError{trap:e})?;
        let helper = &self.helper;
        //let module_ref = helper.get_ref_of_id(module_id).unwrap();
        let externals = module_ref.export_by_name(Helper::ENTRY_MEMORY).unwrap();
        let memory = externals.as_memory().unwrap();
        let mut host = HostBuilder::<'a, Host>::new().memory(memory).build();
        //let memory = self.host.get_memory().unwrap();//externals.as_memory().unwrap();
        memory.grow(Pages(memory_args.len())).unwrap();
        memory.set(0, memory_args).unwrap();
        let results = module_ref.invoke_export(Helper::ENTRY_FUNC,runtime_args, &mut host).map_err(|e| ManagerError::InvokeError{error:e})?;
        let size = memory.current_size().0;
        let memory_vec = memory.get(0, size).unwrap();
        memory.zero(0,size).unwrap();
        Ok((results, memory_vec))
    }

    pub fn call_module_with_verification(&mut self,module_id:ModuleId<Helper::Hash>,runtime_args:&[RuntimeValue],memory_args:&[u8]) -> Result<[(Option<RuntimeValue>,vec::Vec<u8>);2],ManagerError> {
        let module_hash = module_id.as_slice();
        let module_hash_length = RuntimeValue::from(module_hash.len() as i32);
        let (rersult1_values,result1_memory) = self.call_module(&module_id,runtime_args,memory_args)?;
        let runtime_args_length = RuntimeValue::from(runtime_args.len() as i32);
        let runtime_args_vec:vec::Vec<u8> = runtime_args.into_iter().fold(vec::Vec::<u8>::new(),|mut all,val|{
            all.append(&mut Self::runtime2vec(val));
            all
        });
        let result1_value_vec:vec::Vec<u8> = match rersult1_values {
            Some(val) => Self::runtime2vec(&val),
            None => vec::Vec::new()
        };
        let result1_value_length = RuntimeValue::from(result1_value_vec.len() as i32);
        let memory_args_length = RuntimeValue::from(memory_args.len() as i32);
        let result1_memory_length = RuntimeValue::from(result1_memory.len() as i32);
        let runtime_args2:&[RuntimeValue] = &[vec![module_hash_length, runtime_args_length, result1_value_length, memory_args_length, result1_memory_length]].concat()[..];
        let memory_args2 = [module_hash,&runtime_args_vec[..],&result1_value_vec[..],memory_args, &result1_memory[..]].concat();
        let verify_id = ModuleId::from(Helper::VERIFY_MODULE.as_bytes());
        let result2 = self.call_module(&verify_id,runtime_args2,&memory_args2[..])?;
        Ok([(rersult1_values,result1_memory), result2])
    }

    fn runtime2vec(val:&RuntimeValue) -> vec::Vec<u8> {
        match val.value_type() {
            ValueType::I32 => {
                val.try_into::<i32>().unwrap().to_be_bytes().to_vec()
            },
            ValueType::I64 => {
                val.try_into::<i64>().unwrap().to_be_bytes().to_vec()
            },
            ValueType::F32 => {
                val.try_into::<F32>().unwrap().to_float().to_be_bytes().to_vec()
            },
            ValueType::F64 => {
                val.try_into::<F64>().unwrap().to_float().to_be_bytes().to_vec()
            }
        }
    }
}
