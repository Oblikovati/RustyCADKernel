//! The Win64 DbgHelp branch of Standard_StackTrace.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. DbgHelp's process-wide symbol state
//! requires serialization even when callers capture different threads.
#![allow(unsafe_code)]

use std::{ffi::CStr, sync::Mutex};
use windows_sys::Win32::{
    Foundation::HANDLE,
    System::{
        Diagnostics::Debug::*,
        LibraryLoader::GetModuleFileNameA,
        SystemInformation::IMAGE_FILE_MACHINE_AMD64,
        Threading::{GetCurrentProcess, GetCurrentThread},
    },
};

static SYMBOL_SESSION: Mutex<()> = Mutex::new(());

struct SymbolSession(HANDLE);
impl Drop for SymbolSession {
    fn drop(&mut self) {
        // SAFETY: the process symbol session remains locked until this guard drops.
        unsafe { SymCleanup(self.0) };
    }
}

#[inline(never)]
pub(super) fn native_frames(nb_traces: i32, top_skip: i32) -> Option<Vec<String>> {
    let _lock = SYMBOL_SESSION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // SAFETY: process/thread pseudo-handles are always valid in this process.
    let process: HANDLE = unsafe { GetCurrentProcess() };
    let thread: HANDLE = unsafe { GetCurrentThread() };
    let mut context: CONTEXT = CONTEXT {
        ContextFlags: CONTEXT_FULL_AMD64,
        ..Default::default()
    };
    // SAFETY: aligned, writable native CONTEXT; null search path selects the default.
    unsafe { RtlCaptureContext(&mut context) };
    if unsafe { SymInitialize(process, std::ptr::null(), 1) } == 0 {
        return None;
    }
    let _session = SymbolSession(process);
    walk_frames(process, thread, &mut context, nb_traces, top_skip)
}

fn initial_frame(context: &CONTEXT) -> STACKFRAME64 {
    let mut frame: STACKFRAME64 = STACKFRAME64::default();
    frame.AddrPC.Offset = context.Rip;
    frame.AddrFrame.Offset = context.Rsp;
    frame.AddrStack.Offset = context.Rsp;
    frame.AddrPC.Mode = AddrModeFlat;
    frame.AddrFrame.Mode = AddrModeFlat;
    frame.AddrStack.Mode = AddrModeFlat;
    frame
}

fn walk_frames(
    process: HANDLE,
    thread: HANDLE,
    context: &mut CONTEXT,
    count: i32,
    skip: i32,
) -> Option<Vec<String>> {
    let mut frame: STACKFRAME64 = initial_frame(context);
    let mut lines: Vec<String> = Vec::new();
    for index in 0..count.checked_add(skip)? {
        if !walk_next(process, thread, &mut frame, context) || frame.AddrPC.Offset == 0 {
            break;
        }
        if index >= skip {
            lines.push(symbolized_frame(process, frame.AddrPC.Offset));
        }
    }
    (!lines.is_empty()).then_some(lines)
}

fn walk_next(
    process: HANDLE,
    thread: HANDLE,
    frame: &mut STACKFRAME64,
    context: &mut CONTEXT,
) -> bool {
    // SAFETY: initialized x64 frame/context and live handles; native callbacks use
    // the same process symbol session held under SYMBOL_SESSION for the full walk.
    unsafe {
        StackWalk64(
            IMAGE_FILE_MACHINE_AMD64 as u32,
            process,
            thread,
            frame,
            std::ptr::from_mut(context).cast(),
            None,
            Some(SymFunctionTableAccess64),
            Some(SymGetModuleBase64),
            None,
        ) != 0
    }
}

fn module_filename(process: HANDLE, address: u64) -> String {
    let mut buffer: [u8; 260] = [0; 260];
    // SAFETY: DbgHelp resolves an address from the captured stack. A zero module
    // must not reach GetModuleFileNameA, where it would mean the main executable.
    let module: u64 = unsafe { SymGetModuleBase64(process, address) };
    if module == 0 {
        return String::new();
    }
    // SAFETY: buffer is writable for its declared capacity.
    let count =
        unsafe { GetModuleFileNameA(module as _, buffer.as_mut_ptr(), buffer.len() as u32) };
    String::from_utf8_lossy(&buffer[..count as usize]).into_owned()
}

#[repr(C)]
struct SymbolBuffer {
    info: SYMBOL_INFO,
    // SYMBOL_INFO ends in a variable-length Name. This extends its aligned storage.
    name: [u8; MAX_SYM_NAME as usize],
}

fn symbolized_frame(process: HANDLE, address: u64) -> String {
    let module: String = module_filename(process, address);
    let mut symbol = SymbolBuffer {
        info: SYMBOL_INFO::default(),
        name: [0; MAX_SYM_NAME as usize],
    };
    symbol.info.SizeOfStruct = std::mem::size_of::<SYMBOL_INFO>() as u32;
    symbol.info.MaxNameLen = MAX_SYM_NAME;
    let mut displacement: u64 = 0;
    // SAFETY: the aligned symbol buffer has MaxNameLen bytes beyond SYMBOL_INFO.
    if unsafe { SymFromAddr(process, address, &mut displacement, &mut symbol.info) } == 0 {
        return format!("{module}(???)");
    }
    // SAFETY: successful SymFromAddr writes a terminated name within the buffer.
    let name = unsafe { CStr::from_ptr(symbol.info.Name.as_ptr().cast()) };
    format!("{module}({})", name.to_string_lossy())
}
