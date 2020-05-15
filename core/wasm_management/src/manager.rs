extern crate wasmi;
use wasmi::{Module, ModuleInstance, ModuleRef, ImportsBuilder, RuntimeValue};
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

    pub fn call_module(& self,module_id:&ModuleId<Helper::Hash>,args:&[RuntimeValue]) -> Result<Option<RuntimeValue>,ManagerError> {
        let module_ref = self.helper.get_ref_of_id(module_id).unwrap();
        module_ref.invoke_export(Helper::ENTRY_FUNC,args,&mut Helper::HOST_OBJECT).map_err(|e| ManagerError::InvokeError{error:e})
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
}
