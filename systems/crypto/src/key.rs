use wasmi::{MemoryRef};
use crate::error::CryptoError;
pub trait Key {
    type Base:KeyBase;

    fn seckey_size() -> usize {
        Self::Base::SECRET_SIZE
    }

    fn pubkey_size() -> usize {
        Self::Base::PUBLIC_SIZE
    }

    fn compute_pubkey(memory:&MemoryRef,seckey_ptr:u32,new_ptr:u32) -> Result<(u32,usize),CryptoError> {
        let seckey_slice = memory.get(seckey_ptr, <Self::Base as KeyBase>::SECRET_SIZE).unwrap();
        let seckey:<Self::Base as KeyBase>::Secret = <Self::Base as KeyBase>::serialize_secret(&seckey_slice[..])?;
        let pubkey:<Self::Base as KeyBase>::Public = <Self::Base as KeyBase>::compute_pubkey(&seckey)?;
        memory.set(new_ptr,pubkey.as_ref()).unwrap();
        Ok((new_ptr,Self::pubkey_size()))
    }
}

pub trait KeyBase {
    const SECRET_SIZE:usize;
    const PUBLIC_SIZE:usize;
    type Secret:AsRef<[u8]>; //[u8;SECRET_SIZE]
    type Public:AsRef<[u8]>; //[u8;PUBLIC_SIZE]

    //fn extract_secret() -> Result<Self::Secret,CryptoError>;
    fn compute_pubkey(seckey:&Self::Secret) -> Result<Self::Public,CryptoError>;
    fn serialize_secret(data:&[u8]) -> Result<Self::Secret,CryptoError>;
    fn serialize_public(data:&[u8]) -> Result<Self::Public,CryptoError>;
}
