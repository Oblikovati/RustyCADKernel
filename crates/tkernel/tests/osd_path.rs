// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Legacy path predicates, splitting and portable trek regressions.
#![allow(non_snake_case)]
use rustycad_tkernel::osd::{CadPath, File, Process};

#[test]
fn OSD_PathTest__DosAbsolutePaths() {
    assert!(CadPath::is_absolute_path("c:\\folder\\file.png"));
    assert!(CadPath::is_dos_path("c:\\folder\\file.png"));
    assert!(!CadPath::is_relative_path("c:\\folder\\file.png"));
    assert!(!CadPath::is_unix_path("c:\\folder\\file.png"));
    assert!(!CadPath::is_unc_path("c:\\folder\\file.png"));
    assert!(CadPath::is_absolute_path("D:\\"));
    assert!(CadPath::is_dos_path("D:\\"));
    assert!(CadPath::is_absolute_path("c:\\file.png"));
    assert!(CadPath::is_dos_path("c:\\file.png"));
}

#[test]
fn OSD_PathTest__UncPaths() {
    assert!(CadPath::is_absolute_path("\\\\share\\file.pdf"));
    assert!(CadPath::is_unc_path("\\\\share\\file.pdf"));
    assert!(!CadPath::is_dos_path("\\\\share\\file.pdf"));
    assert!(!CadPath::is_unix_path("\\\\share\\file.pdf"));
}

#[test]
fn OSD_PathTest__FClassesBug_23852_UncPathComponents() {
    let input = if cfg!(windows) {
        r"\\Server\Shared\Folder\File.Extension"
    } else {
        "//Server/Shared/Folder/File.Extension"
    };
    let path = CadPath::new(input);
    assert_eq!(path.trek().as_bytes(), b"||Server|Shared|Folder|");
    assert_eq!(path.name().as_bytes(), b"File");
    assert_eq!(path.extension().as_bytes(), b".Extension");
    #[cfg(windows)]
    {
        let slash = CadPath::new("//Server/Shared/Folder/File.Extension");
        assert_eq!(slash.trek().as_bytes(), b"||Server|Shared|Folder|");
        assert_eq!(slash.name().as_bytes(), b"File");
        assert_eq!(slash.extension().as_bytes(), b".Extension");
    }
}

#[test]
fn OSD_PathTest__NtExtendedPaths() {
    assert!(CadPath::is_absolute_path("\\\\?\\C:\\documents\\file.docx"));
    assert!(CadPath::is_nt_extended_path(
        "\\\\?\\C:\\documents\\file.docx"
    ));
    assert!(!CadPath::is_unc_path("\\\\?\\C:\\documents\\file.docx"));
}

#[test]
fn OSD_PathTest__UncExtendedPaths() {
    assert!(CadPath::is_absolute_path(
        "\\\\?\\UNC\\server\\share\\file.zip"
    ));
    assert!(CadPath::is_unc_path("\\\\?\\UNC\\server\\share\\file.zip"));
    assert!(CadPath::is_nt_extended_path(
        "\\\\?\\UNC\\server\\share\\file.zip"
    ));
    assert!(CadPath::is_unc_extended_path(
        "\\\\?\\UNC\\server\\share\\file.zip"
    ));
}

#[test]
fn OSD_PathTest__RemoteProtocolPaths() {
    assert!(CadPath::is_absolute_path("https://www.server.org/file.gif"));
    assert!(CadPath::is_remote_protocol_path(
        "https://www.server.org/file.gif"
    ));
    assert!(!CadPath::is_unix_path("https://www.server.org/file.gif"));
    assert!(!CadPath::is_dos_path("https://www.server.org/file.gif"));
    assert!(CadPath::is_absolute_path("ftp://ftp.server.com/file.dat"));
    assert!(CadPath::is_remote_protocol_path(
        "ftp://ftp.server.com/file.dat"
    ));
    assert!(CadPath::is_absolute_path("http://example.com/path"));
    assert!(CadPath::is_remote_protocol_path("http://example.com/path"));
}

#[test]
fn OSD_PathTest__ContentProtocolPaths() {
    assert!(CadPath::is_absolute_path("content://file.jpg"));
    assert!(CadPath::is_remote_protocol_path("content://file.jpg"));
    assert!(CadPath::is_content_protocol_path("content://file.jpg"));
}

#[test]
fn OSD_PathTest__UnixAbsolutePaths() {
    assert!(CadPath::is_absolute_path("/home/username/file.txt"));
    assert!(CadPath::is_unix_path("/home/username/file.txt"));
    assert!(!CadPath::is_relative_path("/home/username/file.txt"));
    assert!(!CadPath::is_dos_path("/home/username/file.txt"));
    assert!(CadPath::is_absolute_path("/"));
    assert!(CadPath::is_unix_path("/"));
    assert!(CadPath::is_absolute_path("/boot.bin"));
    assert!(CadPath::is_unix_path("/boot.bin"));
}

