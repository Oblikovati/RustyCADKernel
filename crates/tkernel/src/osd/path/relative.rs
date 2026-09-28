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

//! The build-independent `OSD_Path` statics: `IsValid`, `RelativePath`, `AbsolutePath`,
//! `FolderAndFileFromPath` and `FileNameAndExtension`.
use super::{
    CadPath, PathSyntax, c_bytes,
    text::{self, Text},
};
use crate::{osd::SysType, standard::Failure, tcollection::AsciiString};

/// `IsValid`: empty names are valid; VMS, DOS and MacOS names reject their dialect's
/// forbidden characters; every other dialect accepts anything.
pub(super) fn is_valid(name: &[u8], sys_type: SysType, syntax: PathSyntax) -> bool {
    if name.is_empty() {
        return true;
    }
    let has = |bytes: &[u8]| bytes.iter().any(|byte| name.contains(byte));
    match resolve(sys_type, syntax) {
        SysType::Vms => !has(b"/@\\"),
        SysType::Os2 | SysType::WindowsNt => !has(b"/:*?\"<>|"),
        // Native quirk: a name with a colon is valid; one without must fit 31 bytes.
        SysType::MacOs => name.contains(&b':') || name.len() <= 31,
        _ => true,
    }
}

fn resolve(sys_type: SysType, syntax: PathSyntax) -> SysType {
    match sys_type {
        SysType::Default => syntax.host_sys_type(),
        other => other,
    }
}

/// `RemoveExtraSeparator`: collapses `//` runs and drops one trailing `/`; the Windows
/// build keeps a leading `//`. Raises `Standard_OutOfRange` for an empty string.
fn remove_extra_separator(path: &mut Text, syntax: PathSyntax) -> Result<i32, Failure> {
    let unc: bool = syntax == PathSyntax::Windows && path.starts_with(b"//");
    let start: usize = usize::from(unc);
    let mut kept: Text = path[..start].to_vec();
    for &byte in &path[start..] {
        if byte == b'/' && kept.len() > start && kept.last() == Some(&b'/') {
            continue;
        }
        kept.push(byte);
    }
    let mut length: i32 = text::len(&kept);
    if text::value(&kept, length)? == b'/' {
        length -= 1;
    }
    kept.truncate(length as usize);
    *path = kept;
    Ok(length)
}

impl CadPath {
    /// `RelativePath(dirPath, absFilePath)` on the running host: the path of an absolute
    /// file relative to an absolute directory, or empty when there is none;
    /// `CadPath::relative_path("/a/b", "/a/c/f")? == "../c/f"`.
    ///
    /// An empty directory raises `Standard_OutOfRange`, as natively.
    pub fn relative_path(
        dir_path: impl AsRef<[u8]>,
        abs_file_path: impl AsRef<[u8]>,
    ) -> Result<AsciiString, Failure> {
        Self::relative_path_on(dir_path, abs_file_path, PathSyntax::native())
    }
    /// `RelativePath` with an explicit build (the Windows build keeps a leading `//`);
    /// `CadPath::relative_path_on("C:\\a", "C:\\a\\f", PathSyntax::Windows)?`.
    pub fn relative_path_on(
        dir_path: impl AsRef<[u8]>,
        abs_file_path: impl AsRef<[u8]>,
        syntax: PathSyntax,
    ) -> Result<AsciiString, Failure> {
        let dir: &[u8] = c_bytes(dir_path.as_ref());
        let file: &[u8] = c_bytes(abs_file_path.as_ref());
        let relative: Text = relative_text(dir, file, syntax)?;
        Ok(AsciiString::from_raw(&relative))
    }
    /// `AbsolutePath(dirPath, relFilePath)` on the running host: `relFilePath` resolved
    /// against an absolute directory, or empty when it climbs above the root;
    /// `CadPath::absolute_path("/a/b", "../c")? == "/a/c"`.
    ///
    /// An empty relative path raises `Standard_OutOfRange`, as natively.
    pub fn absolute_path(
        dir_path: impl AsRef<[u8]>,
        rel_file_path: impl AsRef<[u8]>,
    ) -> Result<AsciiString, Failure> {
        Self::absolute_path_on(dir_path, rel_file_path, PathSyntax::native())
    }
    /// `AbsolutePath` with an explicit build;
    /// `CadPath::absolute_path_on("//srv/a", "b", PathSyntax::Windows)?`.
    pub fn absolute_path_on(
        dir_path: impl AsRef<[u8]>,
        rel_file_path: impl AsRef<[u8]>,
        syntax: PathSyntax,
    ) -> Result<AsciiString, Failure> {
        let dir: &[u8] = c_bytes(dir_path.as_ref());
        let rel: &[u8] = c_bytes(rel_file_path.as_ref());
        Ok(AsciiString::from_raw(&absolute_text(dir, rel, syntax)?))
    }
    /// `FolderAndFileFromPath`: splits after the last `/` or `\`, keeping it in the
    /// folder; `CadPath::folder_and_file_from_path("a/b") == ("a/", "b")`.
    pub fn folder_and_file_from_path(path: impl AsRef<[u8]>) -> (AsciiString, AsciiString) {
        let bytes: &[u8] = c_bytes(path.as_ref());
        let split: usize = bytes
            .iter()
            .rposition(|&byte| byte == b'/' || byte == b'\\')
            .map_or(0, |index| index + 1);
        (
            AsciiString::from_raw(&bytes[..split]),
            AsciiString::from_raw(&bytes[split..]),
        )
    }
    /// `FileNameAndExtension`: an extension of 1 to 19 bytes after a dot that is not the
    /// first or second byte, lowercased; `CadPath::file_name_and_extension("A.STEP")`.
    pub fn file_name_and_extension(path: impl AsRef<[u8]>) -> (AsciiString, AsciiString) {
        const EXT_MAX_LEN: usize = 20;
        let bytes: &[u8] = c_bytes(path.as_ref());
        let length: usize = bytes.len();
        for ext_len in (1..length).take_while(|&ext_len| ext_len < EXT_MAX_LEN) {
            if bytes[length - ext_len - 1] != b'.' {
                continue;
            }
            // A dot in first position leaves no name: the whole path is the name.
            if length - ext_len < 2 {
                break;
            }
            let extension: Text = bytes[length - ext_len..].to_ascii_lowercase();
            return (
                AsciiString::from_raw(&bytes[..length - ext_len - 1]),
                AsciiString::from_raw(&extension),
            );
        }
        (AsciiString::from_raw(bytes), AsciiString::default())
    }
}

