extern crate wasmi;
use wasmi::{RuntimeValue};
use sp_std::{vec};
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
