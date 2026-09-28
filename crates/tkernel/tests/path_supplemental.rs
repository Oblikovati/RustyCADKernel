//! Explicit parser profiles and named environment/filesystem fixtures.
use rustycad_tkernel::osd::{
    CadPath, DirectorySource, File, FileStatusSource, PathSyntax, Process,
};
use std::{
    io,
    path::{Path, PathBuf},
};
struct FakeDirectorySource;
impl DirectorySource for FakeDirectorySource {
    fn current_directory(&self) -> io::Result<PathBuf> {
        Ok(PathBuf::from("/workspace/project"))
    }
}
struct FakeFileStatus;
impl FileStatusSource for FakeFileStatus {
    fn exists(&self, path: &Path) -> bool {
        path == Path::new("present.step")
    }
}
#[test]
fn injected_directory_and_file_sources_keep_queries_deterministic() {
    let mut path = Process::current_directory_with(&FakeDirectorySource).unwrap();
    assert_eq!(path.trek().as_bytes(), b"|workspace|project|");
    path.up_trek();
    assert_eq!(path.trek().as_bytes(), b"|workspace|");
    assert!(File::new("present.step").exists_with(&FakeFileStatus));
    assert!(!File::new("absent.step").exists_with(&FakeFileStatus));
    assert!(!File::new("").exists_with(&FakeFileStatus));
}
#[test]
fn syntax_profiles_preserve_unc_parent_markers_and_native_classification_quirks() {
    let mut windows = CadPath::with_syntax(r"C:\one\two\", PathSyntax::Windows);
    assert_eq!(windows.trek().as_bytes(), b"|one|two|");
    windows.up_trek();
    assert_eq!(windows.system_name().as_bytes(), b"C:/one/");
    let unc = CadPath::with_syntax(r"\\Server\Share\file.ext", PathSyntax::Windows);
    assert_eq!(unc.trek().as_bytes(), b"||Server|Share|");
    let unix = CadPath::with_syntax("host:/one/../file.ext", PathSyntax::Unix);
    assert_eq!(unix.system_name().as_bytes(), b"host:/one/../file.ext");
    assert!(CadPath::is_absolute_path("C:relative"));
    assert!(!CadPath::is_remote_protocol_path("://host"));
    assert!(!CadPath::is_remote_protocol_path("x:y://host"));
    assert!(CadPath::is_relative_path(""));
    assert!(CadPath::default().system_name().is_empty());
}
