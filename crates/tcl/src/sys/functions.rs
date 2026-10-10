//! The Tcl/Tk 8.6 functions that OCCT `src/Draw` calls, hand-declared from `tclDecls.h`,
//! `tclPlatDecls.h` and `tkDecls.h`, plus three the safe wrapper needs: `Tcl_Alloc` (to
//! hand a trace error message to Tcl with `TCL_TRACE_RESULT_DYNAMIC`) and
//! `Tcl_CreateCloseHandler`/`Tcl_DeleteCloseHandler` (to learn when a script closes a
//! channel that Rust still refers to). `tests/header_check.rs`
//! compiles every row of the tables below against the installed headers.
#![allow(non_snake_case, non_camel_case_types)]

use super::signature::{callback_types, foreign_functions, variadic_functions};
use super::types::{
    ClientData, Display, Tcl_Channel, Tcl_Command, Tcl_DString, Tcl_Encoding, Tcl_FreeProcPointer,
    Tcl_Interp, Tcl_UniChar, Tk_Uid, Tk_Window, XEvent,
};
use std::ffi::{c_char, c_int, c_uint};

callback_types! {
    CALLBACK_SIGNATURES;
    /// `Tcl_CmdProc`: a string-based command procedure.
    type Tcl_CmdProc = fn(client_data: ClientData, interp: *mut Tcl_Interp, argc: c_int,
                          argv: *mut *const c_char) -> c_int;
    /// `Tcl_CmdDeleteProc`: called when a command is deleted.
    type Tcl_CmdDeleteProc = fn(client_data: ClientData) -> ();
    /// `Tcl_VarTraceProc`: a variable trace; returns NULL or an error message.
    type Tcl_VarTraceProc = fn(client_data: ClientData, interp: *mut Tcl_Interp,
                               part1: *const c_char, part2: *const c_char,
                               flags: c_int) -> *mut c_char;
    /// `Tcl_FileProc`: a file event handler (Unix).
    type Tcl_FileProc = fn(client_data: ClientData, mask: c_int) -> ();
    /// `Tcl_ChannelProc`: a channel event handler.
    type Tcl_ChannelProc = fn(client_data: ClientData, mask: c_int) -> ();
    /// `Tcl_CloseProc`: called when a channel is closed.
    type Tcl_CloseProc = fn(data: ClientData) -> ();
    /// `Tcl_ExitProc`: an exit handler.
    type Tcl_ExitProc = fn(client_data: ClientData) -> ();
    /// `Tcl_PackageInitProc`: a static package initializer.
    type Tcl_PackageInitProc = fn(interp: *mut Tcl_Interp) -> c_int;
    /// `Tcl_AppInitProc`: the application initializer given to `Tk_MainEx`.
    type Tcl_AppInitProc = fn(interp: *mut Tcl_Interp) -> c_int;
}

