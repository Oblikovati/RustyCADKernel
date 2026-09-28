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

//! The Unix-build `OSD_Path(name, sysType)` extractors: `UnixExtract`, `DosExtract`,
//! `VmsExtract` and `MacExtract`, kept statement by statement with their quirks.
use super::{
    CadPath, PathSyntax,
    text::{self, Text},
};
use crate::{osd::SysType, standard::Failure};

mod spelling;
pub(super) use spelling::{system_name, unix_name};

/// Dispatches on the dialect; `Default` is the host's own. Unknown, Taligent and Default
/// leave every component empty, as the native `default:` arm does.
pub(super) fn parse(path: &[u8], sys_type: SysType) -> Result<CadPath, Failure> {
    let todo: SysType = match sys_type {
        SysType::Default => PathSyntax::Unix.host_sys_type(),
        other => other,
    };
    match todo {
        SysType::Vms => vms_extract(path),
        SysType::LinuxRedHat
        | SysType::UnixBsd
        | SysType::UnixSystemV
        | SysType::Aix
        | SysType::Osf => Ok(parse_unix(path)),
        SysType::Os2 | SysType::WindowsNt => Ok(dos_extract(path)),
        SysType::MacOs => Ok(mac_extract(path)),
        _ => Ok(CadPath::empty(PathSyntax::Unix)),
    }
}

/// `UnixExtract`: optional `node:/` prefix, `/` to `|`, the name after the last `|`,
/// then `..` to `^` in the trek only, and the extension from the last dot.
pub(super) fn parse_unix(what: &[u8]) -> CadPath {
    let mut path: CadPath = CadPath::empty(PathSyntax::Unix);
    let mut buffer: Text = what.to_vec();
    if text::search(&buffer, b":/") != -1 {
        path.node = text::token(&buffer, b":/", 1);
        // The node plus ':' always fits: the buffer also holds ":/" outside the token.
        buffer.drain(..path.node.len() + 1);
    }
    path.trek = buffer.clone();
    text::change_all(&mut path.trek, b'/', b'|');
    split_name_after_last_bar(&mut path, &buffer);
    replace_parent_markers(&mut path.trek);
    path.extension = split_extension(&mut path.name);
    path
}

/// `DosExtract` (a DOS path read by the Unix build): `disk:` prefix, `\` to `|`, `..` to
/// `^` before the name is taken, then name and extension.
fn dos_extract(what: &[u8]) -> CadPath {
    let mut path: CadPath = CadPath::empty(PathSyntax::Unix);
    let mut buffer: Text = what.to_vec();
    if text::search(&buffer, b":") != -1 {
        path.disk = text::token(&buffer, b":", 1);
        path.disk.push(b':');
        // Native quirk: the removed length is the token's, wherever the colon was.
        buffer.drain(..path.disk.len().min(buffer.len()));
    }
    path.trek = buffer.clone();
    text::change_all(&mut path.trek, b'\\', b'|');
    replace_parent_markers(&mut path.trek);
    split_name_after_last_bar(&mut path, &buffer);
    path.extension = split_extension(&mut path.name);
    path
}

/// `MacExtract`: `::` to `^`, `:` to `|`, and a name that natively starts one byte after
/// the last `|` (so it loses its first character) and is removed from the trek at its
/// first occurrence.
fn mac_extract(what: &[u8]) -> CadPath {
    let mut path: CadPath = CadPath::empty(PathSyntax::Unix);
    path.trek = what.to_vec();
    let mut position: i32 = text::search(&path.trek, b"::");
    while position != -1 {
        path.trek[position as usize - 1] = b'^';
        path.trek.remove(position as usize);
        position = text::search(&path.trek, b"::");
    }
    text::change_all(&mut path.trek, b':', b'|');
    let last_bar: i32 = text::search_from_end(&path.trek, b"|");
    if last_bar == -1 {
        path.name = what.to_vec();
        path.trek.clear();
        return path;
    }
    // Native `&p[pos + 1]` skips the byte after the bar; past the terminator (a trailing
    // bar) it reads indeterminate memory, taken here as an empty name.
    path.name = path
        .trek
        .get(last_bar as usize + 1..)
        .unwrap_or_default()
        .to_vec();
    let found: i32 = text::search(&path.trek, &path.name);
    // Remove(Search(""), 0) is a native no-op, so an empty name leaves the trek.
    let _ = text::remove(&mut path.trek, found, text::len(&path.name));
    path
}

