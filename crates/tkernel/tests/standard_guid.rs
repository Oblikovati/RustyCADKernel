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

//! Translated from TKernel/GTests/Standard_GUID_Test.cxx.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Guid;

#[test]
fn Standard_GUID_Test__OCC669_ValidGUIDConstruction() {
    assert!(
        "0000000A-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000B-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000C-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000D-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000E-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000F-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000a-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000b-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000c-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000d-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000e-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "0000000f-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-000A-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-000F-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-000a-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-000f-0000-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-000A-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-000F-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-000a-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-000f-0000-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-0000-000A-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-0000-000F-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-0000-000a-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-0000-000f-000000000000"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-0000-0000-00000000000A"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-0000-0000-00000000000F"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-0000-0000-00000000000a"
            .parse::<Guid>()
            .is_ok()
    );
    assert!(
        "00000000-0000-0000-0000-00000000000f"
            .parse::<Guid>()
            .is_ok()
    );
}

#[test]
fn Standard_GUID_Test__OCC669_InvalidGUIDConstruction() {
    assert!(
        "0000000G-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "0000000g-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-000G-0000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-000g-0000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-000G-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-000g-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-0000-000G-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-0000-000g-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-0000-0000-00000000000G"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-0000-0000-00000000000g"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "000000000000000000000000000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "0000000000000000000000000000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000000000000000000000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "0000000000000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-000000000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-000000000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-0000-00000000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "000000000-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "0000000-0000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-00000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-000-0000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-00000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-000-0000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-0000-00000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-0000-000-000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-0000-0000-0000000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!(
        "00000000-0000-0000-0000-00000000000"
            .parse::<Guid>()
            .is_err()
    );
    assert!("".parse::<Guid>().is_err());
}

#[test]
fn Standard_GUID_Test__OCC669_GUIDFormatting() {
    let guid1: Guid = "00000000-0000-0000-0000-000000000000".parse().unwrap();
    let guid2: Guid = "0000000A-000B-000C-000D-00000000000E".parse().unwrap();
    let guid3: Guid = "0000000a-000b-000c-000d-00000000000e".parse().unwrap();
    let guid4: Guid = "0000000A-000B-000C-000D-00000000000E".parse().unwrap();
    assert_eq!(guid2, guid4);
    assert_eq!(guid2, guid3);
    assert_ne!(guid1, guid2);
}

#[test]
fn Standard_GUID_Test__ConstexprConstructionAndComparison() {
    const ZERO: Guid = Guid::new();
    const _: () = assert!(ZERO.is_same(Guid::new()));
    const GUID1: Guid = Guid::from_components(
        0x12345678, 0x1234, 0x5678, 0x9ABC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78,
    );
    const GUID2: Guid = Guid::from_components(
        0x12345678, 0x1234, 0x5678, 0x9ABC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78,
    );
    const GUID3: Guid = Guid::from_components(
        0x12345678, 0x1234, 0x5678, 0x9ABC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x79,
    );
    // OCCT checks both operator== and IsSame in const contexts.
    const _: () = assert!(GUID1.is_same(GUID2));
    const _: () = assert!(GUID1.is_same(GUID2));
    // OCCT checks both operator!= and IsNotSame in const contexts.
    const _: () = assert!(GUID1.is_not_same(GUID3));
    const _: () = assert!(GUID1.is_not_same(GUID3));
    const UUID: rustycad_tkernel::standard::Uuid = GUID1.to_uuid();
    const _: () = assert!(UUID.data1 == 0x12345678);
    const _: () = assert!(UUID.data2 == 0x1234);
    const _: () = assert!(UUID.data3 == 0x5678);
    const FROM_UUID: Guid = Guid::from_uuid(UUID);
    const _: () = assert!(FROM_UUID.is_same(GUID1));
    assert_eq!(GUID1, GUID2);
    assert_ne!(GUID1, GUID3);
    assert_eq!(FROM_UUID, GUID1);
}
