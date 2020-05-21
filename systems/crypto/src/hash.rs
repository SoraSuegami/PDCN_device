use wasmi::{MemoryRef, RuntimeArgs};
use wasmi::memory_units::*;
use hash_db::Hasher as HasherTrait;
use crate::error::CryptoError;
pub trait Hash {
    type Hasher:HasherTrait;
    const SIZE:usize;

    fn hash(memory:&MemoryRef,data_ptr:u32,size:usize,new_ptr:u32) -> Result<(u32,usize),CryptoError> {
        let data = memory.get(data_ptr,size).unwrap();
        let hashed = Self::Hasher::hash(&data[..]);
        let slice = hashed.as_ref();
        memory.set(new_ptr,slice).unwrap();
        Ok((new_ptr,Self::SIZE))
    }
}
