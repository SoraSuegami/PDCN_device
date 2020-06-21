#![no_std]

#![cfg(target_arch = "wasm32")]
use core::arch::wasm32;

use panic::handle_panic;

extern crate crypto;

#[link(wasm_import_module = "host")]
extern {
    fn copy(data_ptr: *const u32, size:usize, new_ptr:*mut u32);
    fn get_hash_size() -> usize;
    fn hash(data_ptr: *const u32, size:usize, new_ptr:*mut u32);
    fn get_signature_size() -> usize;
    fn sign(seed_ptr: *const u32, seed_size:usize, new_ptr:*mut u32);
}

#[no_mangle]
pub extern fn main(module_hash_size:usize, runtime_args_size:usize, runtime_result_size:usize, memory_args_size:usize, memory_result_size:usize) -> (i32,i32) {
    let module_id_ptr = 0 as usize;
    let runtime_args_ptr = module_id_ptr + module_hash_size;
    let runtime_result_ptr = runtime_args_ptr + runtime_args_size;
    let memory_args_ptr = runtime_result_ptr + runtime_result_size;
    let memory_result_prt = memory_args_ptr + memory_args_size;

    let hash_size = unsafe { get_hash_size() };

    let copied_id_ptr = memory_result_prt + memory_result_size;
    unsafe { copy(&(module_id_ptr as u32), module_hash_size, &mut (copied_id_ptr as u32)) };
    let runtime_args_hash_ptr = copied_id_ptr + module_hash_size;
    let runtime_result_hash_ptr = write_hash(runtime_args_ptr, runtime_args_size, runtime_args_hash_ptr, hash_size);
    let memory_args_hash_ptr = write_hash(runtime_result_ptr, runtime_result_size, runtime_result_hash_ptr, hash_size);
    let memory_result_hash_ptr = write_hash(memory_args_ptr, memory_args_size, memory_args_hash_ptr, hash_size);
    let last_hash_ptr = write_hash(memory_result_prt, memory_result_size, memory_result_hash_ptr, hash_size);

    let all_hashed_ptr = write_hash(runtime_args_hash_ptr, hash_size * 4, last_hash_ptr, hash_size);
    unsafe {sign(&(last_hash_ptr as u32), hash_size, &mut (all_hashed_ptr as u32))};
    let signature_size = unsafe { get_signature_size() };
    (all_hashed_ptr as i32, signature_size as i32)
}

#[no_mangle]
extern fn write_hash(seed_ptr:usize, seed_size:usize, last_ptr:usize, hash_size:usize) -> usize {
    let new_ptr = last_ptr + hash_size;
    unsafe { hash(&(seed_ptr as u32), seed_size, &mut (new_ptr as u32)) };
    new_ptr
}

/*#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    unreachable!()
}
*/