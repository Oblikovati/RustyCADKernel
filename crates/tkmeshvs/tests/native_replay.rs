// SPDX-License-Identifier: GPL-2.0-only
//! Static native transcript replay; this test never launches OCCT.
mod probe;
use martensite_tkmeshvs::{MeshVSDrawer, MeshVSDrawerSource, flags::*};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Handle, RuntimeTyped},
    tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::{Graphic3dMaterialAspect, Graphic3dNameOfMaterial};
use std::fmt::Write;
#[test]
fn actual_native_drawer_channels_assignment_dispatch_and_constants_replay() {
    let drawer = Handle::new(MeshVSDrawer::default());
    let mut rows = String::new();
    scalars(&drawer, &mut rows);
    values(&drawer, &mut rows);
    assignment(&drawer, &mut rows);
    probe::dispatch(&drawer, &mut rows);
    rtti_row(&drawer, &mut rows);
    flags_row(&mut rows);
    assert_eq!(rows, include_str!("fixtures/drawer-native.txt"));
}
fn scalars(drawer: &MeshVSDrawer, rows: &mut String) {
    scalar_row(drawer, rows, "miss");
    drawer.set_integer(7, -23);
    drawer.set_double(7, 3.5);
    drawer.set_boolean(7, false);
    drawer.set_integer(7, 42);
    scalar_row(drawer, rows, "scalar");
}
fn scalar_row(drawer: &MeshVSDrawer, rows: &mut String, label: &str) {
    let (mut integer, mut real, mut boolean) = (91, 7.25, true);
    let found = [
        drawer.get_integer(7, &mut integer),
        drawer.get_double(7, &mut real),
        drawer.get_boolean(7, &mut boolean),
    ];
    writeln!(
        rows,
        "{label} {} {integer} {} {real} {} {}",
        found[0] as i32, found[1] as i32, found[2] as i32, boolean as i32
    )
    .unwrap();
}
fn values(drawer: &MeshVSDrawer, rows: &mut String) {
    let mut output = Color::named(NamedColor::RED);
    let mut result = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Brass);
    let mut copied = AsciiString::new("sentinel");
    missed_values(drawer, rows, &mut output, &mut result, &mut copied);
    populate_values(drawer);
    drawer.get_color(-3, &mut output);
    drawer.get_material(-3, &mut result);
    drawer.get_ascii_string(-3, &mut copied);
    stored_values(rows, output, &result, &copied);
}
fn missed_values(
    drawer: &MeshVSDrawer,
    rows: &mut String,
    color: &mut Color,
    material: &mut Graphic3dMaterialAspect,
    text: &mut AsciiString,
) {
    let found = [
        drawer.get_color(-3, color),
        drawer.get_material(-3, material),
        drawer.get_ascii_string(-3, text),
    ];
    missing_value_row(rows, found, color, material, text);
}
fn missing_value_row(
    rows: &mut String,
    found: [bool; 3],
    color: &Color,
    material: &Graphic3dMaterialAspect,
    text: &AsciiString,
) {
    let name = std::str::from_utf8(text.as_bytes()).unwrap();
    writeln!(
        rows,
        "miss-values {} {} {} {} {} {name}",
        found[0] as i32,
        color.red(),
        found[1] as i32,
        material.name() as i32,
        found[2] as i32
    )
    .unwrap();
}
fn stored_values(
    rows: &mut String,
    color: Color,
    material: &Graphic3dMaterialAspect,
    text: &AsciiString,
) {
    let name = std::str::from_utf8(text.as_bytes()).unwrap();
    writeln!(
        rows,
        "values {} {} {} {} {} {name}",
        color.red(),
        color.green(),
        color.blue(),
        material.name() as i32,
        material.transparency()
    )
    .unwrap();
}
fn populate_values(drawer: &MeshVSDrawer) {
    let color = Color::new(
        0.125,
        0.25,
        0.5,
        rustycad_tkernel::quantity::ColorSpace::Rgb,
    )
    .unwrap();
    let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold);
    let mut text = AsciiString::new("native drawer");
    material.set_transparency(0.375).unwrap();
    drawer.set_color(-3, &color);
    drawer.set_material(-3, &material);
    drawer.set_ascii_string(-3, &text);
    material.set_transparency(0.75).unwrap();
    text.set_value(1, b"X").unwrap();
}
fn assignment(drawer: &Handle<MeshVSDrawer>, rows: &mut String) {
    let target = Handle::new(MeshVSDrawer::default());
    target.assign(drawer);
    drawer.assign(drawer);
    drawer.assign(&Handle::<dyn MeshVSDrawerSource>::default());
    drawer.set_integer(7, 99);
    let mut integer = 0;
    target.get_integer(7, &mut integer);
    writeln!(
        rows,
        "assign {} {} {}",
        integer,
        drawer.ref_count(),
        target.ref_count()
    )
    .unwrap();
    removal_row(&target, rows);
}
fn removal_row(target: &MeshVSDrawer, rows: &mut String) {
    let removed = [
        target.remove_integer(7),
        target.remove_integer(7),
        target.remove_double(7),
        target.remove_boolean(7),
        target.remove_color(-3),
        target.remove_material(-3),
        target.remove_ascii_string(-3),
    ];
    let flags = removed.map(|value| (value as i32).to_string()).join(" ");
    writeln!(rows, "remove {flags}").unwrap();
}
fn rtti_row(drawer: &Handle<MeshVSDrawer>, rows: &mut String) {
    let kind = drawer.is_kind(rustycad_tkernel::standard::StandardTransient::static_type());
    let exact = drawer.is_instance(MeshVSDrawer::static_type());
    writeln!(
        rows,
        "rtti {} {} {}",
        drawer.dynamic_type().unwrap().name(),
        kind as i32,
        exact as i32
    )
    .unwrap();
}
fn flags_row(rows: &mut String) {
    writeln!(
        rows,
        "flags {} {} {} {} {}",
        MeshVSDrawerAttribute::User as i32,
        MESHVS_DMF_OCC_MASK,
        MESHVS_DMF_DEFORMED_MASK,
        MESHVS_ET_ELEMENT.0,
        MESHVS_ET_ALL.0
    )
    .unwrap();
}