foreign_functions! {
    TCL_SIGNATURES;
    fn Tcl_FindExecutable(argv0: *const c_char) -> ();
    fn Tcl_CreateInterp() -> *mut Tcl_Interp;
    fn Tcl_DeleteInterp(interp: *mut Tcl_Interp) -> ();
    fn Tcl_Init(interp: *mut Tcl_Interp) -> c_int;
    fn Tcl_Exit(status: c_int) -> !;
    fn Tcl_CreateExitHandler(proc_: Option<Tcl_ExitProc>, client_data: ClientData) -> ();
    fn Tcl_StaticPackage(interp: *mut Tcl_Interp, prefix: *const c_char,
                         init_proc: Option<Tcl_PackageInitProc>,
                         safe_init_proc: Option<Tcl_PackageInitProc>) -> ();
    fn Tcl_Eval(interp: *mut Tcl_Interp, script: *const c_char) -> c_int;
    fn Tcl_EvalFile(interp: *mut Tcl_Interp, file_name: *const c_char) -> c_int;
    fn Tcl_RecordAndEval(interp: *mut Tcl_Interp, cmd: *const c_char, flags: c_int) -> c_int;
    fn Tcl_CommandComplete(cmd: *const c_char) -> c_int;
    fn Tcl_CreateCommand(interp: *mut Tcl_Interp, cmd_name: *const c_char,
                         proc_: Option<Tcl_CmdProc>, client_data: ClientData,
                         delete_proc: Option<Tcl_CmdDeleteProc>) -> Tcl_Command;
    fn Tcl_DeleteCommand(interp: *mut Tcl_Interp, cmd_name: *const c_char) -> c_int;
    fn Tcl_GetStringResult(interp: *mut Tcl_Interp) -> *const c_char;
    fn Tcl_SetResult(interp: *mut Tcl_Interp, result: *mut c_char,
                     free_proc: Tcl_FreeProcPointer) -> ();
    fn Tcl_ResetResult(interp: *mut Tcl_Interp) -> ();
    fn Tcl_AppendElement(interp: *mut Tcl_Interp, element: *const c_char) -> ();
    fn Tcl_AddErrorInfo(interp: *mut Tcl_Interp, message: *const c_char) -> ();
    fn Tcl_GetVar(interp: *mut Tcl_Interp, var_name: *const c_char, flags: c_int) -> *const c_char;
    fn Tcl_SetVar(interp: *mut Tcl_Interp, var_name: *const c_char, new_value: *const c_char,
                  flags: c_int) -> *const c_char;
    fn Tcl_SetVar2(interp: *mut Tcl_Interp, part1: *const c_char, part2: *const c_char,
                   new_value: *const c_char, flags: c_int) -> *const c_char;
    fn Tcl_UnsetVar(interp: *mut Tcl_Interp, var_name: *const c_char, flags: c_int) -> c_int;
    fn Tcl_TraceVar(interp: *mut Tcl_Interp, var_name: *const c_char, flags: c_int,
                    proc_: Option<Tcl_VarTraceProc>, client_data: ClientData) -> c_int;
    fn Tcl_UntraceVar(interp: *mut Tcl_Interp, var_name: *const c_char, flags: c_int,
                      proc_: Option<Tcl_VarTraceProc>, client_data: ClientData) -> ();
    fn Tcl_VarTraceInfo(interp: *mut Tcl_Interp, var_name: *const c_char, flags: c_int,
                        proc_: Option<Tcl_VarTraceProc>,
                        prev_client_data: ClientData) -> ClientData;
    fn Tcl_PutEnv(assignment: *const c_char) -> c_int;
    fn Tcl_Alloc(size: c_uint) -> *mut c_char;
    fn Tcl_DStringInit(ds_ptr: *mut Tcl_DString) -> ();
    fn Tcl_DStringAppend(ds_ptr: *mut Tcl_DString, bytes: *const c_char, length: c_int) -> *mut c_char;
    fn Tcl_DStringFree(ds_ptr: *mut Tcl_DString) -> ();
    fn Tcl_ExternalToUtfDString(encoding: Tcl_Encoding, src: *const c_char, src_len: c_int,
                                ds_ptr: *mut Tcl_DString) -> *mut c_char;
    fn Tcl_UtfToUniCharDString(src: *const c_char, length: c_int,
                               ds_ptr: *mut Tcl_DString) -> *mut Tcl_UniChar;
    fn Tcl_UniCharLen(uni_str: *const Tcl_UniChar) -> c_int;
    fn Tcl_GetStdChannel(type_: c_int) -> Tcl_Channel;
    fn Tcl_SetStdChannel(channel: Tcl_Channel, type_: c_int) -> ();
    fn Tcl_MakeFileChannel(handle: ClientData, mode: c_int) -> Tcl_Channel;
    fn Tcl_RegisterChannel(interp: *mut Tcl_Interp, chan: Tcl_Channel) -> ();
    /// Not called by OCCT: the wrapper learns that a channel was closed through it.
    fn Tcl_CreateCloseHandler(chan: Tcl_Channel, proc_: Option<Tcl_CloseProc>, client_data: ClientData) -> ();
    /// Not called by OCCT: removes the wrapper's close notification.
    fn Tcl_DeleteCloseHandler(chan: Tcl_Channel, proc_: Option<Tcl_CloseProc>, client_data: ClientData) -> ();
    fn Tcl_SetChannelOption(interp: *mut Tcl_Interp, chan: Tcl_Channel,
                            option_name: *const c_char, new_value: *const c_char) -> c_int;
    fn Tcl_Write(chan: Tcl_Channel, s: *const c_char, slen: c_int) -> c_int;
    fn Tcl_Flush(chan: Tcl_Channel) -> c_int;
    fn Tcl_Gets(chan: Tcl_Channel, ds_ptr: *mut Tcl_DString) -> c_int;
    fn Tcl_Close(interp: *mut Tcl_Interp, chan: Tcl_Channel) -> c_int;
    fn Tcl_CreateChannelHandler(chan: Tcl_Channel, mask: c_int, proc_: Option<Tcl_ChannelProc>,
                                client_data: ClientData) -> ();
    fn Tcl_DeleteChannelHandler(chan: Tcl_Channel, proc_: Option<Tcl_ChannelProc>,
                                client_data: ClientData) -> ();
    fn Tcl_DoOneEvent(flags: c_int) -> c_int;

}

