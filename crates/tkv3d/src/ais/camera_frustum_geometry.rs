//! Native ordered face topology; preserve clockwise/anticlockwise winding and duplicated borders.
use super::AisCameraFrustum;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPrimitives, Graphic3dArrayOfSegments, Graphic3dArrayOfTriangles,
};
impl AisCameraFrustum {
    pub(super) fn fill_triangles(&self) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        if fields.triangles.is_null() {
            fields.triangles = Handle::new(Graphic3dArrayOfTriangles::new(8, 36, 0)?);
            fields.triangles.set_vertice(8, 0.0, 0.0, 0.0)?;
            append_frustum_faces(
                &fields.triangles,
                &[0, 1, 0, 1, 0, 1],
                &[0, 0, 1, 1, 1, 0],
                &[1, 0, 0, 0, 1, 1],
            )?;
        }
        replace_frustum_vertices(&fields.triangles, &fields.points)
    }
    pub(super) fn fill_borders(&self) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        if fields.borders.is_null() {
            fields.borders = Handle::new(Graphic3dArrayOfSegments::new(8, 48, 0)?);
            fields.borders.set_vertice(8, 0.0, 0.0, 0.0)?;
            let second = [0, 0, 0, 1, 1, 1, 1, 0];
            append_frustum_faces(&fields.borders, &[0, 1, 1, 1, 1, 0, 0, 0], &second, &second)?;
        }
        replace_frustum_vertices(&fields.borders, &fields.points)
    }
}
fn append_frustum_faces(
    array: &Graphic3dArrayOfPrimitives,
    first: &[i32],
    near: &[i32],
    far: &[i32],
) -> Result<(), Failure> {
    for axis in 0..3 {
        for side in 0..2 {
            append_frustum_face(array, axis, side, first, if side == 0 { near } else { far })?;
        }
    }
    Ok(())
}
fn append_frustum_face(
    array: &Graphic3dArrayOfPrimitives,
    axis: usize,
    side: i32,
    first: &[i32],
    second: &[i32],
) -> Result<(), Failure> {
    for (&u, &v) in first.iter().zip(second) {
        let mut shifts = [0; 3];
        shifts[axis] = side;
        shifts[(axis + 1) % 3] = u;
        shifts[(axis + 2) % 3] = v;
        array.add_edge(shifts[0] * 4 + shifts[1] * 2 + shifts[2] + 1)?;
    }
    Ok(())
}
fn replace_frustum_vertices(
    array: &Graphic3dArrayOfPrimitives,
    points: &[[f64; 3]; 9],
) -> Result<(), Failure> {
    for (index, [x, y, z]) in points.iter().take(8).enumerate() {
        array.set_vertice(index as i32 + 1, *x as f32, *y as f32, *z as f32)?;
    }
    Ok(())
}
