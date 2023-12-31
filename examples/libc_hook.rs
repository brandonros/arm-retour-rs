#![feature(c_variadic)]
#![allow(non_camel_case_types)]

use arm_retour::hooks::Hook;
use ctor::ctor;
use libc::pid_t;
use minidl::Library;
use once_cell::sync::Lazy;
use std::ffi::*;
use std::sync::Mutex;

use num_derive::FromPrimitive;

#[derive(FromPrimitive, Debug, PartialEq)]
pub enum __ptrace_request {
  PTRACE_TRACEME = 0,
  PTRACE_PEEKTEXT = 1,
  PTRACE_PEEKDATA = 2,
  PTRACE_PEEKUSER = 3,
  PTRACE_POKETEXT = 4,
  PTRACE_POKEDATA = 5,
  PTRACE_POKEUSER = 6,
  PTRACE_CONT = 7,
  PTRACE_KILL = 8,
  PTRACE_SINGLESTEP = 9,
  PTRACE_GETREGS = 12,
  PTRACE_SETREGS = 13,
  PTRACE_GETFPREGS = 14,
  PTRACE_SETFPREGS = 15,
  PTRACE_ATTACH = 16,
  PTRACE_DETACH = 17,
  PTRACE_GETFPXREGS = 18,
  PTRACE_SETFPXREGS = 19,
  PTRACE_SYSCALL = 24,
  PTRACE_SETOPTIONS = 0x4200,
  PTRACE_GETEVENTMSG = 0x4201,
  PTRACE_GETSIGINFO = 0x4202,
  PTRACE_SETSIGINFO = 0x4203,
}

type ptrace_fn = unsafe extern "C" fn(request: c_uint, ...) -> c_long;

static LIBC_HANDLE: Lazy<Library> = Lazy::new(|| Library::load("/apex/com.android.runtime/lib/bionic/libc.so").unwrap());
static LIBC_PTRACE: Lazy<ptrace_fn> = Lazy::new(|| unsafe { LIBC_HANDLE.sym("ptrace\0").unwrap() });
static PTRACE_HOOK_STRUCT: Lazy<Mutex<Option<Hook>>> = Lazy::new(|| Mutex::new(None));

unsafe extern "C" fn ptrace_hook(request: c_uint, mut args: ...) -> c_long {
  // TODO: preserve stack/state?
  let parsed_request: __ptrace_request = num::FromPrimitive::from_u32(request).unwrap();
  log::info!("ptrace_hook request = {request:x} parsed_request = {parsed_request:?}");
  // TODO: wait for attach and do special stuff?
  let pid: pid_t = unsafe { args.arg() }; // TODO: should this be a pointer or not?
  let addr: *mut c_void = unsafe { args.arg() };
  let data: *mut c_void = unsafe { args.arg() };
  log::info!("ptrace_hook request = {request:x} pid = {pid} addr = {addr:p} data = {data:p}");
  // Restore the original bytes.
  PTRACE_HOOK_STRUCT.lock().unwrap().as_ref().unwrap().disable();
  // Call original.
  let ret_val = LIBC_PTRACE(request, pid, addr, data);
  // Patch again to point to your hook.
  PTRACE_HOOK_STRUCT.lock().unwrap().as_ref().unwrap().enable();
  log::info!("ptrace_hook request = {request:x} parsed_request = {parsed_request:?} pid = {pid} addr = {addr:p} data = {data:p} ret_val = {ret_val}");
  // TODO: restore stack/registers/state?
  return ret_val;
}

fn patch_libc_ptrace() {
  let ori_fn_address = *LIBC_PTRACE as *const c_void;
  let hook_fn_address = ptrace_hook as *const c_void;
  let ori_fn_bytes_length = 10; // TODO: do not need this dynamically?
  let hook = Hook::new(ori_fn_address, hook_fn_address, ori_fn_bytes_length);
  hook.enable();
  PTRACE_HOOK_STRUCT.lock().unwrap().replace(hook);
}

#[ctor]
fn ctor() {
  patch_libc_ptrace();
}