/// `VmsExtract`: `node"user password"::disk:[dir.sub]name.ext`.
fn vms_extract(what: &[u8]) -> Result<CadPath, Failure> {
    let mut path: CadPath = CadPath::empty(PathSyntax::Unix);
    let mut buffer: Text = what.to_vec();
    if text::search(&buffer, b"\"") != -1 {
        vms_account(&mut path, &mut buffer)?;
    } else if text::search(&buffer, b"::") != -1 {
        path.node = text::token(&buffer, b":", 1);
        text::remove(&mut buffer, 1, text::len(&path.node) + 2)?;
    }
    if text::search(&buffer, b":") != -1 {
        path.disk = text::token(&buffer, b":", 1);
        text::remove(&mut buffer, 1, text::len(&path.disk) + 1)?;
    }
    if text::search(&buffer, b"[") != -1 {
        path.trek = vms_trek(&buffer)?;
        path.name = text::token(&buffer, b"]", 2);
    } else {
        path.name = buffer;
    }
    vms_extension(&mut path)?;
    Ok(path)
}

/// The quoted `node"user password"::` account prefix of a VMS name.
fn vms_account(path: &mut CadPath, buffer: &mut Text) -> Result<(), Failure> {
    if text::value(buffer, 1)? != b'"' {
        path.node = text::token(buffer, b"\"", 1);
        text::remove(buffer, 1, text::len(&path.node))?;
    }
    path.user_name = text::token(buffer, b"\" ", 1);
    // Removes the opening quote, the user and the following space or quote.
    text::remove(buffer, 1, text::len(&path.user_name) + 2)?;
    if text::search(buffer, b"\"") != -1 {
        path.password = text::token(buffer, b"\"", 1);
        text::remove(buffer, 1, text::len(&path.password) + 1)?;
    }
    if text::search(buffer, b"::") != -1 {
        text::remove(buffer, 1, 2)?;
    }
    Ok(())
}

/// The `[dir.sub]` trek of a VMS name in portable syntax; `[000000]` is the root.
fn vms_trek(buffer: &[u8]) -> Result<Text, Failure> {
    let mut trek: Text = text::token(buffer, b"[]", 1);
    if text::value(&trek, 1)? == b'.' {
        trek.remove(0);
    } else {
        trek.insert(0, b'|');
    }
    text::change_all(&mut trek, b'.', b'|');
    text::change_all(&mut trek, b'-', b'^');
    let root: i32 = text::search(&trek, b"000000");
    if root != -1 {
        text::remove(&mut trek, root, 6)?;
        if text::search(&trek, b"||") != -1 {
            trek.remove(0);
        }
    }
    Ok(trek)
}

/// A VMS extension is the second dot-token with a dot prepended, removed from the name
/// at the first dot (so `a.b.c` keeps `a.c` as its name).
fn vms_extension(path: &mut CadPath) -> Result<(), Failure> {
    let dot: i32 = text::search(&path.name, b".");
    if dot == -1 {
        return Ok(());
    }
    path.extension = text::token(&path.name, b".", 2);
    path.extension.insert(0, b'.');
    text::remove(&mut path.name, dot, text::len(&path.extension))
}

/// Takes the name after the last `|` of the trek (`&p[pos]`), or the whole `buffer`
/// with no trek when there is no bar.
fn split_name_after_last_bar(path: &mut CadPath, buffer: &[u8]) {
    let last_bar: i32 = text::search_from_end(&path.trek, b"|");
    if last_bar == -1 {
        path.name = buffer.to_vec();
        path.trek.clear();
        return;
    }
    path.name = path.trek.split_off(last_bar as usize);
}

/// Replaces every `..` of the trek by `^`, searching again from the start each time.
fn replace_parent_markers(trek: &mut Text) {
    let mut position: i32 = text::search(trek, b"..");
    while position != -1 {
        trek[position as usize - 1] = b'^';
        trek.remove(position as usize);
        position = text::search(trek, b"..");
    }
}

/// Moves the text from the last dot on into the extension (`Split(pos - 1)`).
fn split_extension(name: &mut Text) -> Text {
    let dot: i32 = text::search_from_end(name, b".");
    if dot == -1 {
        return Text::new();
    }
    name.split_off(dot as usize - 1)
}
