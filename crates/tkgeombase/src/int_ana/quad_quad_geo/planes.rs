// Rust translation of IntAna_QuadQuadGeo::Perform(gp_Pln, gp_Pln / gp_Cylinder / gp_Sphere)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The plane-plane, plane-cylinder and plane-sphere intersections.
use super::QuadQuadGeo;
use crate::int_ana::{IntConicQuad, ResultType};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::gp::{Cylinder3, Direction3, Line3, Plane3, Point3, Sphere3, Vector3, Xyz};
use std::f64::consts::PI;

impl QuadQuadGeo {
    /// Perform(gp_Pln, gp_Pln, TolAng, Tol): same or empty when the normals'
    /// cross product is at most `tol_ang` (same when both plane distances are within
    /// `tol`), else one line; below 2e-6 rad its origin, when off the planes by more
    /// than 1e-12, is refined by two line-plane intersections.
    /// `inter.perform_planes(p1, p2, 1e-12, 1e-7)?`.
    pub fn perform_planes(
        &mut self,
        first: Plane3,
        second: Plane3,
        tol_ang: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        self.param2bis = 0.;
        let [a1, b1, c1, d1] = first.coefficients()?;
        let [a2, b2, c2, d2] = second.coefficients()?;
        let (vn1, vn2) = (Vector3::new(a1, b1, c1), Vector3::new(a2, b2, c2));
        let vd: Vector3 = vn1.crossed(vn2);
        let (loc1, loc2) = (first.location(), second.location());
        let dist1: f64 = a2 * loc1.x() + b2 * loc1.y() + c2 * loc1.z() + d2;
        let dist2: f64 = a1 * loc2.x() + b1 * loc2.y() + c1 * loc2.z() + d1;
        let mvd: f64 = vd.magnitude();
        if mvd <= tol_ang {
            // The normals are collinear: the planes are the same or parallel.
            self.typeres = if dist1.abs() <= tol && dist2.abs() <= tol {
                ResultType::Same
            } else {
                ResultType::Empty
            };
            self.done = true;
            return Ok(());
        }
        let denom: f64 = a1 * a2 + b1 * b2 + c1 * c2;
        let ddenom: f64 = 1. - denom * denom;
        let denom: f64 = if ddenom.abs() <= 1.0e-16 {
            1.0e-16
        } else {
            ddenom
        };
        let (par1, par2) = (dist1 / denom, -dist2 / denom);
        let (inter1, inter2) = (vn1.crossed(vd), vn2.crossed(vd));
        let x1 = Xyz::new(
            loc1.x() + par1 * inter1.x(),
            loc1.y() + par1 * inter1.y(),
            loc1.z() + par1 * inter1.z(),
        );
        let x2 = Xyz::new(
            loc2.x() + par2 * inter2.x(),
            loc2.y() + par2 * inter2.y(),
            loc2.z() + par2 * inter2.z(),
        );
        self.pt1 = Point3::new(
            (x1.x() + x2.x()) * 0.5,
            (x1.y() + x2.y()) * 0.5,
            (x1.z() + x2.z()) * 0.5,
        );
        self.dir1 = Direction3::from_vector(vd)?;
        self.typeres = ResultType::Line;
        self.nbint = 1;
        // Below 1e-4 degree the origin is computed with an error (~1e-4) that
        // cannot be considered small: refine it when possible.
        if mvd < 2.0e-6 {
            let off1: f64 = a1 * self.pt1.x() + b1 * self.pt1.y() + c1 * self.pt1.z() + d1;
            let off2: f64 = a2 * self.pt1.x() + b2 * self.pt1.y() + c2 * self.pt1.z() + d2;
            if (off1.abs() > 1.0e-12 || off2.abs() > 1.0e-12)
                && !self.refine_line_origin(vn1, (first, second), tol_ang, tol)?
            {
                return Ok(());
            }
        }
        self.done = true;
        Ok(())
    }

