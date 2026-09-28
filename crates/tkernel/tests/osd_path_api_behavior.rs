//! OSD_Path: every public member on both native builds, with the quirks each keeps.
//! The Unix-build expectations are also compared with OCCT by the `osd_path_*` probes.
use rustycad_tkernel::osd::{CadPath, PathSyntax, SysType};

fn unix(path: &str, sys_type: SysType) -> CadPath {
    CadPath::parse(path, sys_type, PathSyntax::Unix).unwrap()
}

fn bytes(path: &CadPath) -> Vec<Vec<u8>> {
    let values = path.values();
    [
        values.node,
        values.user_name,
        values.password,
        values.disk,
        values.trek,
        values.name,
        values.extension,
    ]
    .iter()
    .map(|value| value.as_bytes().to_vec())
    .collect()
}

#[test]
fn unix_build_parses_each_dialect() {
    let path = unix("host:/home/user/f.txt", SysType::LinuxRedHat);
    assert_eq!(bytes(&path)[0], b"host");
    assert_eq!(path.trek().as_bytes(), b"|home|user|");
    let vms = unix("node\"user pass\"::disk:[dir.sub]name.ext", SysType::Vms);
    assert_eq!(
        bytes(&vms),
        [
            &b"node"[..],
            b"user",
            b"pass",
            b"disk",
            b"|dir|sub",
            b"name",
            b".ext"
        ]
    );
    let dos = unix("C:\\..\\x\\f.e", SysType::WindowsNt);
    assert_eq!(dos.disk().as_bytes(), b"C:");
    assert_eq!(dos.trek().as_bytes(), b"|^|x|");
    let mac = unix("a::b:cde", SysType::MacOs);
    assert_eq!(
        mac.trek().as_bytes(),
        b"a^b|c",
        "native MacExtract drops the name's first byte"
    );
    assert_eq!(mac.name().as_bytes(), b"de");
    assert!(
        bytes(&unix("/x", SysType::Taligent))
            .iter()
            .all(Vec::is_empty)
    );
}

#[test]
fn unix_build_keeps_the_vms_name_quirks_and_failures() {
    let vms = unix("disk:name.e1.e2", SysType::Vms);
    assert_eq!(vms.name().as_bytes(), b"name.e2");
    assert_eq!(vms.extension().as_bytes(), b".e1");
    let error = CadPath::parse("\"user", SysType::Vms, PathSyntax::Unix).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(CadPath::parse("disk:[]", SysType::Vms, PathSyntax::Unix).is_err());
    assert_eq!(unix("disk:[]x", SysType::Vms).trek().as_bytes(), b"|x");
}

#[test]
fn unix_build_spells_every_dialect() {
    let path = CadPath::from_values("host", "user", "pw", "DISK$1", "a|^|b|", "n", ".e");
    assert_eq!(
        path.system_name().as_bytes(),
        b"user\"pw\"@host:/DISK1a/../b/n.e"
    );
    let vms = path.system_name_as(SysType::Vms).unwrap();
    assert_eq!(vms.as_bytes(), b"host\"user pw\"::DISK$1:[.a.-.b.]n.e");
    let dos = path.system_name_as(SysType::WindowsNt).unwrap();
    assert_eq!(dos.as_bytes(), b"\\DISK1\\a\\..\\b\\n.e");
    assert_eq!(
        path.system_name_as(SysType::MacOs).unwrap().as_bytes(),
        b"DISK$1:n.e"
    );
    let no_trek = CadPath::from_values("", "", "", "DATA", "", "n", ".e");
    let error = no_trek.system_name_as(SysType::Os2).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert_eq!(
        no_trek.system_name_as(SysType::Vms).unwrap().as_bytes(),
        b"DATA:"
    );
}

#[test]
fn unix_build_edits_treks_by_token_text() {
    let mut path = CadPath::from_values("", "", "", "", "|ab|b|", "n", ".e");
    assert_eq!(path.trek_length(), 2);
    assert_eq!(path.trek_value(2).unwrap().as_bytes(), b"b");
    path.remove_a_trek(2).unwrap();
    assert_eq!(
        path.trek().as_bytes(),
        b"|a|b|",
        "native removes the first text match"
    );
    path.insert_a_trek("new", 2).unwrap();
    assert_eq!(path.trek().as_bytes(), b"|a|new|b|");
    path.remove_a_trek_named("new").unwrap();
    assert_eq!(path.trek().as_bytes(), b"|a|b|");
    path.down_trek("sub");
    assert_eq!(path.trek().as_bytes(), b"|a|b|sub|");
    path.up_trek();
    assert_eq!(path.trek().as_bytes(), b"|a|b|");
    let error = path.remove_a_trek(9).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NumericError");
    assert_eq!(
        error.message(),
        "OSD_Path::RemoveATrek : where has an invalid value"
    );
    assert!(path.trek_value(3).is_err());
    assert!(path.insert_a_trek("x", 0).is_err());
}

#[test]
fn component_setters_store_and_expanded_name_is_a_no_op() {
    let mut path = CadPath::empty(PathSyntax::Unix);
    path.set_node("n");
    path.set_user_name("u");
    path.set_password("p");
    path.set_disk("d");
    path.set_trek("|t|");
    path.set_name("f");
    path.set_extension("e");
    assert_eq!(
        bytes(&path),
        [&b"n"[..], b"u", b"p", b"d", b"|t|", b"f", b"e"]
    );
    let mut expanded = rustycad_tkernel::tcollection::AsciiString::new("kept");
    path.expanded_name(&mut expanded);
    assert_eq!(expanded.as_bytes(), b"kept");
    assert_eq!(path.syntax(), PathSyntax::Unix);
}

