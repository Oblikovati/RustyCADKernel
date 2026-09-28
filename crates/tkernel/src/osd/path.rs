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

//! Host path decomposition with OCCT's portable `|` trek and `^` parent markers.
use crate::tcollection::AsciiString;
/// Native parsing profile; `PathSyntax::Unix` preserves Unix separator rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathSyntax {
    /// Unix-host OSD_Path decomposition; `PathSyntax::Unix`.
    Unix,
    /// Windows-host OSD_Path decomposition; `PathSyntax::Windows`.
    Windows,
}
impl PathSyntax {
    /// Selects the current platform's parser; `PathSyntax::native()`.
    pub const fn native() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}
/// Portable path components; `CadPath::new("/tmp/part.step").extension()`.
#[derive(Clone, Debug)]
pub struct CadPath {
    syntax: PathSyntax,
    node: Vec<u8>,
    disk: Vec<u8>,
    trek: Vec<u8>,
    name: AsciiString,
    extension: AsciiString,
}
impl CadPath {
    /// Parses using the host's path syntax; `CadPath::new("/tmp/part.step")`.
    pub fn new(path: impl AsRef<[u8]>) -> Self {
        Self::with_syntax(path, PathSyntax::native())
    }
    /// Parses an explicit native-host profile; `CadPath::with_syntax("/tmp/part.step",PathSyntax::Unix)`.
    pub fn with_syntax(path: impl AsRef<[u8]>, syntax: PathSyntax) -> Self {
        let path = c_bytes(path.as_ref());
        let (node, disk, remaining) = extract_prefix(path, syntax);
        let split = remaining
            .iter()
            .rposition(|&byte| byte == b'/' || (syntax == PathSyntax::Windows && byte == b'\\'))
            .map_or(0, |index| index + 1);
        let mut trek = portable_trek(&remaining[..split], syntax);
        if syntax == PathSyntax::Windows {
            normalize_windows(&mut trek);
        }
        let filename = &remaining[split..];
        let dot = filename
            .iter()
            .rposition(|&byte| byte == b'.')
            .unwrap_or(filename.len());
        Self {
            syntax,
            node,
            disk,
            trek,
            name: AsciiString::from_raw(&filename[..dot]),
            extension: AsciiString::from_raw(&filename[dot..]),
        }
    }
    /// Returns the portable directory trek; `path.trek().as_bytes()`.
    pub fn trek(&self) -> AsciiString {
        let mut trek = self.trek.clone();
        if self.syntax == PathSyntax::Windows && !trek.is_empty() && trek.last() != Some(&b'|') {
            trek.push(b'|');
        }
        AsciiString::from_raw(&trek)
    }
    /// Borrows the filename without its final extension; `path.name()`.
    pub fn name(&self) -> &AsciiString {
        &self.name
    }
    /// Borrows the final extension including its dot; `path.extension()`.
    pub fn extension(&self) -> &AsciiString {
        &self.extension
    }
    /// Removes the final directory component using host-specific OCCT rules; `path.up_trek()`.
    pub fn up_trek(&mut self) {
        if self.syntax == PathSyntax::Windows {
            self.up_windows();
        } else {
            self.up_unix();
        }
    }
    /// Reconstructs the host system spelling; `path.system_name()`.
    pub fn system_name(&self) -> AsciiString {
        let mut output = if self.syntax == PathSyntax::Windows {
            self.disk.clone()
        } else {
            self.node.clone()
        };
        if self.syntax == PathSyntax::Unix && !output.is_empty() {
            output.push(b':');
        }
        for &byte in &self.trek {
            match byte {
                b'|' => output.push(b'/'),
                b'^' => output.extend_from_slice(b".."),
                _ => output.push(byte),
            }
        }
        let needs_slash = if self.syntax == PathSyntax::Windows {
            !self.trek.is_empty()
        } else {
            !output.is_empty() && output.last() != Some(&b'/')
        };
        if needs_slash {
            output.push(b'/');
        }
        output.extend_from_slice(self.name.as_bytes());
        output.extend_from_slice(self.extension.as_bytes());
        AsciiString::from_raw(&output)
    }
    /// Splits at the last slash of either spelling, preserving it in the folder; `CadPath::folder_and_file_from_path("a/b")`.
    pub fn folder_and_file_from_path(path: impl AsRef<[u8]>) -> (AsciiString, AsciiString) {
        let bytes = path.as_ref();
        let index = bytes
            .iter()
            .rposition(|&byte| byte == b'/' || byte == b'\\')
            .map_or(0, |index| index + 1);
        (
            AsciiString::from_raw(&bytes[..index]),
            AsciiString::from_raw(&bytes[index..]),
        )
    }
    fn up_unix(&mut self) {
        let Some(end) = self
            .trek
            .iter()
            .rposition(|&byte| byte != b'|')
            .map(|index| index + 1)
        else {
            return;
        };
        let start = self.trek[..end]
            .iter()
            .rposition(|&byte| byte == b'|')
            .map_or(0, |index| index + 1);
        self.trek.drain(start..end);
        if let Some(index) = self.trek.windows(2).position(|pair| pair == b"||") {
            self.trek.remove(index);
        }
    }
    fn up_windows(&mut self) {
        let Some(mut position) = self.trek.iter().rposition(|&byte| byte == b'|') else {
            self.trek.clear();
            return;
        };
        while position > 0 && self.trek[position] == b'|' {
            position -= 1;
        }
        self.trek.truncate(position + 1);
    }
}
impl Default for CadPath {
    fn default() -> Self {
        Self::new("")
    }
}
fn c_bytes(bytes: &[u8]) -> &[u8] {
    &bytes[..bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len())]
}
fn extract_prefix(path: &[u8], syntax: PathSyntax) -> (Vec<u8>, Vec<u8>, &[u8]) {
    if syntax == PathSyntax::Windows && path.get(1) == Some(&b':') {
        return (Vec::new(), path[..2].to_vec(), &path[2..]);
    }
    if syntax == PathSyntax::Unix && path.windows(2).any(|pair| pair == b":/") {
        let node = path
            .split(|&byte| byte == b':' || byte == b'/')
            .find(|part| !part.is_empty())
            .unwrap_or_default();
        return (
            node.to_vec(),
            Vec::new(),
            &path[(node.len() + 1).min(path.len())..],
        );
    }
    (Vec::new(), Vec::new(), path)
}
fn portable_trek(bytes: &[u8], syntax: PathSyntax) -> Vec<u8> {
    let mut trek = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"..") {
            trek.push(b'^');
            index += 2;
            continue;
        }
        let byte = bytes[index];
        trek.push(
            if byte == b'/' || (syntax == PathSyntax::Windows && byte == b'\\') {
                b'|'
            } else {
                byte
            },
        );
        index += 1;
    }
    trek
}
fn normalize_windows(trek: &mut Vec<u8>) {
    let original_len = trek.len();
    let mut position = 1;
    while position < trek.len() {
        if trek[position] == b'|' && trek[position - 1] == b'|' && position > 1 {
            trek.remove(position);
        } else {
            position += 1;
        }
    }
    if original_len > 1 && trek.last() == Some(&b'|') {
        trek.pop();
    }
    let mut position = 1;
    while position < trek.len() {
        if trek[position] == b'^' && trek[position - 1] == b'^' {
            trek.remove(position);
        } else {
            position += 1;
        }
    }
}
mod predicates;
