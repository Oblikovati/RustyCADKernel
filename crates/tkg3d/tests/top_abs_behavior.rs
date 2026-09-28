// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! Behaviour tests of the TopAbs package (OCCT has no TopAbs GoogleTest suite); the
//! expected values are the native ones, which tools/oracle/tkg3d_probe_topabs.cxx
//! also checks bit for bit.
use Orientation::{External, Forward, Internal, Reversed};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use std::io;

fn printed(write: impl FnOnce(&mut Vec<u8>) -> io::Result<()>) -> String {
    let mut stream: Vec<u8> = Vec::new();
    write(&mut stream).unwrap();
    String::from_utf8(stream).unwrap()
}

#[test]
fn enumerations_keep_the_native_order() {
    assert_eq!(ShapeEnum::Compound as i32, 0);
    assert_eq!(ShapeEnum::Vertex as i32, 7);
    assert_eq!(ShapeEnum::Shape as i32, 8);
    assert_eq!(External as i32, 3);
    assert_eq!(State::In as i32, 0);
    assert_eq!(State::Unknown as i32, 3);
    assert!(
        ShapeEnum::ALL
            .iter()
            .enumerate()
            .all(|(i, &t)| t as usize == i)
    );
    assert!(
        Orientation::ALL
            .iter()
            .enumerate()
            .all(|(i, &o)| o as usize == i)
    );
    assert!(State::ALL.iter().enumerate().all(|(i, &s)| s as usize == i));
}

#[test]
fn compose_follows_the_native_table() {
    assert_eq!(top_abs::compose(Forward, Reversed), Reversed);
    assert_eq!(top_abs::compose(Reversed, Reversed), Forward);
    assert_eq!(top_abs::compose(Reversed, Forward), Reversed);
    assert_eq!(top_abs::compose(Internal, Forward), Internal);
    assert_eq!(top_abs::compose(Internal, External), External);
    // The native table is read [Or2][Or1]: an INTERNAL Or2 wins over an EXTERNAL Or1,
    // contrary to the table in the TopAbs.hxx comment.
    assert_eq!(top_abs::compose(External, Internal), Internal);
    assert_eq!(top_abs::compose(External, Reversed), External);
}

#[test]
fn reverse_and_complement() {
    let reversed: Vec<Orientation> = Orientation::ALL.map(top_abs::reverse).to_vec();
    assert_eq!(reversed, [Reversed, Forward, Internal, External]);
    let complemented: Vec<Orientation> = Orientation::ALL.map(top_abs::complement).to_vec();
    assert_eq!(complemented, [Reversed, Forward, External, Internal]);
}

#[test]
fn names_of_shape_types_and_orientations() {
    assert_eq!(
        top_abs::shape_type_to_string(ShapeEnum::CompSolid),
        "COMPSOLID"
    );
    assert_eq!(top_abs::shape_type_to_string(ShapeEnum::Shape), "SHAPE");
    assert_eq!(top_abs::shape_orientation_to_string(Internal), "INTERNAL");
    let text: String = printed(|s| top_abs::print_shape_type(ShapeEnum::Edge, s));
    assert_eq!(text, "EDGE");
    assert_eq!(
        printed(|s| top_abs::print_orientation(Reversed, s)),
        "REVERSED"
    );
}

#[test]
fn print_state_keeps_the_shifted_native_names() {
    let names: Vec<String> = State::ALL
        .map(|state| printed(|s| top_abs::print_state(state, s)))
        .to_vec();
    assert_eq!(names, ["ON", "IN", "OUT", "UNKNOWN"]);
}

#[test]
fn shape_type_from_string_ignores_ascii_case() {
    assert_eq!(top_abs::shape_type_from_string("face"), ShapeEnum::Face);
    assert_eq!(
        top_abs::shape_type_from_string("CompSolid"),
        ShapeEnum::CompSolid
    );
    assert_eq!(
        top_abs::try_shape_type_from_string("vErTeX"),
        Some(ShapeEnum::Vertex)
    );
    // The C string ends at the first NUL.
    assert_eq!(top_abs::shape_type_from_string("Edge\0x"), ShapeEnum::Edge);
}

#[test]
fn unknown_shape_type_names() {
    assert_eq!(top_abs::try_shape_type_from_string("FACES"), None);
    assert_eq!(top_abs::try_shape_type_from_string(" FACE"), None);
    assert_eq!(top_abs::try_shape_type_from_string(""), None);
    // Only ASCII letters change case, as with TCollection_AsciiString::UpperCase.
    assert_eq!(top_abs::try_shape_type_from_string("ßhell"), None);
    assert_eq!(top_abs::shape_type_from_string("unknown"), ShapeEnum::Shape);
}

#[test]
fn orientation_from_string() {
    assert_eq!(top_abs::shape_orientation_from_string("external"), External);
    assert_eq!(
        top_abs::try_shape_orientation_from_string("Reversed"),
        Some(Reversed)
    );
    assert_eq!(top_abs::try_shape_orientation_from_string("IN"), None);
    assert_eq!(top_abs::shape_orientation_from_string("OUT"), Forward);
}
