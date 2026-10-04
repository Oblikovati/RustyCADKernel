//! Resource_Manager constructors, traces, lookups, assignments and paths.
use super::support::FakeResourceSystem;
use rustycad_tkernel::{resource::ResourceManager, tcollection::AsciiString};
use std::{path::PathBuf, sync::Arc};

fn system_with(name: &str, content: &[u8]) -> Arc<FakeResourceSystem> {
    let system: Arc<FakeResourceSystem> = Arc::default();
    system.set_environment(&format!("CSF_{name}Defaults"), PathBuf::from("defaults"));
    let path = PathBuf::from(format!("defaults/{name}"));
    system.files.lock().unwrap().insert(path, content.to_vec());
    system
}
fn console(system: &FakeResourceSystem) -> String {
    String::from_utf8(system.console.lock().unwrap().clone()).unwrap()
}

#[test]
fn verbose_named_constructor_reports_loads_and_missing_variables() {
    let _serial = super::serial();
    let system = system_with("App", b"a: 1\nno colon\n");
    let manager = ResourceManager::new("App", true, system.host()).unwrap();
    assert!(manager.is_initialized());
    assert_eq!(
        console(&system),
        "Resource Manager: Syntax error at line 2 in file : App\n\
         Resource Manager: Reference file \"App\" loaded\n\
         Resource Manager Warning: Environment variable \"CSF_AppUserDefaults\" not set.\n"
    );
}
#[test]
fn resource_verbose_variable_forces_verbose_and_missing_file_is_reported() {
    let _serial = super::serial();
    let system: Arc<FakeResourceSystem> = Arc::default();
    system.set_environment("CSF_ResourceVerbose", PathBuf::from("1"));
    system.set_environment("CSF_AppDefaults", PathBuf::from("nowhere"));
    let manager = ResourceManager::new("App", false, system.host()).unwrap();
    assert!(!manager.is_initialized());
    assert!(console(&system).starts_with(
        "Resource Manager Warning: Cannot read file \"App\". File not found or permission denied.\n"
    ));
}
#[test]
fn directory_constructor_loads_both_files_into_the_reference_map() {
    let _serial = super::serial();
    let system = system_with("App", b"a: ref\nb: ref\n");
    system.write_resource(&PathBuf::from("user"), "App", "b", "user");
    let mut manager =
        ResourceManager::with_directories("App", "defaults", "user/", false, system.host())
            .unwrap();
    assert_eq!(manager.value("b").unwrap().as_bytes(), b"user");
    assert_eq!(manager.get_map(true).len(), 2);
    assert!(manager.get_map(false).is_empty());
    let empty = ResourceManager::with_directories("App", "", "", true, system.host()).unwrap();
    assert!(!empty.is_initialized());
    assert!(console(&system).ends_with(
        "aDefaultsDirectory is empty.\nResource Manager Warning: anUserDefaultsDirectory is empty.\n"
    ));
}
#[test]
fn resource_debug_traces_every_key_value_and_saved_line() {
    let _serial = super::serial();
    let system = system_with("App", b"k : v\n");
    system.set_environment("ResourceDebug", PathBuf::from("1"));
    let mut manager = ResourceManager::new("App", false, system.host()).unwrap();
    manager.set_resource("s", b"x");
    system.set_environment("CSF_AppUserDefaults", PathBuf::from("out"));
    assert!(manager.save());
    assert_eq!(
        console(&system),
        "Key = 'k'\t Value = 'v'\nLine = 's:\tx\n'\n"
    );
    // The flag is process-wide and re-read by every name-based constructor.
    let quiet = system_with("Other", b"q: 1\n");
    let _ = ResourceManager::new("Other", false, quiet.host()).unwrap();
    assert!(console(&quiet).is_empty());
}
#[test]
fn fgets_chunks_hide_bytes_after_nul_and_reject_a_leading_nul() {
    let _serial = super::serial();
    let system = system_with(
        "App",
        b"a\0x\nnext: line\nlong.key.name : long value text\n",
    );
    let manager = ResourceManager::new("App", false, system.host()).unwrap();
    assert!(!manager.find("next"));
    assert_eq!(manager.value("anext").unwrap().as_bytes(), b"line");
    assert_eq!(
        manager.value("long.key.name").unwrap().as_bytes(),
        b"long value text"
    );
    let failing = system_with("Bad", b"\0start\n");
    let error = ResourceManager::new("Bad", false, failing.host())
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}
#[test]
fn empty_manager_lookups_assignments_and_maps() {
    let _serial = super::serial();
    let system: Arc<FakeResourceSystem> = Arc::default();
    let mut manager = ResourceManager::empty(system.host());
    assert!(!manager.is_initialized());
    manager.set_resource("text", b"abc\0ignored");
    manager.set_resource_integer("int", -7);
    manager.set_resource_real("real", 0.1);
    manager.set_resource_real("big", 1e300);
    assert_eq!(manager.value("text").unwrap().as_bytes(), b"abc");
    assert_eq!(manager.value("int").unwrap().as_bytes(), b"-7");
    assert_eq!(manager.value("real").unwrap().as_bytes(), b"0.1");
    assert_eq!(manager.value("big").unwrap().as_bytes(), b"1e+300");
    let found = manager.find_value(&AsciiString::new("int")).unwrap();
    assert_eq!(found.as_bytes(), b"-7");
    assert!(manager.find_value(&AsciiString::new("none")).is_none());
    manager
        .get_map(true)
        .bind(AsciiString::new("ref"), AsciiString::new("r"));
    assert!(manager.find("ref"));
    assert_eq!(manager.get_map(false).len(), 4);
    let missing = manager.value("none").unwrap_err();
    assert_eq!(missing.to_string(), "Resource_NoSuchResource: none");
}
#[test]
fn save_without_variable_or_entries_follows_native_messages() {
    let _serial = super::serial();
    let system: Arc<FakeResourceSystem> = Arc::default();
    let manager = ResourceManager::new("App", true, system.host()).unwrap();
    system.console.lock().unwrap().clear();
    assert!(!manager.save());
    assert_eq!(
        console(&system),
        "Resource Manager Warning: environment variable \"CSF_AppUserDefaults\" not set.  Cannot save resources.\n"
    );
    system.set_environment("CSF_AppUserDefaults", PathBuf::from("out"));
    system.console.lock().unwrap().clear();
    assert!(manager.save());
    assert!(system.has_file(&PathBuf::from("out/App")));
    assert!(system.files.lock().unwrap()[&PathBuf::from("out/App")].is_empty());
    assert!(console(&system).is_empty());
}
#[test]
fn resource_path_follows_osd_path_rules() {
    let _serial = super::serial();
    let system: Arc<FakeResourceSystem> = Arc::default();
    let path = |directory: &str, user: bool| {
        let variable = if user {
            "CSF_AppUserDefaults"
        } else {
            "CSF_AppDefaults"
        };
        system.set_environment(variable, PathBuf::from(directory));
        let path = ResourceManager::resource_path_with("App", user, system.as_ref());
        String::from_utf8(path.as_bytes().to_vec()).unwrap()
    };
    assert_eq!(path("", false), "");
    assert_eq!(path("a/b", false), "a/b/App");
    assert_eq!(path("a/b/", true), "a/b/App");
    assert_eq!(path(".hidden", false), "App");
    assert_eq!(path("a/.hidden", true), "a/App");
    assert_eq!(path("a/b.c", false), "a/b.c/App");
    let native = ResourceManager::resource_path("RustyCadUnsetResourceName", false);
    assert!(native.is_empty());
}
