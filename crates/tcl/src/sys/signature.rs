//! Declaration macros that keep each foreign signature and its checkable description in
//! one place: every `extern` item also produces a [`Signature`] row, and the header test
//! (`tests/header_check.rs`) compiles those rows against the system `tcl.h`/`tk.h`.

/// One foreign function or callback type as written in Rust, in token text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature {
    /// The C name (`Tcl_Eval`) or callback typedef name (`Tcl_CmdProc`).
    pub name: &'static str,
    /// The Rust argument types, in `stringify!` form.
    pub arguments: &'static [&'static str],
    /// The Rust return type in `stringify!` form; `"()"` for none, `"!"` for no return.
    pub returns: &'static str,
    /// True when the C function takes `...` after the listed arguments.
    pub variadic: bool,
}

/// Declares non-variadic foreign functions and their [`Signature`] table `$table`.
macro_rules! foreign_functions {
    ($table:ident; $( $(#[$attr:meta])* fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) -> $ret:ty ; )*) => {
        unsafe extern "C" {
            $( $(#[$attr])*
            #[doc = concat!("`", stringify!($name), "` of the C API.")]
            pub fn $name($($arg: $ty),*) -> $ret; )*
        }
        /// Signatures of the functions declared next to this table.
        pub const $table: &[$crate::sys::Signature] = &[
            $( $crate::sys::Signature {
                name: stringify!($name),
                arguments: &[$(stringify!($ty)),*],
                returns: stringify!($ret),
                variadic: false,
            }, )*
        ];
    };
}

/// Declares C-variadic foreign functions (`f(fixed, ...)`) and their table `$table`.
macro_rules! variadic_functions {
    ($table:ident; $( $(#[$attr:meta])* fn $name:ident ( $($arg:ident : $ty:ty),* ; ... ) -> $ret:ty ; )*) => {
        unsafe extern "C" {
            $( $(#[$attr])*
            #[doc = concat!("`", stringify!($name), "` of the C API (C-variadic).")]
            pub fn $name($($arg: $ty),* , ...) -> $ret; )*
        }
        /// Signatures of the variadic functions declared next to this table.
        pub const $table: &[$crate::sys::Signature] = &[
            $( $crate::sys::Signature {
                name: stringify!($name),
                arguments: &[$(stringify!($ty)),*],
                returns: stringify!($ret),
                variadic: true,
            }, )*
        ];
    };
}

/// Declares C callback typedefs (`typedef int (Tcl_CmdProc)(...)`) as Rust function
/// pointer types and their table `$table`.
macro_rules! callback_types {
    ($table:ident; $( $(#[$attr:meta])* type $name:ident = fn ( $($arg:ident : $ty:ty),* $(,)? ) -> $ret:ty ; )*) => {
        $( $(#[$attr])* pub type $name = unsafe extern "C" fn($($arg: $ty),*) -> $ret; )*
        /// Signatures of the callback types declared next to this table.
        pub const $table: &[$crate::sys::Signature] = &[
            $( $crate::sys::Signature {
                name: stringify!($name),
                arguments: &[$(stringify!($ty)),*],
                returns: stringify!($ret),
                variadic: false,
            }, )*
        ];
    };
}

/// Declares integer constants and their `(name, value)` table `$table`.
macro_rules! foreign_constants {
    ($table:ident; $( $(#[$attr:meta])* $name:ident : $ty:ty = $value:expr ; )*) => {
        $( $(#[$attr])* pub const $name: $ty = $value; )*
        /// Names and values of the constants declared next to this table.
        pub const $table: &[(&str, i64)] = &[ $( (stringify!($name), $name as i64), )* ];
    };
}

pub(crate) use {callback_types, foreign_constants, foreign_functions, variadic_functions};
