//! `OSD_Error`: the last system error recorded by an OSD object, and its report.
//!
//! The two native builds differ, and both are kept: on Unix `Perror` builds the
//! message from `errno` and the recording class, stores the portable code and raises
//! `OSD_OSDError`; `Error()` stays 0 until `Perror` ran. On Windows `SetValue` maps the
//! Win32 code at once and `Perror` only prints to standard error.
use super::WhoAmI;
use crate::{standard::Failure, tcollection::AsciiString};

pub mod error_list;
#[cfg(unix)]
mod unix_messages;
mod windows_codes;
#[cfg(unix)]
pub use unix_messages::unix_perror_report;
pub use windows_codes::{windows_error_code, windows_perror_header};

/// `OSD_Error`; `let mut error = OsdError::new(); error.set_value(2, WhoAmI::File as i32, "Open")`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OsdError {
    message: AsciiString,
    errno: i32,
    // The raw OSD_WhoAmI: native code casts any integer, and unknown values take the
    // `default:` branches of Perror.
    code: i32,
    ext_code: i32,
}
impl Default for OsdError {
    fn default() -> Self {
        Self::new()
    }
}
impl OsdError {
    /// Starts without an error; `OsdError::new().failed() == false`.
    pub fn new() -> Self {
        Self {
            message: AsciiString::default(),
            errno: 0,
            // Windows starts from class 0; Unix leaves the code indeterminate until
            // SetValue, and Perror never reads it before then.
            code: 0,
            ext_code: 0,
        }
    }
    /// Records a native error code, the raw `OSD_WhoAmI` of the reporter and a context
    /// message; `error.set_value(libc::ENOENT, WhoAmI::File as i32, "Open")`.
    pub fn set_value(&mut self, errcode: i32, from: i32, message: impl Into<AsciiString>) {
        self.errno = errcode;
        self.code = from;
        self.message = message.into();
        if cfg!(windows) {
            self.ext_code = windows_error_code(errcode, from);
        }
    }
    /// Returns the portable `ERR_*` code; `error.error()`.
    ///
    /// On Unix this is 0 until `perror` classified the error; on Windows `set_value` sets it.
    pub const fn error(&self) -> i32 {
        self.ext_code
    }
    /// Tells whether an error is recorded; `error.failed()`.
    pub const fn failed(&self) -> bool {
        self.errno != 0
    }
    /// Forgets the recorded error but keeps the portable code; `error.reset()`.
    pub const fn reset(&mut self) {
        self.errno = 0;
    }
    /// Reports the recorded error; `error.perror()?` is `Ok` when nothing failed.
    ///
    /// Unix: raises `OSD_OSDError` with the classified message and stores the portable
    /// code. Windows: writes `Error ( <class> )<message>` to standard error and succeeds.
    pub fn perror(&mut self) -> Result<(), Failure> {
        #[cfg(windows)]
        {
            eprintln!(
                "{}{}",
                windows_perror_header(self.code),
                String::from_utf8_lossy(self.message.as_bytes())
            );
            Ok(())
        }
        #[cfg(unix)]
        {
            if self.errno == 0 {
                return Ok(());
            }
            let (text, code) = unix_perror_report(self.errno, self.code, self.message.as_bytes());
            self.ext_code = code;
            Err(osd_osd_error(&text))
        }
    }
    /// Returns the recording class, or `None` for a raw code outside `OSD_WhoAmI`; `error.who()`.
    pub fn who(&self) -> Option<WhoAmI> {
        WhoAmI::from_raw(self.code)
    }
    /// Returns the raw native error number; `error.errno()`.
    pub const fn errno(&self) -> i32 {
        self.errno
    }
    /// Borrows the context message; `error.message()`.
    pub const fn message(&self) -> &AsciiString {
        &self.message
    }
}

/// `OSD_OSDError` (a `Standard_Failure` declared by DEFINE_STANDARD_EXCEPTION);
/// `osd_osd_error("Unknown error")`.
pub fn osd_osd_error(message: &str) -> Failure {
    Failure::with_type("OSD_OSDError", message, "")
}

/// The `Failed`/`Reset`/`Perror`/`Error` members every OSD class forwards to its
/// `OSD_Error`; implement the two accessors, `host.failed()`.
pub trait OsdErrorAccess {
    /// Borrows the recorded error; `object.osd_error()`.
    fn osd_error(&self) -> &OsdError;
    /// Borrows the recorded error mutably; `object.osd_error_mut()`.
    fn osd_error_mut(&mut self) -> &mut OsdError;
    /// `Failed()`: tells whether the last operation recorded an error; `object.failed()`.
    fn failed(&self) -> bool {
        self.osd_error().failed()
    }
    /// `Reset()`: forgets the recorded error; `object.reset()`.
    fn reset(&mut self) {
        self.osd_error_mut().reset();
    }
    /// `Perror()`: reports the recorded error (see [`OsdError::perror`]); `object.perror()?`.
    fn perror(&mut self) -> Result<(), Failure> {
        self.osd_error_mut().perror()
    }
    /// `Error()`: the portable `ERR_*` code; `object.error()`.
    fn error(&self) -> i32 {
        self.osd_error().error()
    }
}
