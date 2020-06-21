use std::fs;
use std::path::Path;
use std::io::{Write,IoSlice};
use std::process::Command;
use id::ModuleId;
use pdcn_system_crypto::Sha256Base;
use sha2::{Sha256 as ExSha256, Digest};

struct Sha256([u8;32]);

impl AsRef<[u8]> for Sha256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Sha256Base for Sha256 {
    const HASH_SIZE: usize = 32;
    type Output = [u8;32];
    fn hash(seed:&[u8]) -> Self::Output {
        let mut hasher = ExSha256::new();
        hasher.update(seed);
        let result = hasher.finalize();
        result.into()
    }
}

fn main(){
    const wasm_folder:&str = "../../../wasm_apps";
    const cargo_file_path:&str = "Cargo.toml";
    const define_file_path:&str = "../../wasm_management/src/define.rs";
    let wasm_path = Path::new(wasm_folder);
    if wasm_path.is_dir() {
        fs::remove_file(define_file_path).unwrap();
        let mut defines = fs::OpenOptions::new().append(true).create(true).open(define_file_path).unwrap();
        defines.write(b"use crate::define_wasm;\n");
        defines.write(b"define_wasm!(");
        let wasm_dirs = fs::read_dir(wasm_path).unwrap().map(|folder| folder.unwrap()).filter(|dir:&fs::DirEntry|{
            let dir_path = dir.path();
            dir_path.is_dir() && fs::read(dir_path.join(cargo_file_path).as_path()).is_ok()
        }).collect::<Vec<fs::DirEntry>>();
        let size = wasm_dirs.len();
        for (i,entry) in wasm_dirs.into_iter().enumerate() {
            let dir_path = entry.path();
            let dir_str = dir_path.file_name().unwrap().to_str().unwrap();
            Command::new("cd")
                    .arg(dir_str);
            Command::new("cargo build")
                    .args(&["--target","wasm32-unknown-unknown","--release"]);
            let path_buf = dir_path.join("target/wasm32-unknown-unknown/release/".to_string()+dir_str+".wasm");
            let wasm_path = path_buf.as_path();
            let bytes = fs::read(wasm_path).unwrap();
            let module_id = ModuleId::<Sha256>::from(&bytes[..]);
            defines.write(b"(");
            defines.write(format!("{:?}",module_id.as_slice()).as_bytes());
            defines.write(b",");
            defines.write(format!("{:?}",&bytes[..]).as_bytes());
            defines.write(b",");
            defines.write(bytes.len().to_string().as_bytes());
            defines.write(b")");
            if i+1!=size {
                defines.write(b",");
            }
        }
        defines.write(b");");
        //defines.write_vectored(&bufs[..]);
    }
}