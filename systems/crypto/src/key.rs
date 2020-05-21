use wasmi::{MemoryRef, RuntimeArgs};
use wasmi::memory_units::*;
use hash_db::Hasher as HasherTrait;
use crate::error::CryptoError;
pub trait Key {
    type Base:KeyBase;

    fn get_public_size() -> usize {
        Self::Base::PUBLIC_SIZE
    }

    fn get_public(memory:&MemoryRef,new_ptr:u32) -> Result<(u32,usize),CryptoError> {
        let public_data = Self::Base::compute_public()?;
        memory.set(new_ptr,public_data.as_ref()).unwrap();
        Ok((new_ptr,Self::get_public_size()))
    }
}

pub trait KeyBase {
    const SECRET_SIZE:usize;
    const PUBLIC_SIZE:usize;
    type Secret:AsRef<[u8]>; //[u8;SECRET_SIZE]
    type Public:AsRef<[u8]>; //[u8;PUBLIC_SIZE]

    fn extract_secret() -> Result<Self::Secret,CryptoError>;
    fn compute_public() -> Result<Self::Public,CryptoError>;
}
