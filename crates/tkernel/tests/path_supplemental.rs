//! Explicit parser profiles and named environment/filesystem fixtures.
use rustycad_tkernel::osd::{CadPath, LocalTime, PathSyntax, Process, ProcessSource};
const WINDOWS_CURRENT_TREK: &[u8] = b"|workspace|";
const WINDOWS_PARENT_TREK: &[u8] = b"|";
struct FakeDirectorySource;
impl ProcessSource for FakeDirectorySource {
    fn process_id(&self) -> i32 {
        1
    }
    fn is_super_user(&self) -> bool {
        false
    }
    fn user_name(&self) -> Result<Vec<u8>, i32> {
        Ok(Vec::new())
    }
    fn local_time(&self) -> Result<LocalTime, i32> {
        Ok(LocalTime::default())
    }
    fn current_directory(&self) -> Result<Vec<u8>, i32> {
        Ok(b"/workspace/project".to_vec())
    }
    fn set_current_directory(&self, _path: &[u8]) -> Result<(), i32> {
        Ok(())
    }
    fn executable_path(&self) -> Vec<u8> {
        Vec::new()
    }
}
#[test]
fn injected_directory_source_keeps_queries_deterministic() {
    let mut path = Process::with_source(FakeDirectorySource).current_directory();
    // Windows CurrentDirectory keeps the final component as Name, as OCCT does.
    let expected: &[u8] = if cfg!(windows) {
        WINDOWS_CURRENT_TREK
    } else {
        b"|workspace|project|"
    };
    assert_eq!(path.trek().as_bytes(), expected);
    path.up_trek();
    let parent: &[u8] = if cfg!(windows) {
        WINDOWS_PARENT_TREK
    } else {
        b"|workspace|"
    };
    assert_eq!(path.trek().as_bytes(), parent);
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

#[test]
fn windows_directory_components_match_the_process_source_contract() {
    let mut path = CadPath::with_syntax("/workspace/project", PathSyntax::Windows);
    assert_eq!(path.trek().as_bytes(), WINDOWS_CURRENT_TREK);
    assert_eq!(path.name().as_bytes(), b"project");
    path.up_trek();
    assert_eq!(path.trek().as_bytes(), WINDOWS_PARENT_TREK);
}
