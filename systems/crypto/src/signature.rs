use wasmi::{MemoryRef};
use crate::error::CryptoError;
use crate::key::{Key as KeyTrait, KeyBase};
pub trait Signature {
    type Base:SignatureBase;

    fn size() -> usize {
        Self::Base::SIGNATURE_SIZE
    }

    fn sign(memory:&MemoryRef,seed_ptr:u32,seed_size:usize,seckey_ptr:u32,new_ptr:u32) -> Result<(u32,usize),CryptoError> {
        let seed_data = memory.get(seed_ptr, seed_size).unwrap();
        let seckey_slice = memory.get(seckey_ptr, <<<Self::Base as SignatureBase>::Key as KeyTrait >::Base as KeyBase>::SECRET_SIZE).unwrap();
        let seckey:<<<Self::Base as SignatureBase>::Key as KeyTrait >::Base as KeyBase>::Secret = <<<Self::Base as SignatureBase>::Key as KeyTrait >::Base as KeyBase>::serialize_secret(&seckey_slice[..])?;
        let signature = Self::Base::sign(&seed_data[..],&seckey)?;
        memory.set(new_ptr,signature.as_ref()).unwrap();
        Ok((new_ptr,Self::size()))
    }

    fn verify(memory:&MemoryRef,seed_ptr:u32,seed_size:usize,pubkey_ptr:u32,signature_ptr:u32) -> Result<bool,CryptoError> {
        let seed_data = memory.get(seed_ptr, seed_size).unwrap();
        let pubkey_slice = memory.get(pubkey_ptr, <<<Self::Base as SignatureBase>::Key as KeyTrait >::Base as KeyBase>::PUBLIC_SIZE).unwrap();
        let pubkey:<<<Self::Base as SignatureBase>::Key as KeyTrait >::Base as KeyBase>::Public = <<<Self::Base as SignatureBase>::Key as KeyTrait >::Base as KeyBase>::serialize_public(&pubkey_slice[..])?;
        let signature_slice = memory.get(signature_ptr, <Self::Base as SignatureBase>::SIGNATURE_SIZE).unwrap();
        let signature:<Self::Base as SignatureBase>::Signature = <Self::Base as SignatureBase>::serialize_signature(&signature_slice[..])?;
        <Self::Base as SignatureBase>::verify(&seed_data[..], &pubkey, &signature)
    }
}

pub trait SignatureBase:AsRef<[u8]> {
    const SIGNATURE_SIZE:usize;
    type Key:KeyTrait;
    type Signature:AsRef<[u8]>; //[u8;SIGNATURE_SIZE]

    fn sign(seed:&[u8],seckey:&<<Self::Key as KeyTrait>::Base as KeyBase>::Secret) -> Result<Self::Signature,CryptoError>;
    fn verify(seed:&[u8],pubkey:&<<Self::Key as KeyTrait>::Base as KeyBase>::Public, signature:&Self::Signature) -> Result<bool,CryptoError>;
    fn serialize_signature(data:&[u8]) -> Result<Self::Signature,CryptoError>;
}
