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

//! OCCT's `FSD` package: file drivers of the `Storage` framework (text, compatible text
//! and binary formats) and the Base64 codec. Every driver opens its file through an
//! injected [`DriverFileSystem`].
mod base64;
mod binary_file;
mod c_file;
mod file_header;
mod file_system;
mod text_file;

pub use base64::Base64;
pub use binary_file::BinaryFile;
pub(crate) use c_file::CFile;
pub use file_header::FileHeader;
pub use file_system::{DriverFileSystem, FileOpenMode, MemoryFileSystem, NativeDriverFileSystem};
pub use text_file::{CmpDialect, CmpFile, File, FsdDialect, TextDialect, TextFile};