#[test]
fn windows_build_splits_and_normalizes_treks() {
    let path = CadPath::with_syntax("C:\\a\\\\b\\..\\..\\f.x.y", PathSyntax::Windows);
    assert_eq!(bytes(&path)[3..], [&b"C:"[..], b"|a|b|^|^|", b"f.x", b".y"]);
    assert_eq!(path.system_name().as_bytes(), b"C:/a/b/../../f.x.y");
    let parents = CadPath::with_syntax(r"..\..\..\f", PathSyntax::Windows);
    assert_eq!(
        parents.trek().as_bytes(),
        b"^|^|^|",
        "only adjacent markers collapse"
    );
    let error = CadPath::parse("a", SysType::Vms, PathSyntax::Windows).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    assert_eq!(
        error.message(),
        "OSD_Path :: OSD_Path (): unknown system type"
    );
    let message = path.system_name_as(SysType::MacOs).unwrap_err();
    assert_eq!(
        message.message(),
        "OSD_Path :: SystemName (): unknown system type"
    );
    let mut values = CadPath::empty(PathSyntax::Windows);
    values.set_values("", "", "", "", "|x||y|", "n", "e");
    assert_eq!(values.extension().as_bytes(), b".e");
    assert_eq!(values.trek().as_bytes(), b"|x|y|");
}

#[test]
fn windows_build_edits_treks_by_bar_position() {
    let mut path = CadPath::with_syntax("\\a\\b\\c\\f", PathSyntax::Windows);
    assert_eq!(path.trek_length(), 3);
    path.remove_a_trek(2).unwrap();
    assert_eq!(path.trek().as_bytes(), b"|a|c|");
    path.insert_a_trek("new", 1).unwrap();
    assert_eq!(path.trek().as_bytes(), b"|new|a|c|");
    path.remove_a_trek_named("a").unwrap();
    assert_eq!(path.trek().as_bytes(), b"|new|c|");
    assert_eq!(path.trek_value(2).unwrap().as_bytes(), b"c");
    assert!(path.trek_value(7).unwrap().is_empty());
    path.down_trek("d");
    assert_eq!(path.trek().as_bytes(), b"|new|c|d|");
    path.up_trek();
    assert_eq!(path.trek().as_bytes(), b"|new|c|");
    path.remove_a_trek(9).unwrap();
    let mut empty = CadPath::empty(PathSyntax::Windows);
    assert_eq!(
        empty.trek_value(1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(empty.remove_a_trek_named("x").is_err());
}

#[test]
fn relative_and_absolute_paths_follow_the_native_token_walk() {
    let relative = CadPath::relative_path("/a/b", "/a/c/f").unwrap();
    assert_eq!(relative.as_bytes(), b"../c/f");
    assert!(CadPath::relative_path("/a", "/a").unwrap().is_empty());
    assert!(CadPath::relative_path("", "/a").is_err());
    let drive = CadPath::relative_path("C:\\a\\b", "c:\\A\\B\\f").unwrap();
    assert_eq!(drive.as_bytes(), b"f", "drive tokens compare without case");
    assert_eq!(
        CadPath::absolute_path("/a/b", "../c").unwrap().as_bytes(),
        b"/a/c"
    );
    assert_eq!(
        CadPath::absolute_path("/a/b", "C:\\x").unwrap().as_bytes(),
        b"C:\\x"
    );
    assert!(CadPath::absolute_path("rel", "x").unwrap().is_empty());
    assert!(CadPath::absolute_path("/a", "").is_err());
    let unc = CadPath::absolute_path_on("//srv//a", "b", PathSyntax::Windows).unwrap();
    assert_eq!(unc.as_bytes(), b"//srv/a/b");
    let unix = CadPath::absolute_path_on("//srv//a", "b", PathSyntax::Unix).unwrap();
    assert_eq!(unix.as_bytes(), b"/srv/a/b");
    let windows = CadPath::relative_path_on("/x/y", "/x/z", PathSyntax::Windows).unwrap();
    assert_eq!(windows.as_bytes(), b"../z");
}

#[test]
fn name_and_extension_splitters_and_validity() {
    let (name, extension) = CadPath::file_name_and_extension("dir/file.STEP");
    assert_eq!(
        (name.as_bytes(), extension.as_bytes()),
        (&b"dir/file"[..], &b"step"[..])
    );
    let (name, extension) = CadPath::file_name_and_extension(".hidden");
    assert_eq!(
        (name.as_bytes(), extension.as_bytes()),
        (&b".hidden"[..], &b""[..])
    );
    let (name, _) = CadPath::file_name_and_extension("n.ABCDEFGHIJKLMNOPQRST");
    assert_eq!(
        name.as_bytes(),
        b"n.ABCDEFGHIJKLMNOPQRST",
        "extensions stop at 19 bytes"
    );
    let (folder, file) = CadPath::folder_and_file_from_path("a\\b/c");
    assert_eq!(
        (folder.as_bytes(), file.as_bytes()),
        (&b"a\\b/"[..], &b"c"[..])
    );
    assert!(CadPath::is_valid("", SysType::Vms));
    assert!(!CadPath::is_valid("a/b", SysType::Vms));
    assert!(!CadPath::is_valid("a*b", SysType::WindowsNt));
    assert!(CadPath::is_valid(
        "a:12345678901234567890123456789012",
        SysType::MacOs
    ));
    assert!(!CadPath::is_valid(
        "12345678901234567890123456789012",
        SysType::MacOs
    ));
    assert!(!CadPath::is_valid_on(
        "a:b",
        SysType::Default,
        PathSyntax::Windows
    ));
    assert!(CadPath::is_valid_on(
        "a:b",
        SysType::Default,
        PathSyntax::Unix
    ));
    assert_eq!(PathSyntax::Windows.host_sys_type(), SysType::WindowsNt);
}
