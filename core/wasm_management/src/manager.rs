extern crate wasmi;
use wasmi::{Module, ModuleInstance, ModuleRef, ImportsBuilder, RuntimeValue};
use sp_std::{vec};
use crate::module_id::{ModuleId};
use hash_db::Hasher;
use crate::error::{ManagerError};
use crate::host::{Host as HostTrait};

pub trait ManagementHelper:Sized+Default {
    type Hash:Hasher;
    type Host:HostTrait;
    const ENTRY_FUNC:&'static str;
    const HOST:&'static str;
    const VERIFY_MODULE:&'static str;
    fn get_import_builder<'a>(&'a self) -> ImportsBuilder<'a>;
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
}
