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

//! The Windows-build `OSD_Path`: `_splitpath` parsing, `_remove_dup` trek normalization,
//! `SystemName` and the dialect check `_test_raise`. Pure byte logic, compiled on every
//! host so the Linux tests cover it.
use super::{
    CadPath, PathSyntax,
    text::{self, Text},
};
use crate::{osd::SysType, standard::Failure};

/// `_MAX_PATH`: the native SystemName trek buffer.
const MAX_PATH: usize = 260;

/// `_test_raise`: only Default and WindowsNT are accepted; the message is the native
/// `"OSD_Path :: <caller> (): unknown system type"`.
pub(super) fn check_sys_type(sys_type: SysType, caller: &str) -> Result<(), Failure> {
    if matches!(sys_type, SysType::Default | SysType::WindowsNt) {
        return Ok(());
    }
    let message: String = format!("OSD_Path :: {caller} (): unknown system type");
    Err(Failure::with_type("Standard_ProgramError", &message, ""))
}

/// `OSD_Path(name)` of the Windows build.
pub(super) fn parse(what: &[u8]) -> CadPath {
    let mut path: CadPath = CadPath::empty(PathSyntax::Windows);
    let (drive, directory, name, extension) = split_path(what);
    path.disk = drive.to_vec();
    path.name = name.to_vec();
    path.extension = extension.to_vec();
    path.trek = portable_trek(directory);
    remove_duplicates(&mut path.trek);
    path
}

/// The MSVC `_splitpath` split: a drive when the second byte is ':', the directory up to
/// the last `/` or `\`, and the extension from the last dot after it. The CRT's
/// `_MAX_*` component limits are not reproduced (paths over 256 bytes per component).
fn split_path(path: &[u8]) -> (&[u8], &[u8], &[u8], &[u8]) {
    let drive_len: usize = if path.get(1) == Some(&b':') { 2 } else { 0 };
    let (drive, rest) = path.split_at(drive_len);
    let directory_len: usize = rest
        .iter()
        .rposition(|&byte| byte == b'/' || byte == b'\\')
        .map_or(0, |index| index + 1);
    let (directory, file) = rest.split_at(directory_len);
    let dot: usize = file
        .iter()
        .rposition(|&byte| byte == b'.')
        .unwrap_or(file.len());
    (drive, directory, &file[..dot], &file[dot..])
}

/// Separators to `|`, `..` to `^`, other bytes kept.
fn portable_trek(directory: &[u8]) -> Text {
    let mut trek: Text = Text::with_capacity(directory.len());
    let mut index: usize = 0;
    while index < directory.len() {
        let byte: u8 = directory[index];
        if byte == b'\\' || byte == b'/' {
            trek.push(b'|');
        } else if byte == b'.' && directory.get(index + 1) == Some(&b'.') {
            trek.push(b'^');
            index += 1;
        } else {
            trek.push(byte);
        }
        index += 1;
    }
    trek
}

/// `_remove_dup`: collapses `||` runs except a leading one (UNC), drops one trailing `|`
/// when the original was longer than one byte, then collapses `^^` runs.
pub(super) fn remove_duplicates(trek: &mut Text) {
    let original_len: usize = trek.len();
    collapse_runs(trek, b'|', true);
    if original_len > 1 && trek.last() == Some(&b'|') {
        trek.pop();
    }
    collapse_runs(trek, b'^', false);
}

/// Keeps the first byte of every run of `marker` (a run starting at position 1 is kept
/// whole when `keep_leading`).
fn collapse_runs(trek: &mut Text, marker: u8, keep_leading: bool) {
    let mut position: usize = 0;
    while position < trek.len() {
        let run_start: bool = trek[position] == marker && trek.get(position + 1) == Some(&marker);
        if !run_start || (keep_leading && position == 0) {
            position += 1;
            continue;
        }
        position += 1;
        while trek.get(position) == Some(&marker) {
            trek.remove(position);
        }
    }
}

/// `SetValues` post-processing: a missing extension dot is prefixed, the trek is
/// normalized.
pub(super) fn normalize_values(path: &mut CadPath) {
    if path.extension.first().is_some_and(|&byte| byte != b'.') {
        path.extension.insert(0, b'.');
    }
    remove_duplicates(&mut path.trek);
}

/// `SystemName`: disk, trek with `/` and `..`, a `/` when the trek is not empty, name
/// and extension. Node, user and password are not spelled.
pub(super) fn system_name(path: &CadPath) -> Text {
    let trek: Text = system_trek(&path.trek);
    let mut full: Text = path.disk.clone();
    full.extend_from_slice(&trek);
    if !trek.is_empty() {
        full.push(b'/');
    }
    full.extend_from_slice(&path.name);
    full.extend_from_slice(&path.extension);
    full
}

