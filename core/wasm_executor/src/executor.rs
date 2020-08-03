extern crate wasmi;
use wasmi::{Module, ModuleInstance, ImportsBuilder};
use wasmi::memory_units::*;
use heapless::{Vec, ArrayLength};
use id::{ModuleId};
use pdcn_system_crypto::Sha256Base;
use crate::error::{ManagerError};
use crate::host::{Host as HostTrait, HostBuilder};
use core::marker::PhantomData;

pub trait HelperTrait:Sized {
    type Hash:Sha256Base;
    const ID_SIZE:usize = <Self::Hash as Sha256Base>::HASH_SIZE;
    const ENTRY_FUNC:&'static str;
    const ENTRY_MEMORY:&'static str;
    const HOST_MODULE:&'static str;
    const ATTESTATION_MODULE:&'static str;
    fn bytes_of_id(id:&ModuleId<Self::Hash>) -> Option<&[u8]>;
}

pub struct Executor<Helper:HelperTrait,Host:HostTrait> {
    helper:PhantomData<Helper>,
    host:PhantomData<Host>
}

impl<Helper:HelperTrait,Host:HostTrait> Executor<Helper,Host> {
    pub fn call_module <ML,SL> (
        module_id:&ModuleId<Helper::Hash>,
        args:&[u8],
        storage:&[u8],
        caller:&ModuleId<Helper::Hash>
    ) -> Result<(Vec<u8,ML>,Vec<u8,SL>),ManagerError> 
    where
        ML: ArrayLength<u8>,
        SL: ArrayLength<u8>
    {
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
        let caller_hash = caller.as_slice();
        let storage_len = storage.len();
        let memory_len = args.len();
        let memory_offset = storage_len+Helper::ID_SIZE;
        memory.grow(Pages(memory_offset+memory_len)).unwrap();
        memory.set(0, storage).unwrap();
        memory.set(storage_len as u32, caller_hash).unwrap();
        memory.set(memory_offset as u32, args).unwrap();
        module_ref.invoke_export(Helper::ENTRY_FUNC,&[], &mut host).map_err(|e| ManagerError::InvokeError{error:e})?;
        let size = memory.current_size().0;
        let mut memory_vec = Vec::<u8,ML>::new();
        let mut storage_vec = Vec::<u8,SL>::new();
        memory.get_into(0, storage_vec.as_mut()).unwrap();
        memory.get_into(memory_offset as u32, memory_vec.as_mut()).unwrap();
        memory.zero(0,size).unwrap();
        Ok((memory_vec, storage_vec))
    }

    /*pub fn call_attestatione(
        module_id:&ModuleId<Helper::Hash>,
        input:(/*&[RuntimeValue],*/&[u8],&[u8]),
        output:(/*Option<RuntimeValue>,*/&[u8],&[u8]),
        attestation_storage:&[u8]
    ) -> Result<[(/*Option<RuntimeValue>,*/vec::Vec<u8>,vec::Vec<u8>);2],ManagerError> {
        let module_hash = module_id.as_slice();
        let module_hash_length = RuntimeValue::from(module_hash.len() as i32);
        let (/*runtime_args,*/memory_args,storage1) = input;
        let (/*rersult1_values,*/result1_memory,result1_storage) = output;
        /*let runtime_args_length = RuntimeValue::from(runtime_args.len() as i32);
        let runtime_args_vec:vec::Vec<u8> = runtime_args.into_iter().fold(vec::Vec::<u8>::new(),|mut all,val| {
            all.append(&mut Self::runtime2vec(val));
            all
        });
        let result1_value_vec:vec::Vec<u8> = match rersult1_values {
            Some(val) => Self::runtime2vec(&val),
            None => vec::Vec::new()
        };
        let result1_value_length = RuntimeValue::from(result1_value_vec.len() as i32);*/
        let memory_args_length = RuntimeValue::from(memory_args.len() as i32);
        let result1_memory_length = RuntimeValue::from(result1_memory.len() as i32);
        let storage1_length = RuntimeValue::from(storage1.len() as i32);
        let result1_storage_length = RuntimeValue::from(result1_storage.len() as i32);
        let attestation_id = ModuleId::<Helper::Hash>::from(Helper::ATTESTATION_MODULE.as_bytes());
        let attestation_hash =  attestation_id.as_slice();
        //let runtime_args2:&[RuntimeValue] = &[module_hash_length, runtime_args_length, result1_value_length, memory_args_length, result1_memory_length, storage1_length, result1_storage_length];
        let memory_args2 = [module_hash_length, runtime_args_length, result1_value_length, memory_args_length, result1_memory_length, storage1_length, result1_storage_length, module_hash, attestation_hash, &runtime_args_vec[..], &result1_value_vec[..], memory_args, result1_memory, storage1, result1_storage].concat();
        let result2 = Self::call_module(&attestation_id,runtime_args2,&memory_args2[..],attestation_storage)?;
        Ok([(rersult1_values,result1_memory.to_vec(),result1_storage.to_vec()), result2])
    }*/

    /*fn runtime2vec(val:&RuntimeValue) -> vec::Vec<u8> {
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
    }*/
}
