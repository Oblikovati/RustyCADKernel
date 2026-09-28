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

//! Original UnitsAPI assertions; independent injected unit contexts replace global mutable state.
#![allow(non_snake_case)]
use rustycad_tkernel::units::{UnitSentence, UnitSystem, UnitsApi, UnitsDictionary};
fn api() -> UnitsApi {
    UnitsApi::new(UnitsDictionary::builtin().unwrap())
}
#[test]
fn UnitsAPI_Test__BUC60727_AnyToLS_Conversion() {
    let mut units = api();
    units.set_local_system(UnitSystem::Mdtv).unwrap();
    assert_eq!(units.any_to_ls(3.0, "mm").unwrap(), 3.0);
}
#[test]
fn UnitsAPI_Test__AnyToAny_UnknownUnit() {
    let units = api();
    assert_eq!(units.any_to_any(1.0, "unknown_unit", "mm").unwrap(), 1.0);
    assert_eq!(units.any_to_any(1.0, "mm", "unknown_unit").unwrap(), 1.0);
}
#[test]
fn UnitsAPI_Test__FClassesBug_30800_PoiseConversion() {
    let units = api();
    assert!((units.any_to_si(1.0, "Po").unwrap() - 0.1).abs() <= 1e-12);
    assert!((units.any_to_any(1.0, "Po", "kg/cm/s").unwrap() - 0.001).abs() <= 1e-12);
}
#[test]
fn UnitsAPI_Test__FClassesBug_11568_2_MinuteAlias() {
    assert_eq!(api().any_to_si(1.0, "mn").unwrap(), 60.0);
}
#[test]
fn UnitsAPI_Test__FClassesBug_11568_4_MinuteSymbol() {
    assert_eq!(api().any_to_si(1.0, "min").unwrap(), 60.0);
}
fn parsed_word(expression: &str) -> String {
    let mut units = api();
    units.set_local_system(UnitSystem::Si).unwrap();
    let mut sentence = UnitSentence::new(units.dictionary(), expression.as_bytes()).unwrap();
    assert!(sentence.is_done());
    let token = sentence.evaluate().unwrap();
    assert!(token.is_some());
    String::from_utf8(token.unwrap().word().to_vec()).unwrap()
}
#[test]
fn UnitsAPI_Test__FClassesBug_11568_1_MinuteAliasParsing() {
    assert_eq!(parsed_word("mn"), "min");
}
#[test]
fn UnitsAPI_Test__FClassesBug_11568_3_MinuteSymbolParsing() {
    assert_eq!(parsed_word("min"), "min");
}
#[test]
fn UnitsAPI_Test__FClassesBug_9848_1_InvalidUnitSentence() {
    let mut units = api();
    units.set_local_system(UnitSystem::Si).unwrap();
    let sentence = UnitSentence::new(units.dictionary(), b"tonne/(mm*s**2)").unwrap();
    assert!(!sentence.is_done());
}
#[test]
fn UnitsAPI_Test__FClassesBug_9848_2_ValidCompositeUnitSentence() {
    assert_eq!(parsed_word("ton/(m*s**2)"), "(ton)/((m)*((s)**(2)))");
}
