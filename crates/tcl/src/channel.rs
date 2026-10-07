//! Channels: the standard channels, file channels, I/O and options.
//!
//! Tcl owns every channel and a script may close one at any time (`close stdout`), so a
//! [`Channel`] does not keep it alive: it registers a close handler and refuses I/O once
//! Tcl reports the close. (Taking a counted reference instead would change behavior:
//! Tcl closes a standard channel when its count drops back to one.) A channel freshly
//! made from a file descriptor has no owner yet; it is a [`NewChannel`] until it is
//! registered, installed as a standard channel, or closed.
use crate::dstring::c_length;
use crate::text::to_c_string;
use crate::{DString, InterpHandle, TclError, sys};
use std::cell::Cell;
use std::ffi::{c_char, c_int};
use std::ptr::NonNull;
use std::rc::Rc;

/// Which standard channel `Tcl_GetStdChannel` selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdChannel {
    /// `TCL_STDIN`.
    Input,
    /// `TCL_STDOUT`.
    Output,
    /// `TCL_STDERR`.
    Error,
}

impl StdChannel {
    fn code(self) -> c_int {
        match self {
            Self::Input => sys::TCL_STDIN,
            Self::Output => sys::TCL_STDOUT,
            Self::Error => sys::TCL_STDERR,
        }
    }
}

/// A reference to a channel that Tcl owns; I/O fails once Tcl has closed the channel.
///
/// ```
/// use rustycad_tcl::{Channel, StdChannel};
/// let output = Channel::standard(StdChannel::Output).expect("stdout exists");
/// assert_eq!(output.write(b"").unwrap(), 0);
/// ```
pub struct Channel {
    raw: NonNull<sys::Tcl_Channel_>,
    open: Rc<Cell<bool>>,
}

impl Channel {
    /// Starts watching `raw` for its close.
    ///
    /// # Safety
    /// `raw` must be a live channel of the current thread.
    unsafe fn watch(raw: NonNull<sys::Tcl_Channel_>) -> Self {
        let open = Rc::new(Cell::new(true));
        let client_data = Rc::into_raw(Rc::clone(&open)).cast_mut().cast();
        // SAFETY: live channel; Tcl keeps one `Rc` reference until the close callback
        // or `Drop` releases it.
        unsafe { sys::Tcl_CreateCloseHandler(raw.as_ptr(), Some(close_trampoline), client_data) };
        Self { raw, open }
    }

    /// `Tcl_GetStdChannel`; `None` when the process has no such stream.
    pub fn standard(kind: StdChannel) -> Option<Self> {
        crate::initialize_library();
        // SAFETY: plain query of the thread's standard channel table.
        let raw = NonNull::new(unsafe { sys::Tcl_GetStdChannel(kind.code()) })?;
        // SAFETY: Tcl returned a live channel of this thread.
        Some(unsafe { Self::watch(raw) })
    }

    /// The live raw handle, or an error naming `function` once Tcl closed the channel.
    pub(crate) fn live(&self, function: &'static str) -> Result<sys::Tcl_Channel, TclError> {
        if self.open.get() {
            return Ok(self.raw.as_ptr());
        }
        Err(TclError::Failed {
            function,
            argument: String::new(),
            message: "the channel was closed".into(),
        })
    }

    /// `Tcl_RegisterChannel(interp, ...)`: makes the channel visible to scripts of `interp`.
    pub fn register(&self, interp: &InterpHandle) -> Result<(), TclError> {
        let raw = self.live("Tcl_RegisterChannel")?;
        // SAFETY: live interpreter and channel.
        unsafe { sys::Tcl_RegisterChannel(interp.as_ptr(), raw) };
        Ok(())
    }

    /// `Tcl_Write`: returns the byte count Tcl accepted.
    pub fn write(&self, bytes: &[u8]) -> Result<usize, TclError> {
        let (raw, length) = (self.live("Tcl_Write")?, c_length(bytes.len())?);
        // SAFETY: live channel; `bytes` holds `length` readable bytes.
        let written = unsafe { sys::Tcl_Write(raw, bytes.as_ptr().cast::<c_char>(), length) };
        usize::try_from(written).map_err(|_| os_failure("Tcl_Write", bytes))
    }

    /// `Tcl_Flush`.
    pub fn flush(&self) -> Result<(), TclError> {
        // SAFETY: live channel.
        let code = unsafe { sys::Tcl_Flush(self.live("Tcl_Flush")?) };
        if code == sys::TCL_OK {
            Ok(())
        } else {
            Err(os_failure("Tcl_Flush", b""))
        }
    }

    /// `Tcl_Gets`: appends the next line (without its end) to `line`; returns the number
    /// of bytes read, or `None` at end of file, when no full line is available, or when
    /// the channel was closed.
    pub fn gets(&self, line: &mut DString) -> Option<usize> {
        let raw = self.live("Tcl_Gets").ok()?;
        // SAFETY: live channel and initialized string.
        usize::try_from(unsafe { sys::Tcl_Gets(raw, line.as_mut_ptr()) }).ok()
    }

    /// `Tcl_SetChannelOption(NULL, ...)`, e.g. `("-buffering", "line")`.
    pub fn set_option(&self, name: &str, value: &str) -> Result<(), TclError> {
        set_channel_option(self.live("Tcl_SetChannelOption")?, name, value)
    }
}

