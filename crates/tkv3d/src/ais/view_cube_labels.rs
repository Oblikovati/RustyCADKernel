//! Native face labels use the outward plane frame with Y-up/Z-up orientation rules.
use super::AisViewCubePresentationSource;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::prs3d::Prs3dTextAspect;
use crate::v3d::{V3d, V3dTypeOfOrientation as Orientation};
use rustycad_tkernel::{
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkmath::gp::{Axis2, Direction3};
use rustycad_tkservice::graphic3d::*;
pub(super) fn submit_cube_side_labels<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    let settings = object.view_cube_presentation_base().settings.read().clone();
    let aspect = object.attributes().text_aspect();
    let group = prs.new_group()?;
    group.set_group_primitives_aspect(&aspect.aspect().cast());
    for (index, label) in settings.side_labels.iter().enumerate() {
        if label.is_empty() {
            continue;
        }
        let direction = V3d::get_proj_axis(Orientation::ALL[index]);
        let position = super::view_cube_geometry::cube_point(
            direction.xyz()
                * (settings.geometry.size * 0.5 + settings.geometry.facet_extension + 2.0),
        );
        let up = cube_face_up(settings.is_y_up, Orientation::ALL[index]);
        let frame = Axis2::with_x_direction(position, direction, up.crossed(direction)?)?;
        submit_cube_face_label(&group, &aspect, label, &frame);
    }
    Ok(())
}
fn cube_face_up(is_y_up: bool, orientation: Orientation) -> Direction3 {
    if is_y_up {
        return if matches!(orientation, Orientation::Ypos | Orientation::Yneg) {
            Direction3::Z.reversed()
        } else {
            Direction3::Y
        };
    }
    match orientation {
        Orientation::Zpos => Direction3::Y,
        Orientation::Zneg => Direction3::Y.reversed(),
        _ => Direction3::Z,
    }
}
fn submit_cube_face_label(
    group: &Handle<dyn Graphic3dGroup>,
    aspect: &Prs3dTextAspect,
    label: &AsciiString,
    frame: &Axis2,
) {
    let text = Handle::new(Graphic3dText::new(aspect.height() as f32));
    text.set_text_ascii(label);
    text.set_orientation(frame);
    text.set_own_anchor_point(false);
    text.set_horizontal_alignment(aspect.horizontal_justification());
    text.set_vertical_alignment(aspect.vertical_justification());
    group.add_text(&text, true);
}
