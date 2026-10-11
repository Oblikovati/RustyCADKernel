//! Native rectangle and incrementally spaced isolines retain float-buffer rounding.
use crate::prs3d::Prs3dPlaneAspect;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkservice::graphic3d::*;

pub(super) fn draw_plane_edges(
    group: &Handle<dyn Graphic3dGroup>,
    plane: &GeomPlane,
    aspect: &Handle<Prs3dPlaneAspect>,
    extents: [f64; 2],
) -> Result<(), Failure> {
    if !aspect.display_edges() {
        return Ok(());
    }
    group.set_primitives_aspect(&aspect.edges_aspect().aspect().cast());
    let [x, y] = extents;
    let vertices = Handle::new(Graphic3dArrayOfPolylines::new(5, 0, 0, 0)?);
    for (u, v) in [(-x, y), (x, y), (x, -y), (-x, -y), (-x, y)] {
        vertices.add_vertex_pnt(&plane.value(u, v)?)?;
    }
    group.add_primitive_array(&vertices.cast(), true);
    Ok(())
}
pub(super) fn draw_plane_isos(
    group: &Handle<dyn Graphic3dGroup>,
    plane: &GeomPlane,
    aspect: &Handle<Prs3dPlaneAspect>,
    extents: [f64; 2],
) -> Result<(), Failure> {
    if !aspect.display_iso() {
        return Ok(());
    }
    group.set_primitives_aspect(&aspect.iso_aspect().aspect().cast());
    let spacing = aspect.iso_distance();
    let counts = [
        iso_count(extents[0], spacing)?,
        iso_count(extents[1], spacing)?,
    ];
    let vertices = iso_array(counts)?;
    append_isos(&vertices, plane, extents, spacing, counts[0], false)?;
    append_isos(&vertices, plane, extents, spacing, counts[1], true)?;
    group.add_primitive_array(&vertices.cast(), true);
    Ok(())
}
fn iso_array(counts: [i32; 2]) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
    let capacity = counts[0]
        .checked_add(counts[1])
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(|| {
            Failure::new(&format!(
                "plane iso counts {counts:?}: expected signed 32-bit vertex capacity"
            ))
        })?;
    Ok(Handle::new(Graphic3dArrayOfSegments::new(capacity, 0, 0)?))
}
fn iso_count(extent: f64, spacing: f64) -> Result<i32, Failure> {
    let ratio = (2. * extent).abs() / spacing;
    if !ratio.is_finite()
        || ratio.trunc() < f64::from(i32::MIN) + 1.
        || ratio.trunc() > f64::from(i32::MAX)
    {
        return Err(Failure::new(&format!(
            "plane iso extent {extent}, spacing {spacing}, ratio {ratio}: expected finite signed 32-bit count before subtraction"
        )));
    }
    Ok(ratio as i32 - 1)
}
fn append_isos(
    vertices: &Handle<Graphic3dArrayOfSegments>,
    plane: &GeomPlane,
    extents: [f64; 2],
    spacing: f64,
    count: i32,
    horizontal: bool,
) -> Result<(), Failure> {
    let axis = usize::from(horizontal);
    let mut current = -extents[axis] + spacing;
    for _ in 0..count {
        let ends = if horizontal {
            [(extents[0], current), (-extents[0], current)]
        } else {
            [(current, extents[1]), (current, -extents[1])]
        };
        for (u, v) in ends {
            vertices.add_vertex_pnt(&plane.value(u, v)?)?;
        }
        current += spacing;
    }
    Ok(())
}
