//! Integer-quantized native state retains the defined NaN sentinel without loose tolerances.
use super::*;
use crate::{prs_mgr::*, prs3d::Prs3dDrawer};
use rustycad_tkernel::{quantity::Color, standard::Failure};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use std::fmt::Write;
fn appearance_number(value: f64) -> i64 {
    if value.is_nan() {
        return -999;
    }
    (value * 1000.0).round() as i64
}
pub(super) fn appearance_color(rows: &mut String, color: Color) {
    for channel in color.rgb() {
        write!(rows, " {}", appearance_number(f64::from(channel))).unwrap();
    }
}
pub(super) fn appearance_drawer(rows: &mut String, label: &str, drawer: &AisColoredDrawer) {
    write!(rows, "{label}").unwrap();
    appearance_drawer_flags(rows, drawer);
    write!(
        rows,
        " {}",
        appearance_number(f64::from(drawer.transparency()))
    )
    .unwrap();
    appearance_drawer_aspects(rows, drawer);
}
fn appearance_drawer_aspects(rows: &mut String, drawer: &Prs3dDrawer) {
    for line in super::shape_appearance::shape_colored_line_aspects(drawer) {
        write!(
            rows,
            " {}",
            appearance_number(f64::from(line.aspect().width()))
        )
        .unwrap();
        appearance_color(rows, line.aspect().interior_color());
    }
    appearance_color(rows, drawer.point_aspect().aspect().interior_color());
    let facing = AspectTypeOfFacingModel::BothSide;
    appearance_color(rows, drawer.shading_aspect().color(facing));
    writeln!(
        rows,
        " {}",
        appearance_number(drawer.shading_aspect().transparency(facing))
    )
    .unwrap();
}
pub(super) fn appearance_parent(rows: &mut String, label: &str, object: &AisColoredShape) {
    writeln!(
        rows,
        "{label} {} {} {} {} {} {}",
        object.custom_aspects_map().len(),
        i32::from(object.has_color()),
        i32::from(object.has_width()),
        i32::from(object.has_material()),
        appearance_number(object.width()),
        appearance_number(object.transparency())
    )
    .unwrap();
}
pub(super) fn appearance_failure(rows: &mut String, label: &str, result: Result<(), Failure>) {
    let failure =
        result.expect_err("appearance accepted invalid value; expected pinned native failure");
    writeln!(rows, "{label} {}", failure.exception_type()).unwrap();
}

fn appearance_drawer_flags(rows: &mut String, drawer: &AisColoredDrawer) {
    for flag in [
        drawer.has_own_color(),
        drawer.has_own_width(),
        drawer.has_own_transparency(),
        drawer.has_own_material(),
        drawer.has_own_shading_aspect(),
        drawer.has_own_line_aspect(),
        drawer.has_own_wire_aspect(),
        drawer.has_own_point_aspect(),
    ] {
        write!(rows, " {}", i32::from(flag)).unwrap();
    }
}
