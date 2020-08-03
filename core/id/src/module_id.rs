use pdcn_system_crypto::Sha256Base;

pub struct ModuleId<H:Sha256Base>(<H as Sha256Base>::Output);

impl<H:Sha256Base> ModuleId<H> {
    pub fn as_hash(&self)->&<H as Sha256Base>::Output {
        &self.0
    }

    pub fn as_slice(&self)->&[u8] {
        &self.0.as_ref()
    }
}

impl<H:Sha256Base> From<&[u8]> for ModuleId<H> {
    fn from(buffer:&[u8])->Self {
        Self(H::hash(buffer))
    }
}

impl<H:Sha256Base> PartialEq for ModuleId<H> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