/// The trek into the native 260-byte buffer: a `^` that no longer leaves room for `..`
/// is copied as `^`, and bytes past the buffer are dropped.
fn system_trek(trek: &[u8]) -> Text {
    let mut out: Text = Text::with_capacity(trek.len());
    for &byte in trek {
        if out.len() >= MAX_PATH {
            break;
        }
        match byte {
            b'|' => out.push(b'/'),
            b'^' if out.len() < MAX_PATH - 1 => out.extend_from_slice(b".."),
            other => out.push(other),
        }
    }
    out
}

/// `UpTrek`: truncates before the last `|`, and before any `|` run that precedes it.
pub(super) fn up_trek(trek: &mut Text) {
    let mut position: i32 = text::search_from_end(trek, b"|");
    if position == -1 {
        position = 0;
    } else if position > 1 {
        while trek[position as usize - 1] == b'|' && position != 1 {
            position -= 1;
        }
    }
    trek.truncate(position as usize);
}

/// `DownTrek`: joins with `|` unless either side already has one, then normalizes.
pub(super) fn down_trek(trek: &mut Text, name: &[u8]) {
    if name.first().is_some_and(|&byte| byte != b'|')
        && trek.last().is_some_and(|&byte| byte != b'|')
    {
        trek.push(b'|');
    }
    trek.extend_from_slice(name);
    remove_duplicates(trek);
}

/// `TrekLength`: the number of `|` tokens; 0 for an empty trek or a lone `|`.
pub(super) fn trek_length(trek: &[u8]) -> i32 {
    if trek.is_empty() || trek == b"|" {
        return 0;
    }
    trek.split(|&byte| byte == b'|')
        .filter(|part| !part.is_empty())
        .count() as i32
}

/// `RemoveATrek(where)`: removes the where-th `|`-led component (a leading `|` is added
/// and removed around the edit when missing). Raises for an empty trek.
pub(super) fn remove_at(trek: &mut Text, at: i32) -> Result<(), Failure> {
    if trek_length(trek) < at {
        return Ok(());
    }
    let added: bool = lead_with_bar(trek)?;
    let first: i32 = text::location(trek, at, b'|', 1, text::len(trek))?;
    if first != 0 {
        let mut next: i32 = text::location(trek, at + 1, b'|', 1, text::len(trek))?;
        if next == 0 {
            next = text::len(trek) + 1;
        }
        text::remove(trek, first, next - first)?;
    }
    if added {
        text::remove(trek, 1, 1)?;
    }
    Ok(())
}

/// `RemoveATrek(name)`: removes `|name|` (bars added as needed) at its first occurrence.
pub(super) fn remove_named(trek: &mut Text, name: &[u8]) -> Result<(), Failure> {
    let added: bool = lead_with_bar(trek)?;
    trek.push(b'|');
    let mut wanted: Text = name.to_vec();
    if text::value(&wanted, 1)? != b'|' {
        wanted.insert(0, b'|');
    }
    if text::last(&wanted)? != b'|' {
        wanted.push(b'|');
    }
    let found: i32 = text::search(trek, &wanted);
    if found != -1 {
        text::remove(trek, found + 1, text::len(&wanted) - 1)?;
    }
    if added {
        text::remove(trek, 1, 1)?;
    }
    drop_trailing_bar(trek)
}

/// `TrekValue(where)`: the where-th token of the trek (a leading `|` added when missing).
pub(super) fn value_at(trek: &[u8], at: i32) -> Result<Text, Failure> {
    let mut led: Text = trek.to_vec();
    lead_with_bar(&mut led)?;
    Ok(text::token(&led, b"|", at))
}

/// `InsertATrek(name, where)`: inserts `name|` after the where-th `|`, then normalizes.
pub(super) fn insert_at(trek: &mut Text, name: &[u8], at: i32) -> Result<(), Failure> {
    let added: bool = lead_with_bar(trek)?;
    trek.push(b'|');
    let position: i32 = text::location(trek, at, b'|', 1, text::len(trek))?;
    if position != 0 {
        let mut inserted: Text = name.to_vec();
        if text::last(&inserted)? != b'|' {
            inserted.push(b'|');
        }
        text::insert(trek, position + 1, &inserted)?;
    }
    if added {
        text::remove(trek, 1, 1)?;
    }
    drop_trailing_bar(trek)?;
    remove_duplicates(trek);
    Ok(())
}

/// `if (myTrek.Value(1) != '|') { flag = true; myTrek.Insert(1, '|'); }`: raises for an
/// empty trek, as `Value(1)` does.
fn lead_with_bar(trek: &mut Text) -> Result<bool, Failure> {
    if text::value(trek, 1)? == b'|' {
        return Ok(false);
    }
    trek.insert(0, b'|');
    Ok(true)
}

/// `if (myTrek.Value(myTrek.Length()) == '|') myTrek.Trunc(myTrek.Length() - 1)`.
fn drop_trailing_bar(trek: &mut Text) -> Result<(), Failure> {
    if text::last(trek)? == b'|' {
        trek.pop();
    }
    Ok(())
}