impl Clone for Channel {
    fn clone(&self) -> Self {
        // A closed channel gets a handle that reports the close too: no close handler
        // may be registered on freed memory.
        if self.open.get() {
            // SAFETY: the channel is live while `open` is set.
            unsafe { Self::watch(self.raw) }
        } else {
            Self {
                raw: self.raw,
                open: Rc::clone(&self.open),
            }
        }
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        if !self.open.get() {
            // Tcl closed the channel and already released its reference.
            return;
        }
        let client_data = Rc::as_ptr(&self.open).cast_mut().cast();
        // SAFETY: the channel is open, so the close handler is registered; removing it
        // means Tcl will not call it, and its reference is released here once.
        unsafe {
            sys::Tcl_DeleteCloseHandler(self.raw.as_ptr(), Some(close_trampoline), client_data);
            drop(Rc::from_raw(Rc::as_ptr(&self.open)));
        }
    }
}

/// `Tcl_CloseProc` of every [`Channel`]: marks it closed and releases Tcl's reference.
///
/// # Safety
/// Called once by Tcl with the client data of `watch`.
unsafe extern "C" fn close_trampoline(client_data: sys::ClientData) {
    // SAFETY: Tcl's one reference, released here.
    let open = unsafe { Rc::from_raw(client_data.cast_const().cast::<Cell<bool>>()) };
    open.set(false);
}

/// `Tcl_SetChannelOption` with a NULL interpreter (no message is left anywhere).
fn set_channel_option(raw: sys::Tcl_Channel, name: &str, value: &str) -> Result<(), TclError> {
    let (c_name, c_value) = (to_c_string(name)?, to_c_string(value)?);
    // SAFETY: live channel (checked by the callers) and C strings.
    let code = unsafe {
        sys::Tcl_SetChannelOption(std::ptr::null_mut(), raw, c_name.as_ptr(), c_value.as_ptr())
    };
    if code == sys::TCL_OK {
        return Ok(());
    }
    Err(TclError::Failed {
        function: "Tcl_SetChannelOption",
        argument: format!("{name} {value}"),
        message: "Tcl rejected the option or its value".into(),
    })
}

/// A failure carrying the OS error that Tcl left in `errno`.
fn os_failure(function: &'static str, argument: &[u8]) -> TclError {
    TclError::Failed {
        function,
        argument: String::from_utf8_lossy(argument).into_owned(),
        message: std::io::Error::last_os_error().to_string(),
    }
}

/// A channel created from a file descriptor that no interpreter owns yet; dropping it
/// closes it (`Tcl_Close(NULL, ...)`, legal because nothing references it).
pub struct NewChannel {
    raw: Option<NonNull<sys::Tcl_Channel_>>,
}

impl NewChannel {
    /// `Tcl_MakeFileChannel` over descriptor `fd` (Unix), which Tcl then owns, with
    /// mode `TCL_READABLE`/`TCL_WRITABLE`; `None` when Tcl refuses it.
    #[cfg(unix)]
    pub fn from_file_descriptor(fd: std::os::fd::RawFd, mode: c_int) -> Option<Self> {
        crate::initialize_library();
        // On Unix the handle is the descriptor value itself, cast to a pointer.
        let handle = std::ptr::without_provenance_mut::<std::ffi::c_void>(fd as usize);
        // SAFETY: Tcl stores the descriptor; it never dereferences the handle.
        let raw = NonNull::new(unsafe { sys::Tcl_MakeFileChannel(handle, mode) })?;
        Some(Self { raw: Some(raw) })
    }

    fn take(&mut self) -> NonNull<sys::Tcl_Channel_> {
        self.raw
            .take()
            .expect("a NewChannel holds its channel until consumed")
    }

    /// `Tcl_SetChannelOption(NULL, ...)`.
    pub fn set_option(&self, name: &str, value: &str) -> Result<(), TclError> {
        let raw = self
            .raw
            .expect("a NewChannel holds its channel until consumed");
        set_channel_option(raw.as_ptr(), name, value)
    }

    /// `Tcl_RegisterChannel(interp, ...)`: hands the channel to `interp`.
    pub fn register(mut self, interp: &InterpHandle) -> Channel {
        let raw = self.take();
        // SAFETY: live channel and interpreter; the interpreter now owns the channel.
        unsafe {
            sys::Tcl_RegisterChannel(interp.as_ptr(), raw.as_ptr());
            Channel::watch(raw)
        }
    }

    /// `Tcl_SetStdChannel` then `Tcl_RegisterChannel(NULL, ...)`, as DRAW installs its
    /// own standard channels: the process-wide table now owns the channel.
    pub fn install_as_standard(mut self, kind: StdChannel) -> Channel {
        let raw = self.take();
        // SAFETY: live channel; the NULL registration is the reference the standard
        // channel table keeps.
        unsafe {
            sys::Tcl_SetStdChannel(raw.as_ptr(), kind.code());
            sys::Tcl_RegisterChannel(std::ptr::null_mut(), raw.as_ptr());
            Channel::watch(raw)
        }
    }

    /// `Tcl_Close(NULL, ...)`: closes the channel and its descriptor.
    pub fn close(mut self) -> Result<(), TclError> {
        let raw = self.take();
        // SAFETY: nothing registered the channel, so its reference count is zero.
        let code = unsafe { sys::Tcl_Close(std::ptr::null_mut(), raw.as_ptr()) };
        if code == sys::TCL_OK {
            Ok(())
        } else {
            Err(os_failure("Tcl_Close", b""))
        }
    }
}

impl Drop for NewChannel {
    fn drop(&mut self) {
        if let Some(raw) = self.raw.take() {
            // SAFETY: unregistered channel; closing it is its only release path.
            unsafe { sys::Tcl_Close(std::ptr::null_mut(), raw.as_ptr()) };
        }
    }
}