fn relative_text(dir: &[u8], file: &[u8], syntax: PathSyntax) -> Result<Text, Failure> {
    let mut file_path: Text = file.to_vec();
    let windows_style: bool = text::search(dir, b":") == 2;
    if windows_style {
        let same_drive: bool = text::search(&file_path, b":") == 2
            && text::value(dir, 1)?.eq_ignore_ascii_case(&text::value(&file_path, 1)?);
        if !same_drive {
            return Ok(Text::new());
        }
        text::change_all(&mut file_path, b'\\', b'/');
        if text::search(&file_path, b"/") != 3 {
            return Ok(Text::new());
        }
    } else if text::value(dir, 1)? != b'/' || text::value(&file_path, 1)? != b'/' {
        return Ok(Text::new());
    }
    let length: i32 = remove_extra_separator(&mut file_path, syntax)?;
    if !windows_style {
        if length < 2 {
            return Ok(Text::new());
        }
        file_path = text::sub_string(&file_path, 2, length)?;
    }
    climb_relative(dir, file_path, windows_style)
}

/// Strips the directory tokens the file shares, then prefixes `../` for every other one.
fn climb_relative(dir: &[u8], mut file_path: Text, windows_style: bool) -> Result<Text, Failure> {
    let mut sibling: bool = false;
    for n in 1.. {
        let mut dir_token: Text = text::token(dir, b"/\\", n);
        if dir_token.is_empty() {
            return Ok(file_path);
        }
        if !sibling {
            let slash: i32 = text::search(&file_path, b"/");
            if slash > 0 {
                if slash == text::len(&file_path) {
                    return Ok(Text::new());
                }
                let mut file_token: Text = text::sub_string(&file_path, 1, slash - 1)?;
                if windows_style {
                    dir_token.make_ascii_uppercase();
                    file_token.make_ascii_uppercase();
                }
                if dir_token == file_token {
                    file_path = text::sub_string(&file_path, slash + 1, text::len(&file_path))?;
                    continue;
                }
            } else if dir_token == file_path {
                return Ok(Text::new());
            } else {
                sibling = true;
            }
        }
        file_path.splice(0..0, b"../".iter().copied());
    }
    unreachable!("the token loop returns once the directory has no more tokens")
}

fn absolute_text(dir: &[u8], rel: &[u8], syntax: PathSyntax) -> Result<Text, Failure> {
    if text::search(rel, b"/") == 1 || text::search(rel, b":") == 2 {
        return Ok(rel.to_vec());
    }
    if text::search(dir, b"/") != 1 && text::search(dir, b":") != 2 {
        return Ok(Text::new());
    }
    let mut dir_path: Text = dir.to_vec();
    let mut rel_path: Text = rel.to_vec();
    if text::search(&dir_path, b":") == 2 {
        text::change_all(&mut dir_path, b'\\', b'/');
    }
    text::change_all(&mut rel_path, b'\\', b'/');
    remove_extra_separator(&mut dir_path, syntax)?;
    let mut length: i32 = remove_extra_separator(&mut rel_path, syntax)?;
    while text::search(&rel_path, b"../") == 1 {
        if length == 3 {
            return Ok(Text::new());
        }
        rel_path = text::sub_string(&rel_path, 4, length)?;
        length -= 3;
        let slash: i32 = text::search_from_end(&dir_path, b"/");
        if dir_path.is_empty() || slash < 0 {
            return Ok(Text::new());
        }
        text::trunc(&mut dir_path, slash - 1)?;
    }
    dir_path.push(b'/');
    dir_path.extend_from_slice(&rel_path);
    Ok(dir_path)
}
