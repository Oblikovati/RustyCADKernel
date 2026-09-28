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

//! The Unix-build `OSD_Path::SystemName`: the trek converters `P2VMS`, `P2DOS`, `P2MAC`,
//! `P2UNIX` and the assembly of each dialect's full name.
use super::super::{
    CadPath,
    text::{self, Text},
};
use crate::{osd::SysType, standard::Failure};

/// `SystemName(fullName, sysType)` of the Unix build, `sys_type` already resolved from
/// Default; every dialect but VMS, OS2/WindowsNT and MacOs is spelled the Unix way.
pub(in crate::osd::path) fn system_name(
    path: &CadPath,
    sys_type: SysType,
) -> Result<Text, Failure> {
    Ok(match sys_type {
        SysType::Vms => vms_name(path),
        SysType::Os2 | SysType::WindowsNt => dos_name(path)?,
        SysType::MacOs => mac_name(path),
        _ => unix_name(path),
    })
}

/// `node"user password"::disk:[dir.sub]name.ext`; a user without a node gets node "0".
fn vms_name(path: &CadPath) -> Text {
    let way: Text = trek_to_vms(&path.trek);
    let mut node: Text = path.node.clone();
    let mut full: Text = path.node.clone();
    if !path.user_name.is_empty() {
        if node.is_empty() {
            node = b"0".to_vec();
            full.extend_from_slice(&node);
        }
        full.push(b'"');
        full.extend_from_slice(&path.user_name);
        if !path.password.is_empty() {
            full.push(b' ');
            full.extend_from_slice(&path.password);
        }
        full.push(b'"');
    }
    if !node.is_empty() {
        full.extend_from_slice(b"::");
    }
    if !path.disk.is_empty() {
        full.extend_from_slice(&path.disk);
        full.push(b':');
    }
    // Native quirk: name and extension are appended only when there is a trek.
    if !way.is_empty() {
        full.push(b'[');
        full.extend_from_slice(&way);
        full.push(b']');
        full.extend_from_slice(&path.name);
        full.extend_from_slice(&path.extension);
    }
    full
}

/// `P2VMS`: drop a leading `|` (else prepend one), then `|` to `.` and `^` to `-`.
fn trek_to_vms(trek: &[u8]) -> Text {
    let mut way: Text = trek.to_vec();
    if way.is_empty() {
        return way;
    }
    // The native `Value(1) != '\0'` test is always true here, so the "[000000]" root
    // spelling is unreachable.
    if way[0] == b'|' {
        way.remove(0);
    } else {
        way.insert(0, b'|');
    }
    way.iter_mut().for_each(|byte| {
        *byte = match *byte {
            b'|' => b'.',
            b'^' => b'-',
            other => other,
        }
    });
    way
}

/// MSDOS-like names: a one-letter-plus-colon disk is kept, any other disk (except a
/// single character, which is dropped) becomes a `\disk\` prefix without `$`.
///
/// Native `Way.Value(1)` raises `Standard_OutOfRange` for such a disk with an empty trek.
fn dos_name(path: &CadPath) -> Result<Text, Failure> {
    let way: Text = trek_to_dos(&path.trek);
    let mut full: Text = Text::new();
    let disk: &[u8] = &path.disk;
    if disk.len() != 1 && !disk.is_empty() {
        if disk.len() == 2 && disk[0].is_ascii_alphabetic() && disk[1] == b':' {
            full.extend_from_slice(disk);
        } else {
            full.push(b'\\');
            full.extend(disk.iter().copied().filter(|&byte| byte != b'$'));
            if text::value(&way, 1)? != b'\\' {
                full.push(b'\\');
            }
        }
    }
    if !way.is_empty() {
        full.extend_from_slice(&way);
        full.push(b'\\');
    }
    full.extend_from_slice(&path.name);
    full.extend_from_slice(&path.extension);
    Ok(full)
}

/// `P2DOS`: drop one trailing `|`, `|` to `\`, `^` to `..`.
fn trek_to_dos(trek: &[u8]) -> Text {
    let mut way: Text = trek.to_vec();
    if way.last() == Some(&b'|') {
        way.pop();
    }
    let mut out: Text = Text::with_capacity(way.len());
    for byte in way {
        match byte {
            b'|' => out.push(b'\\'),
            b'^' => out.extend_from_slice(b".."),
            other => out.push(other),
        }
    }
    out
}

/// `disk:` then the name; the converted trek is computed but, natively, never appended.
fn mac_name(path: &CadPath) -> Text {
    let mut full: Text = Text::new();
    if !path.disk.is_empty() {
        full.extend_from_slice(&path.disk);
        full.push(b':');
    }
    full.extend_from_slice(&path.name);
    full.extend_from_slice(&path.extension);
    full
}

/// `user"password"@node:/disk/trek/name.ext` (the user part only with a node).
pub(in crate::osd::path) fn unix_name(path: &CadPath) -> Text {
    let mut full: Text = Text::new();
    if !path.user_name.is_empty() && !path.node.is_empty() {
        full.extend_from_slice(&path.user_name);
        if !path.password.is_empty() {
            full.push(b'"');
            full.extend_from_slice(&path.password);
            full.push(b'"');
        }
        full.push(b'@');
    }
    if !path.node.is_empty() {
        full.extend_from_slice(&path.node);
        full.push(b':');
    }
    if !path.disk.is_empty() {
        full.push(b'/');
        full.extend(path.disk.iter().copied().filter(|&byte| byte != b'$'));
    }
    full.extend_from_slice(&trek_to_unix(&path.trek));
    if full.last().is_some_and(|&byte| byte != b'/') {
        full.push(b'/');
    }
    full.extend_from_slice(&path.name);
    full.extend_from_slice(&path.extension);
    full
}

/// `P2UNIX`: `|` to `/`, `^` to `..`.
fn trek_to_unix(trek: &[u8]) -> Text {
    let mut out: Text = Text::with_capacity(trek.len());
    for &byte in trek {
        match byte {
            b'|' => out.push(b'/'),
            b'^' => out.extend_from_slice(b".."),
            other => out.push(other),
        }
    }
    out
}
