//! `DsgPrs_XYZAxisPresentation` at OCCT 3d097a0 retains ordered line, arrow and label aspects.
use crate::prs3d::{Prs3dArrow, Prs3dArrowAspect, Prs3dLineAspect, Prs3dTextAspect};
use rustycad_tkernel::{
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::*;
/// Native axis presentation overloads; `DsgPrsXYZAxisPresentation::add(&prs, &line, &dir, length, "X", &start, &end)?`.
pub struct DsgPrsXYZAxisPresentation;
impl DsgPrsXYZAxisPresentation {
    /// Inherit the line aspect for arrow and label; `Self::add(&prs, &line, &dir, 10., "X", &a, &b)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        line: &Handle<Prs3dLineAspect>,
        direction: &Direction3,
        length: f64,
        label: impl AsRef<[u8]>,
        first: &Point3,
        last: &Point3,
    ) -> Result<(), Failure> {
        let group = axis_segment(prs, line, first, last)?;
        Prs3dArrow::draw(
            &group,
            last,
            direction,
            std::f64::consts::PI / 180. * 10.,
            length / 10.,
        )?;
        axis_label(&group, label.as_ref(), last);
        Ok(())
    }
    /// Use independent arrow and text aspects; `Self::add_with_aspects(&prs, &line, &arrow, &text, &dir, 10., "X", &a, &b)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_with_aspects(
        prs: &Handle<dyn Graphic3dStructureSource>,
        line: &Handle<Prs3dLineAspect>,
        arrow: &Handle<Prs3dArrowAspect>,
        text: &Handle<Prs3dTextAspect>,
        direction: &Direction3,
        length: f64,
        label: impl AsRef<[u8]>,
        first: &Point3,
        last: &Point3,
    ) -> Result<(), Failure> {
        let group = axis_segment(prs, line, first, last)?;
        group.set_primitives_aspect(&arrow.aspect().cast());
        Prs3dArrow::draw(&group, last, direction, arrow.angle(), length / 10.)?;
        group.set_primitives_aspect(&text.aspect().cast());
        axis_label(&group, label.as_ref(), last);
        Ok(())
    }
}
fn axis_segment(
    prs: &Handle<dyn Graphic3dStructureSource>,
    line: &Handle<Prs3dLineAspect>,
    first: &Point3,
    last: &Point3,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    let group = prs.current_group()?;
    group.set_primitives_aspect(&line.aspect().cast());
    let vertices = Handle::new(Graphic3dArrayOfSegments::new(2, 0, 0)?);
    vertices.add_vertex_pnt(first)?;
    vertices.add_vertex_pnt(last)?;
    group.add_primitive_array(&vertices.cast(), true);
    Ok(group)
}
fn axis_label(group: &Handle<dyn Graphic3dGroup>, label: &[u8], position: &Point3) {
    let label = label.split(|byte| *byte == 0).next().unwrap_or_default();
    if label.is_empty() {
        return;
    }
    let text = Handle::new(Graphic3dText::new(1. / 81.));
    text.set_text_ascii(&AsciiString::from_bytes(label));
    text.set_position(position);
    group.add_text(&text, true);
}