#[cfg(unix)]
foreign_functions! {
    TCL_UNIX_SIGNATURES;
    /// Unix only (`tclPlatDecls`/`tclDecls` slot reserved on Windows); also `Tk_CreateFileHandler`.
    fn Tcl_CreateFileHandler(fd: c_int, mask: c_int, proc_: Option<Tcl_FileProc>,
                             client_data: ClientData) -> ();
    /// Unix only, as `Tcl_CreateFileHandler`.
    fn Tcl_DeleteFileHandler(fd: c_int) -> ();
}

variadic_functions! {
    TCL_VARIADIC_SIGNATURES;
    /// The argument list ends with a NULL `const char *`.
    fn Tcl_AppendResult(interp: *mut Tcl_Interp; ...) -> ();
    /// Prints a printf-style message and aborts the process.
    fn Tcl_Panic(format: *const c_char; ...) -> !;
}

foreign_functions! {
    TK_SIGNATURES;
    fn Tk_Init(interp: *mut Tcl_Interp) -> c_int;
    fn Tk_MainWindow(interp: *mut Tcl_Interp) -> Tk_Window;
    fn Tk_SetAppName(tkwin: Tk_Window, name: *const c_char) -> *const c_char;
    fn Tk_MainLoop() -> ();
    /// `Tk_Main(argc, argv, proc)` is a macro over this with `Tcl_CreateInterp()`.
    fn Tk_MainEx(argc: c_int, argv: *mut *mut c_char, app_init_proc: Option<Tcl_AppInitProc>,
                 interp: *mut Tcl_Interp) -> ();
    fn Tk_GetUid(str_: *const c_char) -> Tk_Uid;
    fn Tk_GeometryRequest(tkwin: Tk_Window, req_width: c_int, req_height: c_int) -> ();
    fn Tk_HandleEvent(event_ptr: *mut XEvent) -> ();
    fn Tk_GetNumMainWindows() -> c_int;
}

/// `Tk_Display(tkwin)`, a macro reading `Tk_FakeWin::display`.
///
/// # Safety
/// `tkwin` must be a live window record owned by Tk.
pub unsafe fn Tk_Display(tkwin: Tk_Window) -> *mut Display {
    // SAFETY: the caller passes a live Tk window; Tk_FakeWin is its public prefix and
    // `tests/header_check.rs` checks the field offset against tk.h.
    unsafe { (*tkwin.cast::<super::types::Tk_FakeWinHead>()).display }
}

/// `Tk_Name(tkwin)`, a macro reading `Tk_FakeWin::nameUid`.
///
/// # Safety
/// `tkwin` must be a live window record owned by Tk.
pub unsafe fn Tk_Name(tkwin: Tk_Window) -> Tk_Uid {
    // SAFETY: as `Tk_Display`; the offset of `nameUid` is checked against tk.h.
    unsafe { (*tkwin.cast::<super::types::Tk_FakeWinHead>()).nameUid }
}
