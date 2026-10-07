//! The deprecated `Graphic3d_Group::Text` and `Marker` short-cuts: each builds a
//! `Graphic3d_Text` (or a one-point array) and adds it like `AddText` (`AddPrimitiveArray`).
use super::{
    Graphic3dArrayOfPoints, Graphic3dGroup, Graphic3dGroupMembers as _,
    Graphic3dHorizontalTextAlignment, Graphic3dText, Graphic3dTextPath, Graphic3dVertex,
    Graphic3dVerticalTextAlignment,
};
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkmath::gp::{Axis2, Point3};

/// The text layout arguments of the long `Text` overloads (the angle and the path are
/// ignored natively).
#[derive(Clone, Copy, Debug)]
pub struct Graphic3dGroupTextLayout {
    /// `AHeight`.
    pub height: f64,
    /// `AAngle` (unused).
    pub angle: f64,
    /// `ATp` (unused).
    pub path: Graphic3dTextPath,
    /// `AHta`.
    pub horizontal: Graphic3dHorizontalTextAlignment,
    /// `AVta`.
    pub vertical: Graphic3dVerticalTextAlignment,
}

/// A text of `height` (rounded to float) at `point`.
fn text_at(height: f64, point: &Graphic3dVertex) -> Graphic3dText {
    let text = Graphic3dText::new(height as f32);
    text.set_position(&Point3::new(
        f64::from(point.x()),
        f64::from(point.y()),
        f64::from(point.z()),
    ));
    text
}

/// A text of the layout, aligned.
fn aligned_text(layout: &Graphic3dGroupTextLayout, point: &Graphic3dVertex) -> Graphic3dText {
    let text = text_at(layout.height, point);
    text.set_horizontal_alignment(layout.horizontal);
    text.set_vertical_alignment(layout.vertical);
    text
}

/// A text of the layout in the plane of `orientation`.
fn oriented_text(
    layout: &Graphic3dGroupTextLayout,
    orientation: &Axis2,
    has_own_anchor: bool,
) -> Graphic3dText {
    let text = Graphic3dText::new(layout.height as f32);
    text.set_orientation(orientation);
    text.set_own_anchor_point(has_own_anchor);
    text.set_horizontal_alignment(layout.horizontal);
    text.set_vertical_alignment(layout.vertical);
    text
}

/// `TCollection_ExtendedString::ToExtString` as the UTF-8 text of `Graphic3d_Text`.
fn utf8_of(text: &ExtendedString) -> String8 {
    String8::from_unicode_units(text.as_utf16_with_nul(), -1)
}

/// The default body of the virtual `Text(const char*, Graphic3d_Vertex, ...)`.
pub(crate) fn text_at_vertex<G: Graphic3dGroup + ?Sized>(
    group: &G,
    text: &str,
    point: &Graphic3dVertex,
    layout: &Graphic3dGroupTextLayout,
    to_eval_min_max: bool,
) {
    let params = aligned_text(layout, point);
    params.set_text_str(text);
    group.add_text(&Handle::new(params), to_eval_min_max);
}

/// The default body of the virtual `Text(const char*, gp_Ax2, ...)`.
pub(crate) fn text_in_plane<G: Graphic3dGroup + ?Sized>(
    group: &G,
    text: &str,
    orientation: &Axis2,
    layout: &Graphic3dGroupTextLayout,
    flags: [bool; 2],
) {
    let params = oriented_text(layout, orientation, flags[1]);
    params.set_text_str(text);
    group.add_text(&Handle::new(params), flags[0]);
}

/// The default body of the virtual `Text(TCollection_ExtendedString, gp_Ax2, ...)`.
pub(crate) fn ext_text_in_plane<G: Graphic3dGroup + ?Sized>(
    group: &G,
    text: &ExtendedString,
    orientation: &Axis2,
    layout: &Graphic3dGroupTextLayout,
    flags: [bool; 2],
) {
    let params = oriented_text(layout, orientation, flags[1]);
    params.set_text(&utf8_of(text));
    group.add_text(&Handle::new(params), flags[0]);
}

/// The non-virtual deprecated `Text` and `Marker` short-cuts of every group.
pub trait Graphic3dGroupTextMembers: Graphic3dGroup {
    /// `Marker`: a one-point array at `point`; `group.marker(&point, true)?`.
    fn marker(&self, point: &Graphic3dVertex, to_eval_min_max: bool) -> Result<(), Failure> {
        let points = Graphic3dArrayOfPoints::new(1, 0)?;
        points.add_vertex(point.x(), point.y(), point.z())?;
        self.add_primitive_array(&Handle::new(points).cast(), to_eval_min_max);
        Ok(())
    }
    /// `Text(const char*, Graphic3d_Vertex, height, evalMinMax)`; `group.simple_text("A", &point, 12.0, true)`.
    fn simple_text(&self, text: &str, point: &Graphic3dVertex, height: f64, to_eval_min_max: bool) {
        let params = text_at(height, point);
        params.set_text_str(text);
        self.add_text(&Handle::new(params), to_eval_min_max);
    }
    /// `Text(TCollection_ExtendedString, Graphic3d_Vertex, height, angle, path, hta, vta, evalMinMax)`;
    /// `group.ext_text_at_vertex(&text, &point, &layout, true)`.
    fn ext_text_at_vertex(
        &self,
        text: &ExtendedString,
        point: &Graphic3dVertex,
        layout: &Graphic3dGroupTextLayout,
        to_eval_min_max: bool,
    ) {
        let params = aligned_text(layout, point);
        params.set_text(&utf8_of(text));
        self.add_text(&Handle::new(params), to_eval_min_max);
    }
    /// `Text(TCollection_ExtendedString, Graphic3d_Vertex, height, evalMinMax)`;
    /// `group.ext_simple_text(&text, &point, 12.0, true)`.
    fn ext_simple_text(
        &self,
        text: &ExtendedString,
        point: &Graphic3dVertex,
        height: f64,
        to_eval_min_max: bool,
    ) {
        let params = text_at(height, point);
        params.set_text(&utf8_of(text));
        self.add_text(&Handle::new(params), to_eval_min_max);
    }
}
impl<T: Graphic3dGroup + ?Sized> Graphic3dGroupTextMembers for T {}
