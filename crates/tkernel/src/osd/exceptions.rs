//! The OSD exception classes declared by DEFINE_STANDARD_EXCEPTION, as typed
//! `Failure` constructors. Each keeps the native class name as its exception type.
use crate::standard::Failure;

macro_rules! osd_exceptions {
    ($($(#[$meta:meta])* $function:ident => $native:literal;)+) => {
        $(
            $(#[$meta])*
            #[doc = concat!("Raises `", $native, "`; `osd::", stringify!($function), "(\"message\")`.")]
            pub fn $function(message: &str) -> Failure {
                Failure::with_type($native, message, "")
            }
        )+
        /// Every OSD exception type name, in the order declared above.
        pub const OSD_EXCEPTION_TYPES: &[&str] = &[$($native),+];
    };
}

osd_exceptions! {
    /// Base of the structured-exception (Windows SEH) translations.
    exception => "OSD_Exception";
    /// Windows EXCEPTION_ACCESS_VIOLATION.
    exception_access_violation => "OSD_Exception_ACCESS_VIOLATION";
    /// Windows EXCEPTION_ARRAY_BOUNDS_EXCEEDED.
    exception_array_bounds_exceeded => "OSD_Exception_ARRAY_BOUNDS_EXCEEDED";
    /// Windows console CTRL_BREAK or CTRL_C (see `OSD::ControlBreak`).
    exception_ctrl_break => "OSD_Exception_CTRL_BREAK";
    /// Windows EXCEPTION_ILLEGAL_INSTRUCTION.
    exception_illegal_instruction => "OSD_Exception_ILLEGAL_INSTRUCTION";
    /// Windows EXCEPTION_IN_PAGE_ERROR.
    exception_in_page_error => "OSD_Exception_IN_PAGE_ERROR";
    /// Windows EXCEPTION_INT_OVERFLOW.
    exception_int_overflow => "OSD_Exception_INT_OVERFLOW";
    /// Windows EXCEPTION_INVALID_DISPOSITION.
    exception_invalid_disposition => "OSD_Exception_INVALID_DISPOSITION";
    /// Windows EXCEPTION_NONCONTINUABLE_EXCEPTION.
    exception_noncontinuable_exception => "OSD_Exception_NONCONTINUABLE_EXCEPTION";
    /// Windows EXCEPTION_PRIV_INSTRUCTION.
    exception_priv_instruction => "OSD_Exception_PRIV_INSTRUCTION";
    /// Windows EXCEPTION_STACK_OVERFLOW.
    exception_stack_overflow => "OSD_Exception_STACK_OVERFLOW";
    /// Windows STATUS_NO_MEMORY.
    exception_status_no_memory => "OSD_Exception_STATUS_NO_MEMORY";
    /// Base of the Unix signal translations.
    signal => "OSD_Signal";
    /// Unix SIGBUS.
    sigbus => "OSD_SIGBUS";
    /// Unix SIGHUP.
    sighup => "OSD_SIGHUP";
    /// Unix SIGILL.
    sigill => "OSD_SIGILL";
    /// Unix SIGINT.
    sigint => "OSD_SIGINT";
    /// Unix SIGKILL.
    sigkill => "OSD_SIGKILL";
    /// Unix SIGQUIT.
    sigquit => "OSD_SIGQUIT";
    /// Unix SIGSEGV.
    sigsegv => "OSD_SIGSEGV";
    /// Unix SIGSYS.
    sigsys => "OSD_SIGSYS";
}
