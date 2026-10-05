//! The OSD exception classes declared by DEFINE_STANDARD_EXCEPTION, as typed
//! `Failure` constructors. Each keeps the native class name as its exception type.
use crate::standard::Failure;

macro_rules! osd_exceptions {
    ($($(#[$meta:meta])* $function:ident => $native:literal : $parent:literal;)+) => {
        $(
            $(#[$meta])*
            #[doc = concat!("Raises `", $native, "`; `osd::", stringify!($function), "(\"message\")`.")]
            pub fn $function(message: &str) -> Failure {
                Failure::with_type($native, message, "")
            }
        )+
        /// Every OSD exception type name, in the order declared above.
        pub const OSD_EXCEPTION_TYPES: &[&str] = &[$($native),+];
        /// The direct native base class of every macro-generated OSD exception constructor.
        pub(crate) const OSD_EXCEPTION_HIERARCHY: &[(&str, &str)] = &[$(($native, $parent)),+];
    };
}

osd_exceptions! {
    /// Base of the structured-exception (Windows SEH) translations.
    exception => "OSD_Exception" : "Standard_Failure";
    /// Windows EXCEPTION_ACCESS_VIOLATION.
    exception_access_violation => "OSD_Exception_ACCESS_VIOLATION" : "OSD_Exception";
    /// Windows EXCEPTION_ARRAY_BOUNDS_EXCEEDED.
    exception_array_bounds_exceeded => "OSD_Exception_ARRAY_BOUNDS_EXCEEDED" : "OSD_Exception";
    /// Windows console CTRL_BREAK or CTRL_C (see `OSD::ControlBreak`).
    exception_ctrl_break => "OSD_Exception_CTRL_BREAK" : "OSD_Exception";
    /// Windows EXCEPTION_ILLEGAL_INSTRUCTION.
    exception_illegal_instruction => "OSD_Exception_ILLEGAL_INSTRUCTION" : "OSD_Exception";
    /// Windows EXCEPTION_IN_PAGE_ERROR.
    exception_in_page_error => "OSD_Exception_IN_PAGE_ERROR" : "OSD_Exception";
    /// Windows EXCEPTION_INT_OVERFLOW.
    exception_int_overflow => "OSD_Exception_INT_OVERFLOW" : "OSD_Exception";
    /// Windows EXCEPTION_INVALID_DISPOSITION.
    exception_invalid_disposition => "OSD_Exception_INVALID_DISPOSITION" : "OSD_Exception";
    /// Windows EXCEPTION_NONCONTINUABLE_EXCEPTION.
    exception_noncontinuable_exception => "OSD_Exception_NONCONTINUABLE_EXCEPTION" : "OSD_Exception";
    /// Windows EXCEPTION_PRIV_INSTRUCTION.
    exception_priv_instruction => "OSD_Exception_PRIV_INSTRUCTION" : "OSD_Exception";
    /// Windows EXCEPTION_STACK_OVERFLOW.
    exception_stack_overflow => "OSD_Exception_STACK_OVERFLOW" : "OSD_Exception";
    /// Windows STATUS_NO_MEMORY.
    exception_status_no_memory => "OSD_Exception_STATUS_NO_MEMORY" : "OSD_Exception";
    /// Base of the Unix signal translations.
    signal => "OSD_Signal" : "Standard_Failure";
    /// Unix SIGBUS.
    sigbus => "OSD_SIGBUS" : "OSD_Signal";
    /// Unix SIGHUP.
    sighup => "OSD_SIGHUP" : "OSD_Signal";
    /// Unix SIGILL.
    sigill => "OSD_SIGILL" : "OSD_Signal";
    /// Unix SIGINT.
    sigint => "OSD_SIGINT" : "OSD_Signal";
    /// Unix SIGKILL.
    sigkill => "OSD_SIGKILL" : "OSD_Signal";
    /// Unix SIGQUIT.
    sigquit => "OSD_SIGQUIT" : "OSD_Signal";
    /// Unix SIGSEGV.
    sigsegv => "OSD_SIGSEGV" : "OSD_Signal";
    /// Unix SIGSYS.
    sigsys => "OSD_SIGSYS" : "OSD_Signal";
}

/// Parent of the non-macro OSD_OSDError constructor; `exception_parent(OSD_OSD_ERROR_PARENT.0)`.
pub(crate) const OSD_OSD_ERROR_PARENT: (&str, &str) = ("OSD_OSDError", "Standard_Failure");
