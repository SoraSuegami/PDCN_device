#[macro_export]
macro_rules! define_wasm {
    ($(($id:expr,$bytes:expr,$size:expr)),*) => {

        use sp_std::{vec};
        use pdcn_system_crypto::Sha256Base;
        use id::ModuleId;

        pub fn bytes_of_id<H:Sha256Base>(_id:&ModuleId<H>) -> Option<&[u8]> {
            let id_slice = _id.as_slice();
            /*match id_slice {
                $(
                    &$id => Some($bytes),
                )*
                _ => None
            }*/
            $(
                if(id_slice == &$id[..]) {
                    return  Some(&($bytes as [u8;$size]));
                }
            )*
            else {
                return None;
            }
        }
    };
}

