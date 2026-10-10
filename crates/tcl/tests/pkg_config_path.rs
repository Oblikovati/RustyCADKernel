//! The build script must preserve both drive letters and existing package directories.
#[path = "../build/pkg_config_path.rs"]
mod pkg_config_path;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[test]
fn shim_precedes_existing_paths_with_spaces() {
    let prefix = if cfg!(windows) {
        "C:/native"
    } else {
        "/native"
    };
    let shim = Path::new(prefix).join("shim directory");
    let paths = [PathBuf::from("first directory"), PathBuf::from("second")];
    let current = std::env::join_paths(&paths).unwrap();
    let joined = pkg_config_path::prepend(&shim, &current).unwrap();
    assert_eq!(
        std::env::split_paths(&joined).collect::<Vec<_>>(),
        [vec![shim], paths.to_vec()].concat()
    );
}

#[test]
fn empty_search_path_keeps_the_shim() {
    let joined = pkg_config_path::prepend(Path::new("shim"), OsStr::new("")).unwrap();
    assert_eq!(
        std::env::split_paths(&joined).collect::<Vec<_>>(),
        vec![PathBuf::from("shim")]
    );
}

#[test]
fn invalid_path_separator_is_rejected() {
    let separator = if cfg!(windows) { ";" } else { ":" };
    let directory = format!("invalid{separator}directory");
    assert!(pkg_config_path::prepend(Path::new(&directory), OsStr::new("")).is_err());
}
