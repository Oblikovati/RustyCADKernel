//! Compare touched native fields bit-for-bit, not a tolerance or rendered approximation.
use super::AspectOwner;
use rustycad_tkernel::quantity::Color;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsEdgeMembers, Graphic3dAspectsLineMembers, Graphic3dAspectsMembers,
    Graphic3dAspectsSource, Graphic3dAspectsTextMembers,
};
use std::fmt::Write;
pub(super) fn real(rows: &mut String, value: f64) {
    write!(rows, " {}", value.to_bits()).unwrap();
}
fn color(rows: &mut String, value: Color) {
    real(rows, value.red());
    real(rows, value.green());
    real(rows, value.blue());
}
pub(super) fn aspect(rows: &mut String, kind: i32, value: &AspectOwner) {
    let Some(value) = value.get() else {
        rows.push_str(" null");
        return;
    };
    rows.push_str(" value");
    if kind < 2 {
        fill(rows, value);
        return;
    }
    color(rows, value.interior_color());
    match kind {
        2 => line(rows, value),
        3 => {
            write!(rows, " {}", value.marker_type() as i32).unwrap();
            real(rows, value.marker_scale() as f64);
        }
        _ => font(rows, value),
    }
}
fn fill(rows: &mut String, value: &dyn Graphic3dAspectsSource) {
    write!(rows, " {}", value.interior_style() as i32).unwrap();
    color(rows, value.interior_color());
    color(rows, value.back_interior_color());
    color(rows, value.edge_color());
    write!(rows, " {}", value.line_type() as i32).unwrap();
    real(rows, value.line_width() as f64);
    let hatch = value.hatch_style();
    write!(
        rows,
        " {}",
        hatch.get().map_or(-1, |style| style.hatch_type())
    )
    .unwrap();
    let front = value.front_material();
    let back = value.back_material();
    write!(rows, " {} {}", front.name() as i32, back.name() as i32).unwrap();
    real(rows, front.transparency() as f64);
    real(rows, back.transparency() as f64);
}
fn line(rows: &mut String, value: &dyn Graphic3dAspectsSource) {
    write!(rows, " {}", value.line_type() as i32).unwrap();
    real(rows, value.line_width() as f64);
    write!(rows, " {}", value.line_pattern()).unwrap();
}
fn font(rows: &mut String, value: &dyn Graphic3dAspectsSource) {
    write!(
        rows,
        " {} {} {} ",
        value.text_style() as i32,
        value.text_display_type() as i32,
        value.text_font_aspect() as i32
    )
    .unwrap();
    let font = value.text_font();
    let Some(font) = font.get() else {
        rows.push('-');
        return;
    };
    for byte in font.string().as_bytes() {
        write!(rows, "{byte:x}:").unwrap();
    }
}
