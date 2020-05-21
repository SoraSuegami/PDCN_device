use wasmi::{MemoryRef, RuntimeArgs};
use wasmi::memory_units::*;
use hash_db::Hasher as HasherTrait;
use sp_std::convert::TryFrom;
use crate::error::CryptoError;
use crate::key::{Key as KeyTrait, KeyBase};
pub trait Signature {
    type Base:SignatureBase;

    fn get_signature_size() -> usize {
        Self::Base::SIGNATURE_SIZE
    }

    fn get_signature(memory:&MemoryRef,seed_ptr:u32,seed_size:usize,new_ptr:u32) -> Result<(u32,usize),CryptoError> {
        let seed_data = memory.get(seed_ptr, seed_size).unwrap();
        let signature = Self::Base::sign(&seed_data[..])?;
        memory.set(new_ptr,signature.as_ref()).unwrap();
        Ok((new_ptr,Self::get_signature_size()))
    }
}

pub trait SignatureBase:AsRef<[u8]> {
    const SIGNATURE_SIZE:usize;
    type Key:KeyTrait;
    type Signature:AsRef<[u8]>; //[u8;SIGNATURE_SIZE]

    fn sign(seed:&[u8]) -> Result<Self::Signature,CryptoError>;
    fn verify(&self,public:&<<Self::Key as KeyTrait>::Base as KeyBase>::Public) -> Result<bool,CryptoError>;
}
