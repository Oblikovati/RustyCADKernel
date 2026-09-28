// Copyright (c) 1996-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! The Storage enumerations and the `Storage_Stream*Error` exception family.
use crate::standard::Failure;

/// `Storage_Error`: the status of a storage or retrieval step; `StorageError::Ok`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StorageError {
    /// `Storage_VSOk`.
    #[default]
    Ok,
    /// `Storage_VSOpenError`.
    OpenError,
    /// `Storage_VSModeError`.
    ModeError,
    /// `Storage_VSCloseError`.
    CloseError,
    /// `Storage_VSAlreadyOpen`.
    AlreadyOpen,
    /// `Storage_VSNotOpen`.
    NotOpen,
    /// `Storage_VSSectionNotFound`.
    SectionNotFound,
    /// `Storage_VSWriteError`.
    WriteError,
    /// `Storage_VSFormatError`.
    FormatError,
    /// `Storage_VSUnknownType`.
    UnknownType,
    /// `Storage_VSTypeMismatch`.
    TypeMismatch,
    /// `Storage_VSInternalError`.
    InternalError,
    /// `Storage_VSExtCharParityError`.
    ExtCharParityError,
    /// `Storage_VSWrongFileDriver`.
    WrongFileDriver,
}

impl StorageError {
    /// The native enumerator value, used by probes and dumps; `StorageError::FormatError.ordinal() == 8`.
    pub fn ordinal(self) -> i32 {
        self as i32
    }
}

/// `Storage_OpenMode`: how a driver opened its container; `OpenMode::Read`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OpenMode {
    /// `Storage_VSNone`: not open.
    #[default]
    None,
    /// `Storage_VSRead`.
    Read,
    /// `Storage_VSWrite`.
    Write,
    /// `Storage_VSReadWrite`.
    ReadWrite,
}

impl OpenMode {
    /// The native enumerator value; `OpenMode::Write.ordinal() == 2`.
    pub fn ordinal(self) -> i32 {
        self as i32
    }
}

/// `Storage_SolveMode`: which schema step resolves an unknown type; `SolveMode::ReadSolve`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolveMode {
    /// `Storage_AddSolve`.
    AddSolve,
    /// `Storage_WriteSolve`.
    WriteSolve,
    /// `Storage_ReadSolve`.
    ReadSolve,
}

/// `Storage_Position`: a `long` stream offset, -1 on error; `let position: Position = -1;`.
pub type Position = i64;

crate::standard::exceptions::define_standard_exceptions! {
    STORAGE_HIERARCHY;
    stream_ext_char_parity_error => "Storage_StreamExtCharParityError" : "Storage_StreamReadError";
    stream_format_error => "Storage_StreamFormatError" : "Standard_Failure";
    stream_mode_error => "Storage_StreamModeError" : "Standard_Failure";
    stream_read_error => "Storage_StreamReadError" : "Standard_Failure";
    stream_type_mismatch_error => "Storage_StreamTypeMismatchError" : "Storage_StreamReadError";
    stream_unknown_type_error => "Storage_StreamUnknownTypeError" : "Storage_StreamReadError";
    stream_write_error => "Storage_StreamWriteError" : "Standard_Failure";
}

/// The `Storage_Stream*Error` exception classes, as a typed name for raising and catching.
///
/// ```
/// use rustycad_tkernel::storage::StreamException;
/// let failure = StreamException::TypeMismatch.raise("GetInteger: expected 4 bytes");
/// assert!(StreamException::TypeMismatch.is(&failure));
/// assert!(StreamException::Read.is(&failure));
/// assert_eq!(failure.exception_type(), "Storage_StreamTypeMismatchError");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamException {
    /// `Storage_StreamExtCharParityError`.
    ExtCharParity,
    /// `Storage_StreamFormatError`.
    Format,
    /// `Storage_StreamModeError`.
    Mode,
    /// `Storage_StreamReadError`.
    Read,
    /// `Storage_StreamTypeMismatchError`.
    TypeMismatch,
    /// `Storage_StreamUnknownTypeError`.
    UnknownType,
    /// `Storage_StreamWriteError`.
    Write,
}

impl StreamException {
    /// The native class name; `StreamException::Read.type_name() == "Storage_StreamReadError"`.
    pub fn type_name(self) -> &'static str {
        match self {
            Self::ExtCharParity => "Storage_StreamExtCharParityError",
            Self::Format => "Storage_StreamFormatError",
            Self::Mode => "Storage_StreamModeError",
            Self::Read => "Storage_StreamReadError",
            Self::TypeMismatch => "Storage_StreamTypeMismatchError",
            Self::UnknownType => "Storage_StreamUnknownTypeError",
            Self::Write => "Storage_StreamWriteError",
        }
    }

    /// Builds the exception with a diagnostic message; `StreamException::Format.raise("...")`.
    pub fn raise(self, message: &str) -> Failure {
        Failure::with_message(self.type_name(), message)
    }

    /// The `catch (Storage_Stream...Error const&)` test, which also catches derived classes;
    /// `StreamException::Read.is(&failure)`.
    pub fn is(self, failure: &Failure) -> bool {
        failure.is_kind(self.type_name())
    }
}
