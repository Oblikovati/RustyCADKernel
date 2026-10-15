//! Printers of the probe, the twin of tools/oracle/tkmeshvs_probe_print.cxx, over the
//! shared token printers of the geometry probes.
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
mod tokens;
pub use tokens::{Reader, attempt, end, flag, int, real};

use super::display_driver::DisplayTrace;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;

/// A text as one `t<hex bytes>` token (the native `ga_text`).
pub fn text(bytes: &[u8]) {
    let digits: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    print!("t{digits} ");
}
/// The bytes of a native C string, up to the first NUL.
pub fn c_string(bytes: &[u8]) -> &[u8] {
    &bytes[..bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len())]
}
pub fn pnt(p: &Point3) {
    real(p.x());
    real(p.y());
    real(p.z());
}
pub fn color(c: &Color) {
    real(c.red());
    real(c.green());
    real(c.blue());
}
fn material(m: &Graphic3dMaterialAspect) {
    int(m.name() as i32);
    real(f64::from(m.transparency()));
    color(&m.ambient_color());
    color(&m.diffuse_color());
    color(&m.specular_color());
    color(&m.emissive_color());
}
pub fn aspect(a: &Handle<dyn Graphic3dAspectsSource>) {
    flag(a.is_null());
    let Some(a) = a.get() else {
        return;
    };
    color(&a.interior_color());
    color(&a.back_interior_color());
    color(&a.edge_color());
    int(a.interior_style() as i32);
    flag(a.distinguish());
    flag(a.to_draw_edges());
    int(a.shading_model() as i32);
    material(&a.front_material());
    int(a.back_material().name() as i32);
    int(a.line_type() as i32);
    real(f64::from(a.line_width()));
    int(a.marker_type() as i32);
    real(f64::from(a.marker_scale()));
    flag(a.to_map_texture());
    flag(a.texture_map().is_null());
    int(a.text_font_aspect() as i32);
    int(a.text_style() as i32);
    int(a.text_display_type() as i32);
    let font = a.text_font();
    match font.get() {
        Some(font) => text(c_string(font.string().as_bytes())),
        None => text(&[]),
    }
}
pub fn structure(prs: &Handle<dyn Graphic3dStructureSource>) {
    flag(prs.is_null());
    let Some(prs) = prs.get() else {
        return;
    };
    let groups = prs.groups();
    int(groups.len() as i32);
    for group in groups.iter().filter_map(Handle::get) {
        flag(group.is_closed());
        aspect(&group.aspects());
    }
}
pub fn trace(trace: &DisplayTrace) {
    let trace = trace.read();
    int(trace.events.len() as i32);
    for v in &trace.events {
        real(*v);
    }
    int(trace.texts.len() as i32);
    for t in trace.texts.iter().filter_map(Handle::get) {
        pnt(&t.position());
        real(f64::from(t.height()));
        text(c_string(t.text().to_c_string()));
    }
}