#[test]
fn OSD_PathTest__RelativePaths() {
    assert!(CadPath::is_relative_path("./subfolder/../file.txt"));
    assert!(!CadPath::is_absolute_path("./subfolder/../file.txt"));
    assert!(CadPath::is_relative_path("../file.txt"));
    assert!(!CadPath::is_absolute_path("../file.txt"));
    assert!(CadPath::is_relative_path("."));
    assert!(!CadPath::is_absolute_path("."));
    assert!(CadPath::is_relative_path(".."));
    assert!(!CadPath::is_absolute_path(".."));
    assert!(CadPath::is_relative_path("folder/file.txt"));
    assert!(!CadPath::is_absolute_path("folder/file.txt"));
    assert!(CadPath::is_relative_path("file.txt"));
    assert!(!CadPath::is_absolute_path("file.txt"));
}

#[test]
fn OSD_PathTest__FolderAndFileFromPath_UnixPaths() {
    let (folder, file) = CadPath::folder_and_file_from_path("/home/username/file.txt");
    assert_eq!(folder.as_bytes(), "/home/username/".as_bytes());
    assert_eq!(file.as_bytes(), "file.txt".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("/file.txt");
    assert_eq!(folder.as_bytes(), "/".as_bytes());
    assert_eq!(file.as_bytes(), "file.txt".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("/home/username/");
    assert_eq!(folder.as_bytes(), "/home/username/".as_bytes());
    assert_eq!(file.as_bytes(), "".as_bytes());
}

#[test]
fn OSD_PathTest__FolderAndFileFromPath_DosPaths() {
    let (folder, file) = CadPath::folder_and_file_from_path("C:\\Users\\John\\document.txt");
    assert_eq!(folder.as_bytes(), "C:\\Users\\John\\".as_bytes());
    assert_eq!(file.as_bytes(), "document.txt".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("C:\\file.txt");
    assert_eq!(folder.as_bytes(), "C:\\".as_bytes());
    assert_eq!(file.as_bytes(), "file.txt".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("C:\\Program Files\\");
    assert_eq!(folder.as_bytes(), "C:\\Program Files\\".as_bytes());
    assert_eq!(file.as_bytes(), "".as_bytes());
}

#[test]
fn OSD_PathTest__FolderAndFileFromPath_RelativePaths() {
    let (folder, file) = CadPath::folder_and_file_from_path("folder/subfolder/file.txt");
    assert_eq!(folder.as_bytes(), "folder/subfolder/".as_bytes());
    assert_eq!(file.as_bytes(), "file.txt".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("../folder/file.txt");
    assert_eq!(folder.as_bytes(), "../folder/".as_bytes());
    assert_eq!(file.as_bytes(), "file.txt".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("file.txt");
    assert_eq!(folder.as_bytes(), "".as_bytes());
    assert_eq!(file.as_bytes(), "file.txt".as_bytes());
}

#[test]
fn OSD_PathTest__FolderAndFileFromPath_ProtocolPaths() {
    let (folder, file) =
        CadPath::folder_and_file_from_path("https://www.server.org/folder/file.gif");
    assert_eq!(
        folder.as_bytes(),
        "https://www.server.org/folder/".as_bytes()
    );
    assert_eq!(file.as_bytes(), "file.gif".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("content://path/file.jpg");
    assert_eq!(folder.as_bytes(), "content://path/".as_bytes());
    assert_eq!(file.as_bytes(), "file.jpg".as_bytes());
}

#[test]
fn OSD_PathTest__EdgeCases() {
    let (folder, file) = CadPath::folder_and_file_from_path("");
    assert_eq!(folder.as_bytes(), "".as_bytes());
    assert_eq!(file.as_bytes(), "".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("/");
    assert_eq!(folder.as_bytes(), "/".as_bytes());
    assert_eq!(file.as_bytes(), "".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("\\");
    assert_eq!(folder.as_bytes(), "\\".as_bytes());
    assert_eq!(file.as_bytes(), "".as_bytes());
    let (folder, file) = CadPath::folder_and_file_from_path("/home/username/foldername");
    assert_eq!(folder.as_bytes(), "/home/username/".as_bytes());
    assert_eq!(file.as_bytes(), "foldername".as_bytes());
}

#[test]
fn OSD_PathTest__MixedSeparators() {
    let (folder, file) = CadPath::folder_and_file_from_path("C:/Users/John\\Documents/file.txt");
    assert!(!folder.is_empty() && !file.is_empty());
}

#[test]
fn OSD_PathTest__OCC310_TrekAndUpTrek() {
    let mut path = CadPath::new("/where/you/want/tmp/qwerty/tmp/");
    assert_eq!(path.trek().as_bytes(), b"|where|you|want|tmp|qwerty|tmp|");
    path.up_trek();
    assert_eq!(path.trek().as_bytes(), b"|where|you|want|tmp|qwerty|");
}

#[test]
fn OSD_PathTest__OCC309_CurrentDirectoryAndUpTrek() {
    // This legacy regression intentionally exercises the actual process directory.
    let mut path = Process::new().current_directory();
    let first = path.system_name();
    assert!(!first.is_empty());
    path.up_trek();
    let second = path.system_name();
    assert!(!second.is_empty());
    assert_ne!(first, second);
    assert!(second.len() < first.len());
}

#[test]
fn OSD_PathTest__FClassesBuc_60944_DotDotPathSystemName() {
    let path = CadPath::new("../../..");
    assert_eq!(path.system_name().as_bytes(), b"../../..");
}

#[test]
fn OSD_FileTest__FClassesBug_710_EmptyFilePathDoesNotThrow() {
    let file = File::new("");
    let _exists = file.exists();
}
