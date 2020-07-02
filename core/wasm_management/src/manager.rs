extern crate wasmi;
use wasmi::{Module, ModuleInstance, ImportsBuilder, RuntimeValue, ValueType};
use wasmi::memory_units::*;
use wasmi::nan_preserving_float::{F32, F64};
use sp_std::{vec};
use id::{ModuleId};
use pdcn_system_crypto::Sha256Base;
use crate::error::{ManagerError};
use crate::host::{Host as HostTrait, HostBuilder};
use core::marker::PhantomData;

pub trait ManagementHelper:Sized {
    type Hash:Sha256Base;
    const ENTRY_FUNC:&'static str;
    const ENTRY_MEMORY:&'static str;
    const HOST_MODULE:&'static str;
    const VERIFY_MODULE:&'static str;
    fn bytes_of_id(id:&ModuleId<Self::Hash>) -> Option<&[u8]>;
}

pub struct ModuleManager<Helper:ManagementHelper> {
    helper:PhantomData<Helper>
}

impl<Helper:ManagementHelper> ModuleManager<Helper> {
    pub fn new() -> Self {
        Self {
            helper:PhantomData
        }
    }

    pub fn call_module<Host:HostTrait>(&mut self,module_id:&ModuleId<Helper::Hash>,runtime_args:&[RuntimeValue],memory_args:&[u8],storage:&[u8]) -> Result<(Option<RuntimeValue>,vec::Vec<u8>,vec::Vec<u8>),ManagerError> {
        let bytes = Helper::bytes_of_id(module_id).unwrap();
        let mut host = HostBuilder::<Host>::new().build();
        let builder = ImportsBuilder::new().with_resolver(Helper::HOST_MODULE, &host);
        let module = Module::from_buffer(bytes).map_err(|e| ManagerError::InstanceError{error: e})?;
        let module_ref = ModuleInstance::new(&module,&builder)
            .map_err(|e| ManagerError::InstanceError{error: e})?
            .run_start(&mut host)
            .map_err(|e| ManagerError::RunError{trap:e})?;
        let externals = module_ref.export_by_name(Helper::ENTRY_MEMORY).unwrap();
        let memory = externals.as_memory().unwrap();
        let mut host = HostBuilder::<Host>::new().module_id(module_id.as_slice()).memory(memory).build();
        let storage_len = storage.len();
        let memory_len = memory_args.len();
        memory.grow(Pages(storage_len+memory_len)).unwrap();
        memory.set(0, storage).unwrap();
        memory.set(storage_len as u32, memory_args);
        let results = module_ref.invoke_export(Helper::ENTRY_FUNC,runtime_args, &mut host).map_err(|e| ManagerError::InvokeError{error:e})?;
        let size = memory.current_size().0;
        let storage_vec = memory.get(0, storage_len).unwrap();
        let memory_vec = memory.get(0, size - storage_len).unwrap();
        memory.zero(0,size).unwrap();
        Ok((results, memory_vec, storage_vec))
    }

    pub fn call_module_with_attestatione<Host:HostTrait>(&mut self,module_id:&ModuleId<Helper::Hash>,runtime_args:&[RuntimeValue],memory_args:&[u8],storage1:&[u8],storage2:&[u8]) -> Result<[(Option<RuntimeValue>,vec::Vec<u8>,vec::Vec<u8>);2],ManagerError> {
        let module_hash = module_id.as_slice();
        let module_hash_length = RuntimeValue::from(module_hash.len() as i32);
        let (rersult1_values,result1_memory,result1_storage) = self.call_module::<Host>(module_id,runtime_args,memory_args,storage1)?;
        let runtime_args_length = RuntimeValue::from(runtime_args.len() as i32);
        let runtime_args_vec:vec::Vec<u8> = runtime_args.into_iter().fold(vec::Vec::<u8>::new(),|mut all,val| {
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
        let storage1_length = RuntimeValue::from(storage1.len() as i32);
        let result1_storage_length = RuntimeValue::from(result1_storage.len() as i32);
        let runtime_args2:&[RuntimeValue] = &[vec![module_hash_length, runtime_args_length, result1_value_length, memory_args_length, result1_memory_length, storage1_length, result1_storage_length]].concat()[..];
        let memory_args2 = [module_hash, &runtime_args_vec[..], &result1_value_vec[..], memory_args, &result1_memory[..], storage1, &result1_storage].concat();
        let verify_id = ModuleId::from(Helper::VERIFY_MODULE.as_bytes());
        let result2 = self.call_module::<Host>(&verify_id,runtime_args2,&memory_args2[..],storage2)?;
        Ok([(rersult1_values,result1_memory,result1_storage), result2])
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
