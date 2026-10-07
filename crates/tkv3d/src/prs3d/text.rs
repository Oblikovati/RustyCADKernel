//! `Prs3d_Text`: adds a text with a text aspect to a group.
use super::Prs3dTextAspect;
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkservice::graphic3d::{Graphic3dGroup, Graphic3dText};

/// The package class `Prs3d_Text`.
pub struct Prs3dText;

/// The UTF-8 text of `TCollection_ExtendedString::ToExtString()`, read up to its first NUL as
/// the native `NCollection_String` constructor does.
fn text_of(text: &ExtendedString) -> String8 {
    let mut units = text.as_utf16().to_vec();
    units.push(0);
    String8::from_unicode_units(&units, -1)
}

/// A new text of the aspect's height and justification, after setting the group aspect.
fn new_text(
    group: &dyn Graphic3dGroup,
    aspect: &Prs3dTextAspect,
    text: &ExtendedString,
) -> Handle<Graphic3dText> {
    group.set_primitives_aspect(&aspect.aspect().cast());
    let graphic = Graphic3dText::new(aspect.height() as f32);
    graphic.set_text(&text_of(text));
    Handle::new(graphic)
}

/// Sets the alignments and adds the text to the group.
fn finish_text(group: &dyn Graphic3dGroup, aspect: &Prs3dTextAspect, text: &Handle<Graphic3dText>) {
    if let Some(graphic) = text.get() {
        graphic.set_horizontal_alignment(aspect.horizontal_justification());
        graphic.set_vertical_alignment(aspect.vertical_justification());
    }
    group.add_text(text, true);
}

impl Prs3dText {
    /// `Draw(group, aspect, text, attachmentPoint)`: a screen-aligned text at a point; null
    /// when the group or the aspect is null (natively a null dereference).
    /// Example: `Prs3dText::draw(&group, &aspect, &ExtendedString::new("A"), &p)`.
    pub fn draw(
        group: &Handle<dyn Graphic3dGroup>,
        aspect: &Handle<Prs3dTextAspect>,
        text: &ExtendedString,
        attachment_point: &Point3,
    ) -> Handle<Graphic3dText> {
        let (Some(group), Some(aspect)) = (group.get(), aspect.get()) else {
            return Handle::default();
        };
        let graphic = new_text(group, aspect, text);
        if let Some(value) = graphic.get() {
            value.set_position(attachment_point);
        }
        finish_text(group, aspect, &graphic);
        graphic
    }
    /// `Draw(group, aspect, text, orientation, hasOwnAnchor)`: a text in a plane.
    /// Example: `Prs3dText::draw_oriented(&group, &aspect, &text, &axes, true)`.
    pub fn draw_oriented(
        group: &Handle<dyn Graphic3dGroup>,
        aspect: &Handle<Prs3dTextAspect>,
        text: &ExtendedString,
        orientation: &Axis2,
        has_own_anchor: bool,
    ) -> Handle<Graphic3dText> {
        let (Some(group), Some(aspect)) = (group.get(), aspect.get()) else {
            return Handle::default();
        };
        let graphic = new_text(group, aspect, text);
        if let Some(value) = graphic.get() {
            value.set_orientation(orientation);
            value.set_own_anchor_point(has_own_anchor);
        }
        finish_text(group, aspect, &graphic);
        graphic
    }
}
