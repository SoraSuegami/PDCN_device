extern crate wasmi;
use wasmi::{RuntimeValue};
use sp_std::{vec};
//use core::hash::{BuildHasher,BuildHasherDefault};
//use no_std_compat::marker::PhantomData;
//use crate::error::{ModuleInstanceError,ModuleRunError};
//use core::str;
//use core::convert::TryInto;
use hex::{encode,FromHexError};
use hash_db::Hasher;
pub struct ModuleId<H:Hasher>(<H as Hasher>::Out);

impl<H:Hasher> ModuleId<H> {
    pub fn as_hash(&self)->&<H as Hasher>::Out {
        &self.0
    }

    pub fn as_slice(&self)->&[u8] {
        &self.0.as_ref()
    }

    pub fn as_wasm_values(&self)->vec::Vec<RuntimeValue> {
        self.as_slice()
            .into_iter()
            .map(|x| RuntimeValue::from(x.clone() as i32))
            .collect::<vec::Vec<RuntimeValue>>()
    }

    pub fn to_string_vec<'a>(&self) -> Result<vec::Vec<u8>,FromHexError> {
        let slice = self.as_slice();
        Ok(encode(slice).into_bytes())
    }
}

impl<H:Hasher> From<&[u8]> for ModuleId<H> {
    fn from(buffer:&[u8])->Self {
        Self(H::hash(buffer))
    }
}


/*pub struct Pure();
pub struct Input();
pub struct Output();

pub struct Module<I:ImportResolver,E:Externals,Type> {
    module:wasmi::Module,
    import:PhantomData<I>,
    external:PhantomData<E>,
    module_type:PhantomData<Type>
}



impl<I:ImportResolver,E:Externals,Type> Module<I,E,Type> {
    /*pub fn new<H:Hasher+Default>(buffer:&[u8]) -> Result<Self,ModuleInstanceError> {
        let module = wasmi::Module::from_buffer(buffer).map_err(|e| ModuleInstanceError::InstanceError{error: e})?;
        let mut hasher_state = H::default();
        buffer.hash(&mut hasher_state);
        let code_hash = hasher_state.finish();
        Ok(Self {
            module:module,
            import:PhantomData,
            external:PhantomData,
            module_type:PhantomData
        })
    }*/

    pub fn new<H:Hasher>(buffer:&[u8]) -> Result<(Self,ModuleId<H>),ModuleInstanceError> {
        let module = wasmi::Module::from_buffer(buffer).map_err(|e| ModuleInstanceError::InstanceError{error: e})?;
        let id = ModuleId::from(buffer);
        Ok((
            Self {
                module:module,
                import:PhantomData,
                external:PhantomData,
                module_type:PhantomData
            },
            id
        ))
    }

    pub fn instance(&self,import:&I)->Result<NotStartedModuleRef,ModuleInstanceError> {
        ModuleInstance::new(&self.module, import).map_err(|e| ModuleInstanceError::InstanceError{error: e})
    }

    pub fn run(&mut self,func_name:&str,args:&[RuntimeValue],import:&I,state:&mut E)->Result<Option<RuntimeValue>,ModuleRunError> {
        let instance = self.instance(import).map_err(|e| ModuleRunError::InstanceError{error: e})?;
        let started = instance.run_start(state).map_err(|e| ModuleRunError::TrapError{error: e})?;
        started.invoke_export(func_name, args, state).map_err(|e| ModuleRunError::RunError{error: e})
    }
}
*/