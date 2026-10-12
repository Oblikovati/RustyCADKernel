//! Native shaded axes and center sphere with screen-aligned datum labels.
use super::{AisViewCubeGeometry, AisViewCubePresentationSettings, AisViewCubePresentationSource};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::prs3d::*;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
    tcollection::ExtendedString,
};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
pub(super) fn cube_axes_location(geometry: &AisViewCubeGeometry) -> Point3 {
    let value = -(geometry.size * 0.5 + geometry.facet_extension + geometry.axes_padding);
    Point3::new(value, value, value)
}
pub(super) fn submit_cube_axes<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    let settings = object.view_cube_presentation_base().settings.read().clone();
    if !settings.draw_axes {
        return Ok(());
    }
    let location = cube_axes_location(&settings.geometry);
    let datum = object.attributes().datum_aspect();
    if datum.is_null() {
        return Err(rustycad_tkernel::standard::exceptions::null_object(
            "AIS_ViewCube datum aspect=null; expected own aspect or interactive-context drawer link",
        ));
    }
    for index in 0..3 {
        submit_cube_datum_axis(prs, &datum, &settings, index, location)?;
    }
    submit_cube_axes_center(prs, &settings.geometry, location)
}
fn submit_cube_datum_axis(
    prs: &Handle<dyn Graphic3dStructureSource>,
    datum: &Prs3dDatumAspect,
    settings: &AisViewCubePresentationSettings,
    index: usize,
    location: Point3,
) -> Result<(), Failure> {
    let part = cube_datum_axis_part(index);
    if !datum.draw_datum_part(part) {
        return Ok(());
    }
    let direction = [Direction3::X, Direction3::Y, Direction3::Z][index];
    let axis = Axis1::new(location, direction);
    submit_cube_axis(prs, datum, &settings.geometry, part, &axis)?;
    submit_optional_cube_axis_label(prs, datum, part, &axis, settings)?;
    Ok(())
}

fn cube_datum_axis_part(index: usize) -> Prs3dDatumParts {
    [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ][index]
}

fn cube_axis_length(geometry: &AisViewCubeGeometry) -> f64 {
    geometry.size + 2.0 * geometry.facet_extension + geometry.axes_padding
}
fn submit_cube_axis(
    prs: &Handle<dyn Graphic3dStructureSource>,
    datum: &Prs3dDatumAspect,
    geometry: &AisViewCubeGeometry,
    part: Prs3dDatumParts,
    axis: &Axis1,
) -> Result<(), Failure> {
    let triangles = cube_axis_shaded_triangles(geometry, axis)?;
    super::view_cube_submission::submit_cube_triangles(
        prs,
        &triangles,
        &datum.shading_aspect(part).aspect(),
    )
}
fn submit_cube_axis_label(
    prs: &Handle<dyn Graphic3dStructureSource>,
    datum: &Prs3dDatumAspect,
    part: Prs3dDatumParts,
    axis: &Axis1,
    geometry: &AisViewCubeGeometry,
    label: &rustycad_tkernel::tcollection::AsciiString,
) -> Result<(), Failure> {
    let point = cube_axis_label_point(geometry, axis);
    Prs3dText::draw(
        &prs.new_group()?,
        &datum.text_aspect(part),
        &ExtendedString::from_c_string(label.as_bytes(), false),
        &point,
    );
    Ok(())
}
fn submit_cube_axes_center(
    prs: &Handle<dyn Graphic3dStructureSource>,
    geometry: &AisViewCubeGeometry,
    location: Point3,
) -> Result<(), Failure> {
    let shading = Handle::new(Prs3dShadingAspect::new()?);
    shading.set_color(
        Color::named(NamedColor::WHITE),
        AspectTypeOfFacingModel::BothSide,
    )?;
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(location.x(), location.y(), location.z()));
    let mut triangles = Handle::default();
    Prs3dToolSphere::new(geometry.axes_sphere_radius, 20, 20)
        .fill_array(&mut triangles, &transform)?;
    super::view_cube_submission::submit_cube_triangles(prs, &triangles, &shading.aspect())
}

fn submit_optional_cube_axis_label(
    prs: &Handle<dyn Graphic3dStructureSource>,
    datum: &Prs3dDatumAspect,
    part: Prs3dDatumParts,
    axis: &Axis1,
    settings: &AisViewCubePresentationSettings,
) -> Result<(), Failure> {
    let label = settings
        .axes_labels
        .seek(&part)
        .filter(|label| !label.is_empty());
    if datum.to_draw_labels()
        && let Some(label) = label
    {
        submit_cube_axis_label(prs, datum, part, axis, &settings.geometry, label)?;
    }
    Ok(())
}

fn cube_axis_shaded_triangles(
    geometry: &AisViewCubeGeometry,
    axis: &Axis1,
) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
    let length = cube_axis_length(geometry);
    Prs3dArrow::draw_shaded(
        axis,
        geometry.axes_radius,
        length,
        geometry.axes_cone_radius,
        0.2 * length,
        20,
    )
}

fn cube_axis_label_point(geometry: &AisViewCubeGeometry, axis: &Axis1) -> Point3 {
    let length = cube_axis_length(geometry);
    let offset = length + 0.2 * length;
    let direction = axis.direction();
    axis.location().translated(Vector3::new(
        direction.x() * offset,
        direction.y() * offset,
        direction.z() * offset,
    ))
}