    /// Moves the line origin onto the first plane along its normal, then onto the
    /// second plane along the normal of the line in the first plane; false (not
    /// done) when an intersection fails or the second line is parallel.
    fn refine_line_origin(
        &mut self,
        normal1: Vector3,
        (first, second): (Plane3, Plane3),
        tol_ang: f64,
        tol: f64,
    ) -> Result<bool, Failure> {
        let mut icq = IntConicQuad::new();
        let dn1: Direction3 = Direction3::from_vector(normal1)?;
        icq.perform_line_plane(Line3::new(self.pt1, dn1), first, tol_ang, tol, 0.0)?;
        if !icq.is_done() {
            return Ok(false);
        }
        let pnt1: Point3 = icq.point(1)?;
        let dl2: Direction3 = self.dir1.crossed(dn1)?;
        icq.perform_line_plane(Line3::new(pnt1, dl2), second, tol_ang, tol, 0.0)?;
        if !icq.is_done() || icq.is_parallel()? {
            return Ok(false);
        }
        self.pt1 = icq.point(1)?;
        Ok(true)
    }

    /// Perform(gp_Pln, gp_Cylinder, Tolang, Tol, H): an axis parallel to the plane
    /// (evaluated over the height `h` when known) gives one tangent line, two lines
    /// or nothing; otherwise the axis point on the plane centers a circle (when the
    /// axis is normal within `tol / radius`) or an ellipse. A nearly parallel axis
    /// relaxes the tolerances and takes the line directions from a second section
    /// 100 further along the axis. `inter.perform_plane_cylinder(p, c, 1e-12, 1e-7, 0.0)?`.
    pub fn perform_plane_cylinder(
        &mut self,
        plane: Plane3,
        cylinder: Cylinder3,
        tolang: f64,
        tol: f64,
        h: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        self.param2bis = 0.0;
        let axec: Line3 = Line3::from_axis(cylinder.axis());
        let normp: Xyz = plane.axis().direction().xyz();
        let (tolang2, toltang, newparams) = relaxed_tolerances(axec, normp, tolang, tol, h)?;
        self.nbint = 0;
        let inter = IntConicQuad::line_plane(axec, plane, tolang2, toltang, h)?;
        if inter.is_parallel()? {
            self.parallel_cylinder_lines(plane, cylinder, tol, newparams)?;
        } else {
            self.cylinder_section(plane, cylinder, tol, inter.point(1)?)?;
        }
        self.done = true;
        Ok(())
    }

    /// The lines of a cylinder whose axis is parallel to the plane.
    fn parallel_cylinder_lines(
        &mut self,
        plane: Plane3,
        cylinder: Cylinder3,
        tol: f64,
        newparams: bool,
    ) -> Result<(), Failure> {
        let [a, b, c, d] = plane.coefficients()?;
        let axec: Line3 = Line3::from_axis(cylinder.axis());
        let origin: Point3 = axec.location();
        let dist: f64 = a * origin.x() + b * origin.y() + c * origin.z() + d;
        let radius: f64 = cylinder.radius();
        let normp: Xyz = plane.axis().direction().xyz();
        self.typeres = ResultType::Line;
        let omega = Xyz::new(
            origin.x() - dist * a,
            origin.y() - dist * b,
            origin.z() - dist * c,
        );
        // A second section 100 further along the axis gives the line directions.
        let shifted: Xyz = origin.xyz() + 100. * axec.direction().xyz();
        let distt: f64 = a * shifted.x() + b * shifted.y() + c * shifted.z() + d;
        let omega1 = Xyz::new(
            shifted.x() - distt * a,
            shifted.y() - distt * b,
            shifted.z() - distt * c,
        );
        if (dist.abs() - radius).abs() < tol {
            self.nbint = 1;
            self.pt1.set_xyz(omega);
            self.dir1 = if newparams {
                Direction3::from_vector(Vector3::between(self.pt1, Point3::from(omega1)))?
            } else {
                axec.direction()
            };
        } else if dist.abs() < radius {
            self.nbint = 2;
            let hh: f64 = (radius * radius - dist * dist).sqrt();
            let axey: Xyz = axec.direction().xyz().crossed(normp);
            self.pt1.set_xyz(omega - hh * axey);
            self.pt2.set_xyz(omega + hh * axey);
            if newparams {
                let sqrt_arg: f64 = radius * radius - distt * distt;
                let ht: f64 = if sqrt_arg > 0. { sqrt_arg.sqrt() } else { 0. };
                let (ppt1, ppt2) = (
                    Point3::from(omega1 - ht * axey),
                    Point3::from(omega1 + ht * axey),
                );
                self.dir1 = Direction3::from_vector(Vector3::between(self.pt1, ppt1))?;
                self.dir2 = Direction3::from_vector(Vector3::between(self.pt2, ppt2))?;
            } else {
                self.dir1 = axec.direction();
                self.dir2 = axec.direction();
            }
        } else {
            self.typeres = ResultType::Empty;
        }
        Ok(())
    }

