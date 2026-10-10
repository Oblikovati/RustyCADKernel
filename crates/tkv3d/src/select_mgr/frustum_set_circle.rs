//! Boundary circles retain the native surface-intersection and line-normal arithmetic.
use super::frustum_set_boundary::{
    frustum_set_face, frustum_set_point_in_triangle, frustum_set_segments_cross,
};
use super::*;
use rustycad_tkernel::{
    precision::confusion,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomCircle, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkmath::gp::{Axis2, Direction3, Line3, Point3, Transform3, Vector3};
impl SelectMgrTriangularFrustumSet {
    pub(super) fn frustum_set_circle_overlap(
        &self,
        radius: f64,
        transform: Transform3,
        filled: bool,
        mut inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        self.frustum_set_circle_projection(transform)?;
        if !self.frustum_set_any_child(|child| {
            child.overlaps_circle(radius, &transform, filled, inside.as_deref_mut())
        })? {
            if let Some(inside) = inside {
                *inside &= false;
            }
            return Ok(false);
        }
        if *self.allow_overlap.read()
            || !self.frustum_set_crosses_circle(radius, transform, filled)?
        {
            return Ok(true);
        }
        if !filled && let Some(inside) = inside {
            *inside &= false;
        }
        Ok(false)
    }
    fn frustum_set_circle_projection(&self, transform: Transform3) -> Result<(), Failure> {
        let center = Point3::default().transformed(transform);
        let (plane, normal) = self.frustum_set_near_plane()?;
        // The native unused projection can still fail while constructing the plane.
        let _projected =
            super::frustum_set_cylinder::frustum_set_plane_projection(center, plane, normal)?;
        Ok(())
    }
    fn frustum_set_crosses_circle(
        &self,
        radius: f64,
        transform: Transform3,
        filled: bool,
    ) -> Result<bool, Failure> {
        let center = Point3::from(transform.translation_part());
        // Native constructs this unused circle first, retaining its radius failures.
        let _circle = GeomCircle::new(Axis2::default().transformed(transform)?, radius)?;
        let normal = Direction3::Z.transformed(transform);
        let plane: Handle<dyn GeomSurface> =
            Handle::new(GeomPlane::from_point(center, normal)?).cast();
        let boundary = self.boundary_points.read();
        for index in boundary.lower()..boundary.lower() + boundary.len() as i32 / 2 {
            let face = frustum_set_face(&boundary, index)?;
            if frustum_set_circle_face(center, radius, filled, &plane, face)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
fn frustum_set_circle_face(
    center: Point3,
    radius: f64,
    filled: bool,
    plane: &Handle<dyn GeomSurface>,
    face: [Point3; 4],
) -> Result<bool, Failure> {
    let Some(line) = frustum_set_face_line(plane, face)? else {
        return Ok(false);
    };
    let distance = line.distance(center);
    if distance > radius {
        return Ok(false);
    }
    let [first, last] = frustum_set_circle_crossings(center, radius, line, distance)?;
    Ok(frustum_set_chord_hits_face([first, last], face, filled))
}
fn frustum_set_circle_crossings(
    center: Point3,
    radius: f64,
    line: Line3,
    distance: f64,
) -> Result<[Point3; 2], Failure> {
    let normal = line.normal(center)?;
    let cross = center.translated(Vector3::from(
        normal.direction().xyz().reversed() * distance,
    ));
    let offset = (radius * radius - distance * distance).sqrt();
    Ok([
        cross.translated(Vector3::from(line.direction().xyz() * offset)),
        cross.translated(Vector3::from(line.direction().xyz().reversed() * offset)),
    ])
}
fn frustum_set_chord_hits_face(chord: [Point3; 2], face: [Point3; 4], filled: bool) -> bool {
    for point in chord {
        if frustum_set_point_in_triangle(point, [face[0], face[1], face[2]])
            || frustum_set_point_in_triangle(point, [face[0], face[2], face[3]])
        {
            return true;
        }
    }
    filled
        || [(0, 1), (1, 2), (2, 3), (0, 3)]
            .iter()
            .any(|&(first, last)| frustum_set_segments_cross(chord, [face[first], face[last]]))
}

fn frustum_set_face_line(
    plane: &Handle<dyn GeomSurface>,
    face: [Point3; 4],
) -> Result<Option<Line3>, Failure> {
    let normal = Direction3::from_vector(
        Vector3::between(face[0], face[1]).crossed(Vector3::between(face[0], face[2])),
    )?;
    let boundary: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_point(face[0], normal)?).cast();
    let intersection =
        GeomIntIntSS::with_surfaces(plane, &boundary, confusion(), GeomIntIntSSApprox::default())?;
    if !intersection.is_done() || intersection.nb_lines() == 0 {
        return Ok(None);
    }
    let line = intersection.line(1)?.cast::<GeomLine>();
    let line = line.get().ok_or_else(frustum_set_missing_line)?.lin();
    Ok(Some(line))
}

fn frustum_set_missing_line() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "plane intersection line=null; expected Geom_Line",
    )
}
