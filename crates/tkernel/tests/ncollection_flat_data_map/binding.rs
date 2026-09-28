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

#[test]
fn NCollection_FlatDataMapTest__TryBindNewKey() {
    let mut map = StringMap::default();
    assert!(map.try_bind(1, AsciiString::new("One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__TryBindExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert!(!map.try_bind(1, AsciiString::new("New One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__TryBoundNewKey() {
    let mut map = StringMap::default();
    assert_eq!(
        map.try_bound(1, AsciiString::new("One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__TryBoundExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert_eq!(
        map.try_bound(1, AsciiString::new("New One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__BoundNewKey() {
    let mut map = StringMap::default();
    assert_eq!(
        map.bound(1, AsciiString::new("One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__BoundExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert_eq!(
        map.bound(1, AsciiString::new("New One")),
        &AsciiString::new("New One")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__EmplaceNewKey() {
    let mut map = StringMap::default();
    assert!(map.emplace(1, || AsciiString::new("One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__EmplaceExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert!(!map.emplace(1, || AsciiString::new("New One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("New One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__EmplacedNewKey() {
    let mut map = StringMap::default();
    assert_eq!(
        map.emplaced(1, || AsciiString::new("One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__EmplacedExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert_eq!(
        map.emplaced(1, || AsciiString::new("New One")),
        &AsciiString::new("New One")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__TryEmplaceNewKey() {
    let mut map = StringMap::default();
    assert!(map.try_emplace(1, || AsciiString::new("One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__TryEmplaceExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert!(!map.try_emplace(1, || AsciiString::new("New One")));
    assert_eq!(map.find(&1), Ok(&AsciiString::new("One")));
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__TryEmplacedNewKey() {
    let mut map = StringMap::default();
    assert_eq!(
        map.try_emplaced(1, || AsciiString::new("One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.len(), 1);
}

#[test]
fn NCollection_FlatDataMapTest__TryEmplacedExistingKey() {
    let mut map = StringMap::default();
    map.bind(1, AsciiString::new("One"));
    assert_eq!(
        map.try_emplaced(1, || AsciiString::new("New One")),
        &AsciiString::new("One")
    );
    assert_eq!(map.len(), 1);
}