    /// The circle or ellipse centered at `center`, the axis point on the plane.
    fn cylinder_section(
        &mut self,
        plane: Plane3,
        cylinder: Cylinder3,
        tol: f64,
        center: Point3,
    ) -> Result<(), Failure> {
        let axis_direction: Direction3 = cylinder.axis().direction();
        let normp: Xyz = plane.axis().direction().xyz();
        let radius: f64 = cylinder.radius();
        self.nbint = 1;
        let axey: Xyz = normp.crossed(axis_direction.xyz());
        let sint: f64 = axey.modulus();
        self.pt1 = center;
        if sint < tol / radius {
            // A circle with the X and Y axes of the cylinder.
            self.typeres = ResultType::Circle;
            self.dir1 = axis_direction;
            self.dir2 = cylinder.position().x_direction();
            self.param1 = radius;
        } else {
            self.typeres = ResultType::Ellipse;
            let cost: f64 = axis_direction.xyz().dot(normp).abs();
            let axex: Xyz = axey.crossed(normp);
            self.dir1.set_xyz(normp)?;
            self.dir2.set_xyz(axex)?;
            self.param1 = radius / cost;
            self.param1bis = radius;
        }
        Ok(())
    }

    /// Perform(gp_Pln, gp_Sphere): a point (the projected center) when the
    /// distance equals the radius within Epsilon(radius), a circle when smaller,
    /// else empty. `inter.perform_plane_sphere(plane, sphere)?`.
    pub fn perform_plane_sphere(&mut self, plane: Plane3, sphere: Sphere3) -> Result<(), Failure> {
        self.done = false;
        self.nbint = 0;
        self.typeres = ResultType::Empty;
        let [a, b, c, d] = plane.coefficients()?;
        let center: Point3 = sphere.location();
        let (x, y, z) = (center.x(), center.y(), center.z());
        let radius: f64 = sphere.radius();
        let dist: f64 = a * x + b * y + c * z + d;
        if (dist.abs() - radius).abs() < epsilon(radius) {
            self.nbint = 1;
            self.typeres = ResultType::Point;
            self.pt1 = Point3::new(x - dist * a, y - dist * b, z - dist * c);
        } else if dist.abs() < radius {
            self.nbint = 1;
            self.typeres = ResultType::Circle;
            self.pt1 = Point3::new(x - dist * a, y - dist * b, z - dist * c);
            self.dir1 = plane.axis().direction();
            if !plane.direct()? {
                self.dir1.reverse();
            }
            self.dir2 = plane.position().x_direction();
            self.param1 = (radius * radius - dist * dist).sqrt();
        }
        self.param2bis = 0.0;
        self.done = true;
        Ok(())
    }
}

/// The angular and linear tolerances of the plane-cylinder intersection: an axis
/// within PI/4 of the plane whose deviation `sinda` is close to `tol` (or small
/// over the height `h`) is treated as parallel with the tolerances
/// `(2 sinda, max(tol, 1.01 sinda h))`; the flag tells that they changed.
fn relaxed_tolerances(
    axec: Line3,
    normp: Xyz,
    tolang: f64,
    tol: f64,
    h: f64,
) -> Result<(f64, f64, bool), Failure> {
    let (ldv, npv) = (Vector3::from(axec.direction()), Vector3::from(normp));
    let da: f64 = ldv.angle(npv)?.abs();
    if da.is_nan() || da <= (PI / 4.) {
        return Ok((tolang, tol, false));
    }
    let dangle: f64 = (ldv.angle(npv)?.abs() - PI / 2.).abs();
    if dangle <= tolang || dangle.is_nan() {
        return Ok((tolang, tol, false));
    }
    let sinda: f64 = dangle.sin().abs();
    let dif: f64 = (sinda - tol).abs();
    // A small deviation of the axis over the known face height H means the axis
    // is effectively parallel: use the line solution, not a degenerate ellipse.
    if dif < tol || (h > 0. && sinda * h < 2. * tol) {
        // The relaxed linear tolerance keeps IntAna_IntConicQuad from rejecting
        // the parallel status over H.
        return Ok((sinda * 2., std_max(tol, sinda * h * 1.01), true));
    }
    Ok((tolang, tol, false))
}
