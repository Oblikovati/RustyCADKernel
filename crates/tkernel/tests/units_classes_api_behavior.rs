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

//! Behaviour of the Units classes, the UnitsAPI context and their native quirks.
#[path = "units_support/mod.rs"]
mod units_support;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::units::{
    Dimensions, Explorer, Measurement, UnitSentence, Units, UnitsSystem,
};
use rustycad_tkernel::units_api::{SystemUnits, UnitsApi};
use std::rc::Rc;
use std::sync::Arc;
use units_support::{InMemoryUnitsResources, RecordingUnitsDiagnostics, units_with};

fn context() -> (Arc<Units>, RecordingUnitsDiagnostics) {
    let recorder = RecordingUnitsDiagnostics::default();
    (units_with(&recorder), recorder)
}

#[test]
fn empty_caches_answer_like_the_zero_initialized_statics() {
    let (units, _) = context();
    let mut dimensions = Handle::default();
    assert_eq!(
        units.to_si_dimensions(2.5, b"", &mut dimensions).unwrap(),
        0.0
    );
    assert!(dimensions.is_null());
    assert!(units.from_si(2.5, b"").unwrap().is_infinite());
    assert_eq!(units.first_quantity(b"").unwrap().unwrap(), "");
    assert_eq!(units.first_quantity(b"m").unwrap().unwrap(), "LENGTH");
    assert!(units.first_quantity(b"zz").unwrap().is_none());
}

#[test]
fn dictionary_quantities_units_and_explorer_walks() {
    let (units, recorder) = context();
    let quantity = units.quantity(b"LENGTH").unwrap();
    assert_eq!(quantity.get().unwrap().name(), "LENGTH");
    assert!(units.quantity(b"BOGUS").unwrap().is_null());
    let dictionary = units.dictionary_of_units(false).unwrap();
    let dictionary = dictionary.get().unwrap();
    assert_eq!(
        dictionary.active_unit(b"LENGTH", &mut Vec::new()).unwrap(),
        "m"
    );
    let mut warning = Vec::new();
    assert_eq!(dictionary.active_unit(b"BOGUS", &mut warning).unwrap(), "");
    assert_eq!(warning, b" La grandeur physique BOGUS n'existe pas.\n");
    let mut explorer = Explorer::for_dictionary_quantity(dictionary, b"MASS").unwrap();
    assert_eq!(explorer.quantity().unwrap(), "MASS");
    assert!(explorer.is_active().unwrap());
    explorer.next_unit();
    assert!(!explorer.is_active().unwrap());
    let missing = Explorer::for_dictionary_quantity(dictionary, b"BOGUS").unwrap();
    let failure = missing.quantity().err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    let failure = units.dimensions(Some(b"BOGUS")).err().unwrap();
    assert!(failure.is_kind("Standard_DomainError"));
    assert!(recorder.printed.lock().unwrap().is_empty());
}

#[test]
fn dimensions_quantity_uses_the_first_captured_sequence() {
    let (units, _) = context();
    let length = Dimensions::a_length();
    assert_eq!(
        length.get().unwrap().quantity(&units).unwrap().unwrap(),
        "LENGTH"
    );
    let odd = Dimensions::handle([9.0; 9]);
    assert!(odd.get().unwrap().quantity(&units).unwrap().is_none());
}

#[test]
fn recreating_the_dictionary_is_an_error_instead_of_a_native_hang() {
    let (units, recorder) = context();
    units.dictionary_of_units(false).unwrap();
    assert!(units.dictionary_of_units(true).is_err());
    let printed = String::from_utf8_lossy(&recorder.printed.lock().unwrap()).into_owned();
    assert!(printed.contains(" encountered twice with the same signification : U\n"));
}

#[test]
fn measurement_conversion_and_warnings() {
    let (units, recorder) = context();
    let mut length = Measurement::with_unit(&units, 2.5, b"km").unwrap();
    length.convert(&units, b"m").unwrap();
    assert_eq!(length.measurement(), 2500.0);
    assert_eq!(length.token().get().unwrap().word(), "m");
    length.convert(&units, b"s").unwrap();
    assert_eq!(length.measurement(), 2500.0);
    length.convert(&units, b"zz").unwrap();
    let mut invalid = Measurement::with_unit(&units, 1.0, b"zz").unwrap();
    invalid.convert(&units, b"m").unwrap();
    assert_eq!(units.convert(3.0, b"zz", b"m").unwrap(), 3.0);
    let messages = recorder.messages.lock().unwrap().clone();
    assert_eq!(
        messages,
        [
            "W:Units_Measurement: can not convert - incorrect unit 'zz' => result is not correct",
            "W:Units_Measurement: can not convert - incorrect unit => result is not correct",
            "W:Units::Convert: can not convert - incorrect unit 'zz' => result is not correct",
        ]
    );
}

