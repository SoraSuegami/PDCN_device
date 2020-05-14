  
extern crate failure;
use failure::Fail;
use wasmi::{Error,Trap};
use core::str;
use hex::{FromHexError};
/*
#[derive(Debug,Fail)]
pub enum ModuleRunError {
    #[fail(display = "fail to get the instance of the module. wasmi error: {}",error)]
    InstanceError {
        error:ModuleInstanceError
    },

    #[fail(display = "traped. wasmi error: {}",error)]
    TrapError {
        error:Trap
    },

    #[fail(display = "fail to run the module. wasmi error: {}",error)]
    RunError {
        error:Error
    }
}

#[derive(Debug,Fail)]
pub enum ModuleGetError {
    #[fail(display = "fail to get the signature of the main func in the module.")]
    SignatureError {},

    #[fail(display = "fail to convert hex of the module id to vec<u8>. utf8 error:{}",error)]
    HexConvertError {
        error:FromHexError
    },

    #[fail(display = "fail to convert the vec<u8> of the module id to utf8. utf8 error:{}",error)]
    Utf8ConvertError {
        error:str::Utf8Error
    },

    #[fail(display = "fail to get the main function. wasmi error:{}",error)]
    FuncError {
        error:Error
    }
}*/



#[derive(Debug,Fail)]
pub enum ManagerError {
    #[fail(display = "fail to get the signature of the main func in the module.")]
    SignatureError {},

    #[fail(display = "fail to convert hex of the module id to vec<u8>. utf8 error:{}",error)]
    HexConvertError {
        error:FromHexError
    },

    #[fail(display = "fail to convert the vec<u8> of the module id to utf8. utf8 error:{}",error)]
    Utf8ConvertError {
        error:str::Utf8Error
    },

    #[fail(display = "fail to get the main function. wasmi error:{}",error)]
    FuncError {
        error:Error
    },

    #[fail(display = "fail to get the instance of the module. wasmi error:{}",error)]
    InstanceError {
        error:Error
    },

    #[fail(display = "fail to run the module. wasmi trap: {}",trap)]
    RunError {
        trap:Trap
    },

    #[fail(display = "fail to invoke the exported function. wasmi error: {}",error)]
    InvokeError {
        error:Error
    },

    #[fail(display = "fail to get the reference of the module. wasmi error:{}",error)]
    RefError {
        error:Error
    },

    #[fail(display = "the name of the interrupt is not found")]
    InterruptNameError {},

    #[fail(display = "the interrupt is not found")]
    InterruptNotFoundError {},
}
/*
#[derive(Debug,Fail)]
pub enum InterruptAddError {
    #[fail(display = "fail to get the signature of the main func in the module.")]
    SignatureError {},

    #[fail(display = "fail to convert hex of the module id to vec<u8>. utf8 error:{}",error)]
    HexConvertError {
        error:FromHexError
    },

    #[fail(display = "fail to convert the vec<u8> of the module id to utf8. utf8 error:{}",error)]
    Utf8ConvertError {
        error:str::Utf8Error
    },

    #[fail(display = "fail to get the main function. wasmi error:{}",error)]
    FuncError {
        error:Error
    },

    #[fail(display = "fail to get the instance of the module. wasmi error:{}",error)]
    InstanceError {
        error:Error
    },

    #[fail(display = "fail to get the name of the module. name error:{}",error)]
    NameError {
        error:InterruptNameError
    }
}*/
/*
#[derive(Debug,Fail)]
pub enum InterruptNameError {
    #[fail(display = "the name is not found")]
    NotFoundError {},
}*/

/*
#[derive(Debug,Fail)]
pub enum InterruptError {
    #[fail(display = "fail to build the module of the interrupt. wasmi error: {}",error)]
    BuildError {
        error:ModuleRunError
    },
}*/

