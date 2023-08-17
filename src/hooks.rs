use std::ffi::*;

use libc::{mprotect, PROT_EXEC, PROT_READ, PROT_WRITE};

use crate::assembly;

pub struct Hook {
  pub ori_fn_address: usize,
  pub hook_fn_address: usize,
  pub ori_fn_bytes: Vec<u8>,
  pub patch_fn_bytes: Vec<u8>,
}

fn patch_function_memory(target_func: *mut u8, patch_instructions: &[u8]) {
  // Change memory protection to make it writable
  let page_size = 4096; // Often 4KB, but you should retrieve this dynamically.
  let page_aligned_addr = (target_func as usize & !(page_size - 1)) as *mut c_void;
  unsafe {
    let ret_val = mprotect(page_aligned_addr, page_size, PROT_READ | PROT_WRITE | PROT_EXEC);
    if ret_val != 0 {
      log::error!("mprotect ret_val = {ret_val}");
    }
  }
  // Write the jump instructions
  unsafe {
    std::ptr::copy_nonoverlapping(patch_instructions.as_ptr(), target_func, patch_instructions.len());
  }
  // Restore memory protection (if needed)
  unsafe {
    let ret_val = mprotect(page_aligned_addr, page_size, PROT_READ | PROT_EXEC);
    if ret_val != 0 {
      log::error!("mprotect ret_val = {ret_val}");
    }
  }
}

fn backup_original_bytes(address: *const c_void, length: usize) -> Vec<u8> {
  let bytes = unsafe { std::slice::from_raw_parts(address as *const u8, length) };
  bytes.to_vec()
}

impl Hook {
  pub fn new(ori_fn_address: *const c_void, hook_fn_address: *const c_void, ori_fn_bytes_length: usize) -> Hook {
    log::info!("build_hook ori_fn_address = {ori_fn_address:p} hook_fn_address = {hook_fn_address:p}");
    // backup
    let ori_fn_bytes = backup_original_bytes(unsafe { ori_fn_address.sub(1) }, ori_fn_bytes_length); // watch out for stupid thumb -1 thing
    log::info!("build_hook ori_fn_bytes = {:02x?}", ori_fn_bytes);
    // assemble
    let register = 12; // r12 / ip
    let movw = assembly::encode_movw((hook_fn_address as usize) as u16, register);
    let movt = assembly::encode_movt(((hook_fn_address as usize) >> 16) as u16, register);
    let bx = assembly::encode_bx();
    let mut patch_fn_bytes = vec![];
    patch_fn_bytes.extend_from_slice(&movw.to_be_bytes());
    patch_fn_bytes.extend_from_slice(&movt.to_be_bytes());
    patch_fn_bytes.extend_from_slice(&bx.to_be_bytes());
    if ori_fn_bytes_length == 10 {
      // no need to pad for nops?
    } else if ori_fn_bytes_length == 12 {
      let nop = assembly::encode_nop();
      patch_fn_bytes.extend_from_slice(&nop.to_be_bytes());
    } else {
      panic!("TODO");
    }
    log::info!("build_hook patch_fn_bytes = {:02x?}", patch_fn_bytes);
    // valiadte lenghth
    assert_eq!(ori_fn_bytes.len(), patch_fn_bytes.len());
    // return
    return Hook {
      ori_fn_address: ori_fn_address as usize,
      hook_fn_address: hook_fn_address as usize,
      ori_fn_bytes: ori_fn_bytes,
      patch_fn_bytes: patch_fn_bytes,
    };
  }

  pub fn enable(&self) {
    patch_function_memory((self.ori_fn_address - 1) as *const c_void as *mut u8, &self.patch_fn_bytes);
  }

  pub fn disable(&self) {
    patch_function_memory((self.ori_fn_address - 1) as *const c_void as *mut u8, &self.ori_fn_bytes);
  }
}
