//! Parser and persistence edge cases from Resource_Manager.cxx.
use rustycad_tkernel::resource::ResourceManager;
use std::{path::PathBuf, rc::Rc};
#[path = "resource_manager/support.rs"]
mod support;
use support::FakeResourceSystem;
fn fixture(bytes: &[u8]) -> (Rc<FakeResourceSystem>, ResourceManager) {
    let system: Rc<FakeResourceSystem> = Rc::default();
    system.set_environment("CSF_AppDefaults", PathBuf::from("defaults"));
    system
        .files
        .borrow_mut()
        .insert(PathBuf::from("defaults/App"), bytes.to_vec());
    let catalog = ResourceManager::new("App", false, system.host()).unwrap();
    (system, catalog)
}
#[test]
fn parser_preserves_native_whitespace_escape_and_duplicate_rules() {
    let (_,catalog)=fixture(b"!ignored: yes\n  !key : kept\n bad line\n:invalid\n x : old\nx: new \r\nspace: \\ leading\ntab:\\\tleading\nslash:\\\\path\nother:\\n\nempty: \t\nlast: eof");
    for (key, value) in [
        ("  !key", "absent"),
        ("!key", "kept"),
        ("x", "new "),
        ("space", " leading"),
        ("tab", "\tleading"),
        ("slash", "\\path"),
        ("other", "\\n"),
        ("empty", ""),
        ("last", "eof"),
    ] {
        if value == "absent" {
            assert!(!catalog.find(key));
        } else {
            assert_eq!(catalog.value(key).unwrap().as_bytes(), value.as_bytes());
        }
    }
    assert!(!catalog.find("!ignored"));
    assert!(catalog.is_initialized());
}
#[test]
fn user_defaults_override_reference_but_save_only_explicit_overrides() {
    let (system, mut catalog) = fixture(b"key:default\n");
    system.write_resource(&PathBuf::from("user"), "App", "key", "user");
    system.set_environment("CSF_AppUserDefaults", PathBuf::from("user"));
    assert!(catalog.save());
    assert!(system.files.borrow()[&PathBuf::from("user/App")].is_empty());
    system.write_resource(&PathBuf::from("user"), "App", "key", "user");
    catalog = ResourceManager::new("App", false, system.host()).unwrap();
    assert_eq!(catalog.value("key").unwrap().as_bytes(), b"user");
    for (key, value) in [
        ("z", "\\path"),
        ("a", " leading"),
        ("m", "\ttab"),
        ("empty", ""),
    ] {
        catalog.set_resource(key, value.as_bytes());
    }
    system.set_environment("CSF_AppUserDefaults", PathBuf::from("new/nested"));
    assert!(catalog.save());
    assert_eq!(
        system.files.borrow()[&PathBuf::from("new/nested/App")],
        b"a:\t\\ leading\nempty:\t\nm:\t\\\ttab\nz:\t\\\\path\n"
    );
    assert!(system.has_file(&PathBuf::from("new/nested/App")));
}
#[test]
fn numeric_conversion_missing_resources_and_missing_save_path() {
    let (_, mut catalog) = fixture(b"int: -42extra\nreal: 1.25more\nbad: nothing\n");
    assert_eq!(catalog.integer("int").unwrap(), -42);
    assert_eq!(catalog.real("real").unwrap(), 1.25);
    assert_eq!(
        catalog.integer("bad").unwrap_err().exception_type(),
        "Standard_TypeMismatch"
    );
    assert_eq!(
        catalog.real("bad").unwrap_err().exception_type(),
        "Standard_TypeMismatch"
    );
    assert_eq!(
        catalog.value("missing").unwrap_err().exception_type(),
        "Resource_NoSuchResource"
    );
    assert!(!catalog.save());
    catalog.set_resource_integer("int", 123);
    assert_eq!(catalog.integer("int").unwrap(), 123);
    let missing = Rc::new(FakeResourceSystem::default());
    assert!(
        !ResourceManager::new("missing", false, missing.host())
            .unwrap()
            .is_initialized()
    );
}
