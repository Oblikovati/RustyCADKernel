// Copyright (c) 1998-1999 Matra Datavision
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

//! Read-only file existence with explicit treatment of OCCT's empty-path regression.
use std::path::{Path, PathBuf};
/// Injectable filesystem metadata query; implement for deterministic tests.
pub trait FileStatusSource {
    /// Checks existence, treating inaccessible paths as absent; `source.exists(path)`.
    fn exists(&self, path: &Path) -> bool;
}
/// Native read-only metadata source; `NativeFileStatus.exists(path)`.
pub struct NativeFileStatus;
impl FileStatusSource for NativeFileStatus {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
/// Initial OSD_File path wrapper; `File::new("").exists()`.
pub struct File {
    path: PathBuf,
}
impl File {
    /// Stores a filesystem path without touching the filesystem; `File::new("part.step")`.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_owned(),
        }
    }
    /// Checks the native filesystem; empty paths return false without raising; `file.exists()`.
    pub fn exists(&self) -> bool {
        self.exists_with(&NativeFileStatus)
    }
    /// Queries injected metadata for nonempty paths; `file.exists_with(&source)`.
    pub fn exists_with(&self, source: &impl FileStatusSource) -> bool {
        !self.path.as_os_str().is_empty() && source.exists(&self.path)
    }
}
