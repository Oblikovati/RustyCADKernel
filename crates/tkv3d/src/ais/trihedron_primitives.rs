//! Native part arrays remain cached until presentation recomputation.
use super::{
    AisTrihedronSource,
    trihedron::{TRI_AXIS_PARTS, TRI_PLANE_PARTS},
};
use crate::prs3d::*;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3};
use rustycad_tkservice::graphic3d::*;
/// Replace the native part cache; `ais_trihedron_update_primitives(&*trihedron, &datum, mode, origin, directions)?`.
pub fn ais_trihedron_update_primitives<O: AisTrihedronSource + ?Sized>(
    object: &O,
    datum: &Handle<Prs3dDatumAspect>,
    mode: Prs3dDatumMode,
    origin: Point3,
    directions: [Direction3; 3],
) -> Result<(), Failure> {
    object.trihedron_base().fields.write().primitives = std::array::from_fn(|_| Handle::default());
    let tips: [Point3; 3] = std::array::from_fn(|index| {
        Point3::from(
            origin.xyz() + directions[index].xyz() * datum.axis_length(TRI_AXIS_PARTS[index]),
        )
    });
    build_trihedron_origin(object, datum, mode, origin)?;
    for (index, part) in TRI_AXIS_PARTS.into_iter().enumerate() {
        build_trihedron_axis(
            object,
            datum,
            mode,
            part,
            origin,
            directions[index],
            tips[index],
        )?;
    }
    for (part, (first, second)) in TRI_PLANE_PARTS.into_iter().zip([(0, 1), (1, 2), (0, 2)]) {
        build_trihedron_plane(
            object,
            datum,
            part,
            [origin, tips[first], tips[second], origin],
        )?;
    }
    Ok(())
}
fn store_trihedron_array<O: AisTrihedronSource + ?Sized>(
    object: &O,
    part: Prs3dDatumParts,
    array: Handle<Graphic3dArrayOfPrimitives>,
) {
    object.trihedron_base().fields.write().primitives[part as usize] = array;
}
fn build_trihedron_origin<O: AisTrihedronSource + ?Sized>(
    object: &O,
    datum: &Handle<Prs3dDatumAspect>,
    mode: Prs3dDatumMode,
    origin: Point3,
) -> Result<(), Failure> {
    let part = Prs3dDatumParts::Origin;
    if !datum.draw_datum_part(part) {
        return Ok(());
    }
    let array = if mode == Prs3dDatumMode::WireFrame {
        let points = Handle::new(Graphic3dArrayOfPoints::new(1, 0)?);
        points.add_vertex_pnt(&origin)?;
        points.cast()
    } else {
        let mut transform = Transform3::default();
        transform.set_translation_part(Vector3::from(origin.xyz()));
        let facets = datum.attribute(Prs3dDatumAttribute::ShadingNumberOfFacettes) as i32;
        let radius = datum.axis_length(Prs3dDatumParts::XAxis)
            * datum.attribute(Prs3dDatumAttribute::ShadingOriginRadiusPercent);
        Prs3dToolSphere::create(radius, facets, facets, &transform)?.cast()
    };
    store_trihedron_array(object, part, array);
    Ok(())
}
fn build_trihedron_axis<O: AisTrihedronSource + ?Sized>(
    object: &O,
    datum: &Handle<Prs3dDatumAspect>,
    mode: Prs3dDatumMode,
    part: Prs3dDatumParts,
    origin: Point3,
    direction: Direction3,
    tip: Point3,
) -> Result<(), Failure> {
    if mode == Prs3dDatumMode::WireFrame {
        return build_trihedron_wire_axis(object, datum, part, origin, direction, tip);
    }
    let axis = Axis1::new(origin, direction);
    let arrow = Prs3dDatumAspect::arrow_part_for_axis(part);
    let length = datum.axis_length(part);
    let cone_length = length * datum.attribute(Prs3dDatumAttribute::ShadingConeLengthPercent);
    let facets = datum.attribute(Prs3dDatumAttribute::ShadingNumberOfFacettes) as i32;
    if datum.draw_datum_part(part) {
        let tube_length = if datum.draw_datum_part(arrow) {
            length - cone_length
        } else {
            length
        };
        let radius = length * datum.attribute(Prs3dDatumAttribute::ShadingTubeRadiusPercent);
        store_trihedron_array(
            object,
            part,
            Prs3dArrow::draw_shaded(&axis, radius, tube_length, 0., 0., facets)?.cast(),
        );
    }
    if datum.draw_datum_part(arrow) {
        let radius = length * datum.attribute(Prs3dDatumAttribute::ShadingConeRadiusPercent);
        store_trihedron_array(
            object,
            arrow,
            Prs3dArrow::draw_shaded(&axis, 0., length, radius, cone_length, facets)?.cast(),
        );
    }
    Ok(())
}
fn build_trihedron_wire_axis<O: AisTrihedronSource + ?Sized>(
    object: &O,
    datum: &Handle<Prs3dDatumAspect>,
    part: Prs3dDatumParts,
    origin: Point3,
    direction: Direction3,
    tip: Point3,
) -> Result<(), Failure> {
    if datum.draw_datum_part(part) {
        let segments = Handle::new(Graphic3dArrayOfSegments::new(2, 0, 0)?);
        segments.add_vertex_pnt(&origin)?;
        segments.add_vertex_pnt(&tip)?;
        store_trihedron_array(object, part, segments.cast());
    }
    let arrow = Prs3dDatumAspect::arrow_part_for_axis(part);
    if datum.draw_datum_part(arrow) {
        let length = datum.axis_length(part)
            * datum.attribute(Prs3dDatumAttribute::ShadingConeLengthPercent);
        let facets = datum.attribute(Prs3dDatumAttribute::ShadingNumberOfFacettes) as i32;
        let array = Prs3dArrow::draw_segments(
            &tip,
            &direction,
            datum.arrow_aspect().angle(),
            length,
            facets,
        )?;
        store_trihedron_array(object, arrow, array.cast());
    }
    Ok(())
}
fn build_trihedron_plane<O: AisTrihedronSource + ?Sized>(
    object: &O,
    datum: &Handle<Prs3dDatumAspect>,
    part: Prs3dDatumParts,
    vertices: [Point3; 4],
) -> Result<(), Failure> {
    if !datum.draw_datum_part(part) {
        return Ok(());
    }
    let lines = Handle::new(Graphic3dArrayOfPolylines::new(4, 0, 0, 0)?);
    for vertex in vertices {
        lines.add_vertex_pnt(&vertex)?;
    }
    store_trihedron_array(object, part, lines.cast());
    Ok(())
}
