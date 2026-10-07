//! Errors of the safe wrapper and the Tcl completion code.
use crate::sys;
use std::ffi::c_int;
use std::fmt;

/// A failure of a wrapper call; every variant carries the offending value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TclError {
    /// The text has a NUL byte, which a C string cannot carry.
    InteriorNul {
        /// The rejected text.
        text: String,
    },
    /// A Tcl call failed; `message` is the interpreter result (or a description).
    Failed {
        /// The C function that failed, e.g. `Tcl_SetVar`.
        function: &'static str,
        /// The argument that was being processed.
        argument: String,
        /// The interpreter result or Tcl's error description.
        message: String,
    },
    /// The byte count does not fit the `int` length argument of the C API.
    TooLong {
        /// The rejected length in bytes.
        length: usize,
    },
}

impl fmt::Display for TclError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InteriorNul { text } => write!(
                formatter,
                "text {text:?} contains a NUL byte; the Tcl C API takes NUL-terminated strings"
            ),
            Self::Failed {
                function,
                argument,
                message,
            } => {
                write!(formatter, "{function}({argument:?}) failed: {message}")
            }
            Self::TooLong { length } => write!(
                formatter,
                "{length} bytes exceed the C `int` length limit of {}",
                c_int::MAX
            ),
        }
    }
}

impl std::error::Error for TclError {}

/// The completion code of a Tcl evaluation (`TCL_OK`, `TCL_ERROR`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Completion {
    /// `TCL_OK`.
    Ok,
    /// `TCL_ERROR`: the interpreter result holds the message.
    Error,
    /// `TCL_RETURN`.
    Return,
    /// `TCL_BREAK`.
    Break,
    /// `TCL_CONTINUE`.
    Continue,
    /// Any other code an application command returned.
    Other(c_int),
}

impl Completion {
    /// Maps a C completion code; `Completion::from_code(1) == Completion::Error`.
    pub fn from_code(code: c_int) -> Self {
        match code {
            sys::TCL_OK => Self::Ok,
            sys::TCL_ERROR => Self::Error,
            sys::TCL_RETURN => Self::Return,
            sys::TCL_BREAK => Self::Break,
            sys::TCL_CONTINUE => Self::Continue,
            other => Self::Other(other),
        }
    }

    /// The C completion code; `Completion::Ok.code() == 0`.
    pub fn code(self) -> c_int {
        match self {
            Self::Ok => sys::TCL_OK,
            Self::Error => sys::TCL_ERROR,
            Self::Return => sys::TCL_RETURN,
            Self::Break => sys::TCL_BREAK,
            Self::Continue => sys::TCL_CONTINUE,
            Self::Other(code) => code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_codes_round_trip() {
        for code in -1..7 {
            assert_eq!(Completion::from_code(code).code(), code);
        }
        assert_eq!(Completion::from_code(1), Completion::Error);
    }

    #[test]
    fn messages_name_the_value() {
        let error = TclError::Failed {
            function: "Tcl_SetVar",
            argument: "x".into(),
            message: "boom".into(),
        };
        assert_eq!(error.to_string(), "Tcl_SetVar(\"x\") failed: boom");
        assert!(
            TclError::TooLong { length: 5 }
                .to_string()
                .starts_with("5 bytes")
        );
    }
}