#[test]
fn units_system_keeps_the_native_quirks() {
    let (units, recorder) = context();
    let system = UnitsSystem::new();
    system.specify(&units, b"LENGTH", b"mm").unwrap();
    system.specify(&units, b"LENGTH", b"cm").unwrap();
    system.specify(&units, b"MASS", b"kg").unwrap();
    system.specify(&units, b"BOGUS", b"m").unwrap();
    system.activates().unwrap();
    assert_eq!(system.active_unit(b"LENGTH").unwrap(), "mm");
    let value = system
        .convert_si_value_to_user_system(&units, b"LENGTH", 1.0)
        .unwrap();
    assert_eq!(value, 1000.0);
    // Two quantities bound the unit search, so a third unit is never found.
    system.specify(&units, b"LENGTH", b"dm").unwrap();
    let failure = system.activate(b"LENGTH", b"dm").err().unwrap();
    assert_eq!(failure.exception_type(), "Units_NoSuchUnit");
    assert!(failure.is_kind("Standard_NoSuchObject"));
    // Removing the only unit removes the quantity and still raises.
    assert!(system.remove(b"MASS", b"kg").is_err());
    assert_eq!(
        system
            .quantities_sequence()
            .get()
            .unwrap()
            .sequence(|s| s.len()),
        1
    );
    let failure = system.active_unit(b"MASS").err().unwrap();
    assert_eq!(failure.exception_type(), "Units_NoSuchType");
    let printed = String::from_utf8_lossy(&recorder.printed.lock().unwrap()).into_owned();
    assert_eq!(
        printed,
        "Warning: in Units_UnitsSystem : Units_NoSuchType 'BOGUS'\n"
    );
}

#[test]
fn shifted_active_unit_converts_to_si_with_the_native_formula() {
    let (units, _) = context();
    let system = UnitsSystem::new();
    system
        .specify(&units, b"THERMODYNAMIC TEMPERATURE", b"deg.C")
        .unwrap();
    system.activates().unwrap();
    let quantity = b"THERMODYNAMIC TEMPERATURE";
    let to_si = system
        .convert_user_system_value_to_si(&units, quantity, 10.0)
        .unwrap();
    assert_eq!(to_si, 10.0 * (1.0 + 273.15));
    let from_si = system
        .convert_si_value_to_user_system(&units, quantity, 283.15)
        .unwrap();
    assert_eq!(from_si, 283.15 / 1.0 - 273.15);
    let copy = system.clone();
    copy.specify(&units, quantity, b"deg.F").unwrap();
    let mut out = Vec::new();
    system.dump(&mut out).unwrap();
    let text = String::from_utf8(out).unwrap();
    assert_eq!(
        text,
        " UNITSSYSTEM : \nTHERMODYNAMIC TEMPERATURE\n  deg.C\n  deg.F\n"
    );
}

#[test]
fn current_units_come_from_the_injected_resources() {
    let resources = Rc::new(InMemoryUnitsResources::default());
    resources.define("CurrentUnits", "/defaults", "LENGTH : mm\n");
    resources.define_user("CurrentUnits", "/user");
    let mut api = units_support::api_with(resources.clone());
    assert_eq!(api.current_unit(b"LENGTH").unwrap(), "mm");
    assert_eq!(api.current_to_si(2.0, b"LENGTH").unwrap(), 0.002);
    assert_eq!(api.current_from_si(0.002, b"LENGTH").unwrap(), 2.0);
    assert_eq!(api.current_to_any(2.0, b"LENGTH", b"cm").unwrap(), 0.2);
    assert_eq!(api.current_to_si(2.0, b"SPEED").unwrap(), 2.0);
    let failure = api.current_unit(b"SPEED").err().unwrap();
    assert_eq!(failure.exception_type(), "Resource_NoSuchResource");
    api.set_current_unit(b"SPEED", b"km/h").unwrap();
    assert_eq!(api.current_unit(b"SPEED").unwrap(), "km/h");
    assert!(api.save().unwrap());
    let saved = resources
        .files
        .borrow()
        .get(std::path::Path::new("/user/CurrentUnits"))
        .cloned();
    assert!(String::from_utf8(saved.unwrap()).unwrap().contains("SPEED"));
    api.set_local_system(SystemUnits::Mdtv).unwrap();
    assert_eq!(api.current_to_ls(2.0, b"LENGTH").unwrap(), 2.0);
    assert!(!api.check(b"LENGTH", b"mm").unwrap());
    api.reload().unwrap();
    assert_eq!(api.local_system(), SystemUnits::Mdtv);
}

#[test]
fn any_to_ls_dimensions_reports_the_native_null_dereference() {
    let mut api = units_support::api();
    let mut dimensions = Handle::default();
    assert!(
        api.any_to_ls_dimensions(1.0, b"zz", &mut dimensions)
            .is_err()
    );
    let mut dimensions = Dimensions::a_less();
    assert_eq!(
        api.any_to_ls_dimensions(1.0, b"zz", &mut dimensions)
            .unwrap(),
        0.0
    );
    assert!(api.any_from_ls(1.0, b"zz").is_err());
    assert_eq!(UnitsApi::dimension_length().get().unwrap().length(), 1.0);
    assert_eq!(
        api.dimensions(Some(b"AREA"))
            .unwrap()
            .get()
            .unwrap()
            .length(),
        2.0
    );
}

#[test]
fn unit_sentence_with_explicit_quantities_binds_only_those_units() {
    let (units, _) = context();
    let dictionary = units.dictionary_of_units(false).unwrap();
    let quantities = dictionary.get().unwrap().sequence();
    let mut sentence = UnitSentence::with_quantities(&units, b"km", &quantities).unwrap();
    assert_eq!(sentence.evaluate().unwrap().get().unwrap().value(), 1000.0);
    let mut out = Vec::new();
    UnitSentence::new(&units, b"mm")
        .unwrap()
        .dump(&mut out)
        .unwrap();
    assert!(String::from_utf8_lossy(&out).starts_with(" UNIT SENTENCE with : \n"));
}
