//! 1:1 translation of src/ApplicationFramework/TKCDF/GTests/CDF_FWOSDriver_Test.cxx.
#![allow(non_snake_case)]
use rustycad_tkcdf::cdf::{CdfFwosDriver, CdfMetaDataDriver};
use rustycad_tkernel::{osd::CadPath, standard::SharedState, tcollection::ExtendedString};
use std::sync::Arc;

/// The native `__FILE__`: this source file's absolute path.
const THIS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cdf_fwos_driver.rs");

fn folder_and_file() -> (ExtendedString, ExtendedString) {
    let (folder, file) = CadPath::folder_and_file_from_path(THIS_FILE);
    (
        ExtendedString::from_ascii_string(&folder, true),
        ExtendedString::from_ascii_string(&file, true),
    )
}

#[test]
fn CDF_FWOSDriverTest__ExistingReadableFile() {
    let (folder, file_name) = folder_and_file();
    assert!(!folder.is_empty());
    assert!(!file_name.is_empty());
    let driver = CdfFwosDriver::new(&Arc::new(SharedState::default()));
    assert!(driver.find_folder(&folder));
    assert!(driver.find(&folder, &file_name, &ExtendedString::new("")));
    assert!(driver.has_read_permission(&folder, &file_name, &ExtendedString::new("")));
}

#[test]
fn CDF_FWOSDriverTest__MissingFileIsNotReadable() {
    let (folder, _) = folder_and_file();
    assert!(!folder.is_empty());
    let driver = CdfFwosDriver::new(&Arc::new(SharedState::default()));
    let missing = ExtendedString::new("CDF_FWOSDriver_missing_test_file");
    assert!(!driver.find(&folder, &missing, &ExtendedString::new("")));
    assert!(!driver.has_read_permission(&folder, &missing, &ExtendedString::new("")));
}
