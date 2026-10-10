//! Native downcast order and inherited polyline geometry dispatch.
use super::{
    sensitive_presentation::SensitivePresentationGeometry, sensitive_presentation_solids::*,
    sensitive_presentation_triangles::*,
};
use crate::select3d::*;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Transform3;
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers, Graphic3dStructureSource};
pub(super) fn append_sensitive_geometry(
    prs: &Handle<dyn Graphic3dStructureSource>,
    geometry: &mut SensitivePresentationGeometry,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    location: Transform3,
) -> Result<(), Failure> {
    let flipper = entity.flipper();
    if !flipper.is_null() {
        prs.current_group()?
            .set_flipping_options(true, &flipper.ref_plane());
    }
    if let Some(value) = entity.cast::<Select3DSensitiveBox>().get() {
        return append_sensitive_box(geometry, value, location);
    }
    if let Some(value) = entity.cast::<Select3DSensitiveCylinder>().get() {
        return append_sensitive_cylinder(geometry, value, location);
    }
    append_sensitive_face_or_lines(geometry, entity, location)
}
fn append_sensitive_face_or_lines(
    geometry: &mut SensitivePresentationGeometry,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    location: Transform3,
) -> Result<(), Failure> {
    if let Some(value) = entity.cast::<Select3DSensitiveCircle>().get() {
        return append_sensitive_circle(
            geometry,
            value.radius(),
            location.multiplied(value.transformation()),
            0.,
        );
    }
    if let Some(value) = entity.cast::<Select3DSensitiveFace>().get() {
        let points = value.get_points();
        geometry.append_line(points.into_iter().map(|p| p.transformed(location)));
        return Ok(());
    }
    append_sensitive_lines(geometry, entity, location)
}
fn append_sensitive_lines(
    geometry: &mut SensitivePresentationGeometry,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    location: Transform3,
) -> Result<(), Failure> {
    if let Some(poly) = entity.cast::<dyn Select3DSensitivePolySource>().get() {
        append_sensitive_poly(geometry, poly.sensitive_poly_core(), location);
        return Ok(());
    }
    if let Some(wire) = entity.cast::<Select3DSensitiveWire>().get() {
        for edge in wire.get_edges() {
            append_sensitive_wire_edge(geometry, &edge, location);
        }
        return Ok(());
    }
    if let Some(segment) = entity.cast::<Select3DSensitiveSegment>().get() {
        append_sensitive_segment(geometry, segment, location);
        return Ok(());
    }
    append_sensitive_points_triangles(geometry, entity, location)
}
fn append_sensitive_points_triangles(
    geometry: &mut SensitivePresentationGeometry,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    location: Transform3,
) -> Result<(), Failure> {
    if let Some(point) = entity.cast::<Select3DSensitivePoint>().get() {
        geometry.points.append(point.point().transformed(location));
    } else if let Some(tri) = entity.cast::<Select3DSensitiveTriangulation>().get() {
        append_sensitive_triangulation(geometry, tri, location)?;
    } else if let Some(triangle) = entity.cast::<Select3DSensitiveTriangle>().get() {
        let (a, b, c) = triangle.points_3d();
        // OCCT 3d097a0 shrinks transformed vertices about the original center here.
        geometry.append_line(shrunk_sensitive_triangle(
            [a, b, c].map(|p| p.transformed(location)),
            triangle.center_3d().xyz(),
        ));
    }
    Ok(())
}
fn append_sensitive_wire_edge(
    geometry: &mut SensitivePresentationGeometry,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    location: Transform3,
) {
    if let Some(segment) = entity.cast::<Select3DSensitiveSegment>().get() {
        append_sensitive_segment(geometry, segment, location);
    } else if let Some(poly) = entity.cast::<dyn Select3DSensitivePolySource>().get() {
        append_sensitive_poly(geometry, poly.sensitive_poly_core(), location);
    }
}
fn append_sensitive_segment(
    geometry: &mut SensitivePresentationGeometry,
    segment: &Select3DSensitiveSegment,
    location: Transform3,
) {
    geometry.append_line([
        segment.start_point().transformed(location),
        segment.end_point().transformed(location),
    ]);
}
fn append_sensitive_poly(
    geometry: &mut SensitivePresentationGeometry,
    poly: &Select3DSensitivePolyCore,
    location: Transform3,
) {
    let (from, to) = poly.array_bounds();
    geometry.append_line((from..=to).map(|i| poly.get_point_3d(i).transformed(location)));
}
