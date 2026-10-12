//! Native degenerate-triangle filtering uses face-local points before the location transform.
use super::shaded_shape_triangles::ShadedFaceMesh;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{gp::Point3, poly::Triangulation};
use rustycad_tkservice::graphic3d::*;

pub(super) fn append_shaded_indices(
    array: &Handle<Graphic3dArrayOfTriangles>,
    face: &ShadedFaceMesh,
    offset: i32,
) -> Result<(), Failure> {
    for index in 1..=face.mesh.nb_triangles() {
        let indices = oriented_shaded_triangle(face, index)?;
        if survives_shaded_triangle(&face.mesh, indices)? {
            array.add_edges3(
                indices[0] + offset,
                indices[1] + offset,
                indices[2] + offset,
            )?;
        }
    }
    Ok(())
}
fn survives_shaded_triangle(mesh: &Triangulation, indices: [i32; 3]) -> Result<bool, Failure> {
    let points = [
        mesh.node(indices[0])?,
        mesh.node(indices[1])?,
        mesh.node(indices[2])?,
    ];
    shaded_triangle_non_degenerate(points)
}
fn shaded_triangle_non_degenerate([a, b, c]: [Point3; 3]) -> Result<bool, Failure> {
    let [first, second, third] = [b.xyz() - a.xyz(), c.xyz() - b.xyz(), a.xyz() - c.xyz()];
    let precision = precision::square_confusion();
    if [first, second, third]
        .iter()
        .any(|edge| edge.square_modulus() <= precision)
    {
        return Ok(false);
    }
    let normal = first.normalized()? ^ second.normalized()?;
    Ok(normal.square_modulus() > precision)
}

fn oriented_shaded_triangle(face: &ShadedFaceMesh, index: i32) -> Result<[i32; 3], Failure> {
    let (a, b, c) = face.mesh.triangle(index)?.get();
    let indices = if face.face.orientation() == Orientation::Reversed {
        [a, c, b]
    } else {
        [a, b, c]
    };
    Ok(indices)
}
