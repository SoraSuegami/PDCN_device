extern crate wasmi;
use wasmi::{ModuleImportResolver, Externals};

pub trait Host:ModuleImportResolver+Externals {}
/*
struct InputInterrupt();
struct OutputInterrupt();

pub enum InterruptKind {
    InputInterrupt,
    OutputInterrupt
}

struct CoreInterrupt();
struct SpecificInterrupt();
pub enum InterruptModuleType {
    CoreInterrupt,
    SpecificInterrupt
}
*/

/*pub trait InputInterrupt:Interrupt<Type=Input> {
    /*fn call_module(&self,func_name:&str,args:&[RuntimeValue],import:&Self::Import,state:&mut Self::State) -> Result<Option<RuntimeValue>,ModuleCallError> {
        let module = self.get_module().ok_or(ModuleCallError::GetError{index:self.get_module_index()})?;
        module.run(func_name, args, import,state).map_err(|e| ModuleCallError::RunError{error:e})
    }*/

}

pub trait OutputInterrupt:Interrupt<Type=Output> {
    
}*/
