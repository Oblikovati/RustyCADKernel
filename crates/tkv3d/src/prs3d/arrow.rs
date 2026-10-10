//! `Prs3d_Arrow`: arrow heads as segment cones or shaded tube-and-cone triangles.
use super::tool_quadric::{prs3d_quadric_triangles_nb, prs3d_quadric_vertices_nb};
use super::{Prs3dToolCylinder, Prs3dToolDisk, Prs3dToolQuadric};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point3, Transform3, Vector3, Xyz, dx, dy, dz};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL, Graphic3dArrayOfSegments, Graphic3dArrayOfTriangles,
    Graphic3dGroup, Graphic3dGroupMembers as _,
};
use std::f64::consts::PI;

/// The package class `Prs3d_Arrow`.
pub struct Prs3dArrow;

/// The reference direction: the axis along which `dir` has its smallest component.
fn least_aligned_axis(dir: &Direction3) -> Direction3 {
    let (x, y, z) = (dir.x().abs(), dir.y().abs(), dir.z().abs());
    if x <= y && x <= z {
        dx()
    } else if y <= z && y <= x {
        dy()
    } else {
        dz()
    }
}

/// The transformation from the frame at `location` along `direction` to the global frame.
fn frame_to_global(location: Point3, direction: Direction3) -> Result<Transform3, Failure> {
    let system = Axis3::from_normal(location, direction)?;
    let mut trsf = Transform3::default();
    trsf.set_transformation_between(system, Axis3::default());
    Ok(trsf)
}

impl Prs3dArrow {
    /// `DrawShaded`: a tube along the axis capped by a cone with its base disk, as triangles
    /// with normals; null when both parts are empty.
    /// Example: `Prs3dArrow::draw_shaded(&axis, 0.1, 10.0, 0.3, 1.0, 12)?`.
    pub fn draw_shaded(
        axis: &Axis1,
        tube_radius: f64,
        axis_length: f64,
        cone_radius: f64,
        cone_length: f64,
        nb_facettes: i32,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        let tube_length = 0.0f64.max(axis_length - cone_length);
        let has_tube = tube_radius > 0.0 && tube_length > 0.0;
        let has_cone = cone_radius > 0.0 && cone_length > 0.0;
        let nb_tris_tube = if has_tube {
            prs3d_quadric_triangles_nb(nb_facettes, 1)
        } else {
            0
        };
        let nb_tris_cone = if has_cone {
            2 * prs3d_quadric_triangles_nb(nb_facettes, 1)
        } else {
            0
        };
        let nb_tris = nb_tris_tube + nb_tris_cone;
        if nb_tris == 0 {
            return Ok(Handle::default());
        }
        let mut max_vertexs = 0;
        if nb_tris_tube > 0 {
            max_vertexs += prs3d_quadric_vertices_nb(nb_facettes, 1, true);
        }
        if nb_tris_cone > 0 {
            max_vertexs += 2 * prs3d_quadric_vertices_nb(nb_facettes, 1, true);
        }
        let array = Graphic3dArrayOfTriangles::new(
            max_vertexs,
            nb_tris * 3,
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
        )?;
        let mut array = Handle::new(array);
        if nb_tris_tube != 0 {
            let trsf = frame_to_global(axis.location(), axis.direction())?;
            Prs3dToolCylinder::new(tube_radius, tube_radius, tube_length, nb_facettes, 1)
                .fill_array(&mut array, &trsf)?;
        }
        if nb_tris_cone != 0 {
            let d = axis.direction();
            let offset = Vector3::new(
                d.x() * tube_length,
                d.y() * tube_length,
                d.z() * tube_length,
            );
            let trsf = frame_to_global(axis.location().translated(offset), d)?;
            Prs3dToolDisk::new(0.0, cone_radius, nb_facettes, 1).fill_array(&mut array, &trsf)?;
            Prs3dToolCylinder::new(cone_radius, 0.0, cone_length, nb_facettes, 1)
                .fill_array(&mut array, &trsf)?;
        }
        Ok(array)
    }
    /// `DrawSegments`: a cone of segments from the tip at `location` back along `dir`.
    /// Example: `Prs3dArrow::draw_segments(&tip, &dir, 0.2, 1.0, 15)?`.
    pub fn draw_segments(
        location: &Point3,
        dir: &Direction3,
        angle: f64,
        length: f64,
        nb_segments: i32,
    ) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
        let segments = Graphic3dArrayOfSegments::new(nb_segments + 1, 2 * (2 * nb_segments), 0)?;
        let center: Xyz = location.xyz() + dir.xyz() * (-length);
        let xyz_i = dir.crossed(least_aligned_axis(dir))?;
        let xyz_j = dir.xyz().crossed(xyz_i.xyz());
        segments.add_vertex_pnt(location)?;
        let tg = angle.tan();
        for vert in 1..=nb_segments {
            let step = 2.0 * PI / f64::from(nb_segments) * f64::from(vert - 1);
            let (cos, sin) = (step.cos(), step.sin());
            let point = Point3::new(
                center.x() + (cos * xyz_i.x() + sin * xyz_j.x()) * length * tg,
                center.y() + (cos * xyz_i.y() + sin * xyz_j.y()) * length * tg,
                center.z() + (cos * xyz_i.z() + sin * xyz_j.z()) * length * tg,
            );
            segments.add_vertex_pnt(&point)?;
        }
        add_cone_edges(&segments, nb_segments + 1)?;
        Ok(Handle::new(segments))
    }
    /// `Draw`: adds a 15-segment arrow cone to a group.
    /// Example: `Prs3dArrow::draw(&group, &tip, &dir, 0.2, 1.0)?`.
    pub fn draw(
        group: &Handle<dyn Graphic3dGroup>,
        location: &Point3,
        direction: &Direction3,
        angle: f64,
        length: f64,
    ) -> Result<(), Failure> {
        let primitives = Self::draw_segments(location, direction, angle, length, 15)?;
        if let Some(group) = group.get() {
            group.add_primitive_array(&primitives.cast(), true);
        }
        Ok(())
    }
}

/// The edges of the arrow cone: tip to each contour vertex, then the contour loop.
fn add_cone_edges(segments: &Graphic3dArrayOfSegments, nb_vertices: i32) -> Result<(), Failure> {
    let first_contour = 2;
    for vert in first_contour..=nb_vertices {
        segments.add_edge(1)?;
        segments.add_edge(vert)?;
    }
    segments.add_edge(nb_vertices)?;
    segments.add_edge(first_contour)?;
    for vert in first_contour..nb_vertices {
        segments.add_edge(vert)?;
        segments.add_edge(vert + 1)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_cone_layout() {
        let segments =
            Prs3dArrow::draw_segments(&Point3::new(0.0, 0.0, 1.0), &dz(), 0.2, 1.0, 4).unwrap();
        assert_eq!(segments.vertex_number(), 5);
        assert_eq!(segments.edge_number(), 16);
    }

    #[test]
    fn shaded_arrow_parts() {
        let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), dz());
        let both = Prs3dArrow::draw_shaded(&axis, 0.1, 10.0, 0.3, 1.0, 6).unwrap();
        assert_eq!(both.vertex_number(), 3 * 14);
        let none = Prs3dArrow::draw_shaded(&axis, 0.0, 1.0, 0.0, 1.0, 6).unwrap();
        assert!(none.is_null());
    }
}
