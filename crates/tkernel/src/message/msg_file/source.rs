// Created on: 2001-04-26
// Created by: OCC Team
// Copyright (c) 2001-2014 OPEN CASCADE SAS
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

//! File-system and environment seam of the message catalog.
use std::fs;

/// Where `MsgFile` reads files and environment variables; inject a named fake in tests.
pub trait MsgFileSource: Send + Sync {
    /// The complete file content, or `None` when it cannot be opened or read (`fopen`/`fread`
    /// failure); `source.read_file("messages.us")`.
    fn read_file(&self, path: &str) -> Option<Vec<u8>>;
    /// An environment variable (`OSD_Environment::Value`); `source.variable("CSF_LANGUAGE")`.
    fn variable(&self, name: &str) -> Option<String>;
}

/// Host file system and process environment; `Arc::new(NativeMsgFileSource)`.
pub struct NativeMsgFileSource;
impl MsgFileSource for NativeMsgFileSource {
    fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        fs::read(path).ok()
    }
    fn variable(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}
