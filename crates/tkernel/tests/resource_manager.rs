// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Complete Resource_Manager regressions using isolated filesystem/environment fixtures.
#![allow(non_snake_case)]
use rustycad_tkernel::resource::ResourceCatalog;
use std::{path::PathBuf, rc::Rc};
#[path = "resource_manager/support.rs"]
mod support;
use support::FakeResourceSystem;
#[test]
fn Resource_ManagerTest__OCC27849_PathsWithSpecialChars() {
    let system: Rc<FakeResourceSystem> = Rc::default();
    for relative in [
        "path",
        "path.with.dots",
        "path with spaces",
        "nested/dirs/path with spaces",
    ] {
        let directory: PathBuf = PathBuf::from("occt_resource_manager_test").join(relative);
        system.write_resource(&directory, "TestResource", "test.resource", "ok");
        system.set_environment("CSF_TestResourceDefaults", directory);
        let resources = ResourceCatalog::new("TestResource", system.clone(), system.clone());
        assert!(resources.find("test.resource"));
        if resources.find("test.resource") {
            assert_eq!(resources.value("test.resource").unwrap().as_bytes(), b"ok");
        }
    }
}
#[test]
fn Resource_ManagerTest__OCC181_SaveToExistingDirectory() {
    let system: Rc<FakeResourceSystem> = Rc::default();
    let source: PathBuf = PathBuf::from("occt_resource_manager_test/source");
    let save: PathBuf = PathBuf::from("occt_resource_manager_test/save_flat");
    system.write_resource(&source, "OCC181", "test.key", "test_value");
    system.add_directory(save.clone());
    system.set_environment("CSF_OCC181UserDefaults", source);
    let resources = ResourceCatalog::new("OCC181", system.clone(), system.clone());
    system.set_environment("CSF_OCC181UserDefaults", save.clone());
    assert!(resources.save());
    assert!(system.has_file(&save.join("OCC181")));
}
#[test]
fn Resource_ManagerTest__OCC181_SaveToNestedNonExistentDirectory() {
    let system: Rc<FakeResourceSystem> = Rc::default();
    let source: PathBuf = PathBuf::from("occt_resource_manager_test/source2");
    let save: PathBuf = PathBuf::from("occt_resource_manager_test/nested/deep/dir");
    system.write_resource(&source, "OCC181", "test.key", "test_value");
    system.set_environment("CSF_OCC181UserDefaults", source);
    let resources = ResourceCatalog::new("OCC181", system.clone(), system.clone());
    system.set_environment("CSF_OCC181UserDefaults", save.clone());
    assert!(resources.save());
    assert!(system.has_file(&save.join("OCC181")));
}
