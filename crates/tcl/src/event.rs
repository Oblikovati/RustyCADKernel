//! The event loop: `Tcl_DoOneEvent`, channel and file handlers with Rust bodies.
//!
//! A handler guard owns the registration; dropping it deletes the handler. Tcl holds an
//! `Rc` reference to the body, and the trampoline takes its own for each call, so a body
//! that drops its own guard (as DRAW's `StdinProc` does at end of input) stays alive
//! until it returns. A panic inside a body aborts the event (it cannot unwind into C);
//! the panic message is written to the standard error stream.
use crate::{Channel, TclError, sys};
use std::ffi::c_int;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

/// `Tcl_DoOneEvent(flags)`: true when an event was processed.
///
/// ```
/// use rustycad_tcl::{do_one_event, sys};
/// // Nothing is pending in a fresh thread.
/// assert!(!do_one_event(sys::TCL_ALL_EVENTS | sys::TCL_DONT_WAIT));
/// ```
pub fn do_one_event(flags: c_int) -> bool {
    crate::initialize_library();
    // SAFETY: processes the event queue of the current thread.
    unsafe { sys::Tcl_DoOneEvent(flags) != 0 }
}

/// A handler body; receives the ready mask (`TCL_READABLE`, ...).
pub type EventBody = Box<dyn Fn(c_int)>;

struct HandlerRecord {
    body: EventBody,
}

/// Runs a handler body for one event, catching a panic at the C boundary.
///
/// # Safety
/// `client_data` must be a live `Rc<HandlerRecord>` reference held by Tcl.
unsafe fn dispatch(client_data: sys::ClientData, mask: c_int) {
    let pointer: *const HandlerRecord = client_data.cast_const().cast();
    // SAFETY: Tcl holds a reference; this call takes its own for its duration.
    let record = unsafe {
        Rc::increment_strong_count(pointer);
        Rc::from_raw(pointer)
    };
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| (record.body)(mask))) {
        let message = panic.downcast_ref::<String>().cloned().unwrap_or_default();
        eprintln!("Rust panic in a Tcl event handler: {message}");
    }
}

/// `Tcl_ChannelProc` / `Tcl_FileProc` of every Rust handler.
///
/// # Safety
/// Called by Tcl with the client data of a live registration.
unsafe extern "C" fn handler_trampoline(client_data: sys::ClientData, mask: c_int) {
    // SAFETY: forwarded contract.
    unsafe { dispatch(client_data, mask) };
}

/// A channel handler registration (`Tcl_CreateChannelHandler`); drop deletes it. When
/// a script closes the channel, Tcl discards the handler itself.
pub struct ChannelHandler {
    channel: Channel,
    record: *const HandlerRecord,
}

impl ChannelHandler {
    /// Registers `body` for the events in `mask` on `channel`.
    pub fn new(channel: &Channel, mask: c_int, body: EventBody) -> Result<Self, TclError> {
        channel.live("Tcl_CreateChannelHandler")?;
        let record = Rc::into_raw(Rc::new(HandlerRecord { body }));
        let handler = Self {
            channel: channel.clone(),
            record,
        };
        handler.set_mask(mask)?;
        Ok(handler)
    }

    /// Changes the event mask (0 suspends the handler), as a repeated
    /// `Tcl_CreateChannelHandler` with the same procedure and data does in C.
    pub fn set_mask(&self, mask: c_int) -> Result<(), TclError> {
        let raw = self.channel.live("Tcl_CreateChannelHandler")?;
        // SAFETY: live channel; the record is owned by this guard.
        unsafe {
            sys::Tcl_CreateChannelHandler(
                raw,
                mask,
                Some(handler_trampoline),
                self.record.cast_mut().cast(),
            )
        };
        Ok(())
    }
}

impl Drop for ChannelHandler {
    fn drop(&mut self) {
        if let Ok(raw) = self.channel.live("Tcl_DeleteChannelHandler") {
            // SAFETY: live channel; removes this guard's registration.
            unsafe {
                sys::Tcl_DeleteChannelHandler(
                    raw,
                    Some(handler_trampoline),
                    self.record.cast_mut().cast(),
                )
            };
        }
        // SAFETY: the guard's reference, released once; a running body holds its own.
        drop(unsafe { Rc::from_raw(self.record) });
    }
}

/// A file descriptor handler registration (`Tcl_CreateFileHandler`, Unix); drop deletes it.
#[cfg(unix)]
pub struct FileHandler {
    fd: c_int,
    record: *const HandlerRecord,
}

#[cfg(unix)]
impl FileHandler {
    /// Registers `body` for the events in `mask` on descriptor `fd`; a later registration
    /// on the same descriptor replaces this one in Tcl, as in C.
    pub fn new(fd: std::os::fd::RawFd, mask: c_int, body: EventBody) -> Self {
        crate::initialize_library();
        let record = Rc::into_raw(Rc::new(HandlerRecord { body }));
        // SAFETY: Tcl stores the descriptor and the record; the guard keeps the record
        // alive until it deletes the handler.
        unsafe {
            sys::Tcl_CreateFileHandler(fd, mask, Some(handler_trampoline), record.cast_mut().cast())
        };
        Self { fd, record }
    }
}

#[cfg(unix)]
impl Drop for FileHandler {
    fn drop(&mut self) {
        // SAFETY: removes the handler of this descriptor, then releases the reference.
        unsafe {
            sys::Tcl_DeleteFileHandler(self.fd);
            drop(Rc::from_raw(self.record));
        }
    }
}
