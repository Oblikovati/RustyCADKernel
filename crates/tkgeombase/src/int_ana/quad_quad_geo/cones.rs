// Rust translation of IntAna_QuadQuadGeo::Perform(gp_Cone, gp_Cone, double)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cone-cone intersection: coaxial cones, parallel axes with equal angles,
//! a common apex, or a common generatrix.
use super::QuadQuadGeo;
use super::axe_operator::AxeOperator;
use super::cylinders::offset;
use super::estim_dist::estim_dist;
use crate::int_ana::ResultType;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{
    Cone3, Direction2, Direction3, Line3, Plane3, Point2, Point3, Vector2, Vector3,
};
use rustycad_tkmath::{elclib, elslib};
use std::f64::consts::PI;

/// Apexes closer than this (1e-10 along the axis) coincide.
const TOL_APEX_CONF: f64 = 1.0e-10;

impl QuadQuadGeo {
    /// Perform(gp_Cone, gp_Cone, Tol): coaxial cones give a point, one or two
    /// circles or same; parallel axes with equal angles (the angular tolerance
    /// widened from EstimDist for distant axes) the section by the plane of the
    /// solution; a common apex (within `tol`) one or two lines or empty; a common
    /// generatrix the section by its characteristic plane; otherwise no geometric
    /// solution. `inter.perform_cones(k1, k2, 1e-7)?`.
    pub fn perform_cones(&mut self, first: Cone3, second: Cone3, tol: f64) -> Result<(), Failure> {
        self.done = true;
        let tg1: f64 = first.semi_angle().tan();
        let mut tg2: f64 = second.semi_angle().tan();
        if (tg1 * tg2) < 0. {
            tg2 = -tg2;
        }
        let tol2: f64 = tol * tol;
        let square_apexes: f64 = first.apex().square_distance(&second.apex());
        let axes = AxeOperator::new(first.axis(), second.axis())?;
        let tol_ang: f64 = self.cone_angular_tolerance(first, second, (tg1, tg2), &axes, tol)?;
        if axes.same() {
            self.coaxial_cones(first, second, (tg1, tg2));
            return Ok(());
        }
        if ((tg1 - tg2).abs() < tol_ang) && axes.parallel() {
            return self.parallel_cones(first, second, tg1, axes.distance(), tol);
        }
        if square_apexes < tol2 {
            return self.common_apex_cones(first, second, tol);
        }
        if axes.intersect() {
            return self.common_generatrix_cones(first, second, &axes, tol);
        }
        self.typeres = ResultType::NoGeometricSolution;
        Ok(())
    }

    /// The angular tolerance of equal angles: the cone precision, widened to
    /// `tol / EstimDist` (at most `tol`) for parallel axes farther than `100 tol`.
    fn cone_angular_tolerance(
        &self,
        first: Cone3,
        second: Cone3,
        (tg1, tg2): (f64, f64),
        axes: &AxeOperator,
        tol: f64,
    ) -> Result<f64, Failure> {
        let mut tol_ang: f64 = self.my_epsilon_angle_cone;
        let equal_parallel: bool = (tg1 - tg2).abs() < tol && axes.parallel();
        let distant: bool = axes.distance() > 100. * tol;
        if !(equal_parallel && distant) {
            return Ok(tol_ang);
        }
        let min_sol_dist: f64 = estim_dist(first, second)?;
        if min_sol_dist < epsilon(1.) {
            tol_ang = tol;
        } else {
            tol_ang = std_max(self.my_epsilon_angle_cone, tol / min_sol_dist);
            tol_ang = std_min(tol_ang, tol);
        }
        Ok(tol_ang)
    }

    /// Coaxial cones: two circles (a point for a common apex) for different
    /// angles; same or one circle midway between the apexes for equal ones.
    fn coaxial_cones(&mut self, first: Cone3, second: Cone3, (tg1, tg2): (f64, f64)) {
        let p: Point3 = first.apex();
        let d_axis: Direction3 = first.position().direction();
        let d: f64 = Vector3::from(d_axis).dot(Vector3::between(p, second.apex()));
        if (tg1 - tg2).abs() > self.my_epsilon_angle_cone {
            if d.abs() < TOL_APEX_CONF {
                self.typeres = ResultType::Point;
                self.nbint = 1;
                self.pt1 = p;
                return;
            }
            let x: f64 = (d * tg2) / (tg1 + tg2);
            self.pt1 = offset(p, x, d_axis);
            self.param1 = (x * tg1).abs();
            let x: f64 = (d * tg2) / (tg2 - tg1);
            self.pt2 = offset(p, x, d_axis);
            self.param2 = (x * tg1).abs();
            self.dir1 = d_axis;
            self.dir2 = d_axis;
            self.nbint = 2;
            self.typeres = ResultType::Circle;
        } else if d.abs() < TOL_APEX_CONF {
            self.typeres = ResultType::Same;
        } else {
            self.typeres = ResultType::Circle;
            self.nbint = 1;
            let x: f64 = d * 0.5;
            self.pt1 = offset(p, x, d_axis);
            self.param1 = (x * tg1).abs();
            self.dir1 = d_axis;
        }
    }

    /// Parallel axes and equal angles: the section of the first cone by the plane
    /// through the meeting point P1 of the generatrices, orthogonal to the plane
    /// of the axes.
    fn parallel_cones(
        &mut self,
        first: Cone3,
        second: Cone3,
        tg1: f64,
        dist: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let (apex1, apex2) = (first.apex(), second.apex());
        let da1: Direction3 = first.position().direction();
        let o1o2 = Vector3::between(apex1, apex2);
        // The vector is normalized before the projection.
        let o1o2n: Direction3 = Direction3::from_vector(o1o2)?;
        let o1o2_da1: f64 = Vector3::from(da1).dot(Vector3::from(o1o2n));
        let db1: Direction3 = Direction3::from_vector(Vector3::new(
            o1o2n.x() - o1o2_da1 * da1.x(),
            o1o2n.y() - o1o2_da1 * da1.y(),
            o1o2n.z() - o1o2_da1 * da1.z(),
        ))?;
        let y_o1o2: f64 = o1o2.dot(Vector3::from(da1));
        let abstg1: f64 = tg1.abs();
        let x2: f64 = (dist / abstg1 - y_o1o2) * 0.5;
        let x1: f64 = x2 + y_o1o2;
        let p1 = Point3::new(
            apex1.x() + x1 * (da1.x() + abstg1 * db1.x()),
            apex1.y() + x1 * (da1.y() + abstg1 * db1.y()),
            apex1.z() + x1 * (da1.z() + abstg1 * db1.z()),
        );
        let middle = Point3::new(
            0.5 * (apex1.x() + apex2.x()),
            0.5 * (apex1.y() + apex2.y()),
            0.5 * (apex1.z() + apex2.z()),
        );
        let normal: Direction3 = da1
            .crossed(db1)?
            .crossed(Direction3::from_vector(Vector3::between(p1, middle))?)?;
        let section = QuadQuadGeo::plane_cone(Plane3::new(p1, normal)?, first, tol, tol)?;
        self.copy_cone_section(&section, false)
    }

    /// Copies the ellipse, circle, hyperbola, parabola (when `parabola`) or, when
    /// not, line of a plane-cone `section` done; no geometric solution for any
    /// other kind.
    fn copy_cone_section(&mut self, section: &QuadQuadGeo, parabola: bool) -> Result<(), Failure> {
        if !section.is_done() {
            return Ok(());
        }
        match section.type_inter()? {
            ResultType::Ellipse => {
                let e = section.ellipse(1)?;
                self.typeres = ResultType::Ellipse;
                (self.pt1, self.dir1, self.dir2) = (
                    e.location(),
                    e.position().direction(),
                    e.position().x_direction(),
                );
                (self.param1, self.param1bis, self.nbint) = (e.major_radius(), e.minor_radius(), 1);
            }
            ResultType::Circle => {
                let c = section.circle(1)?;
                self.typeres = ResultType::Circle;
                (self.pt1, self.dir1, self.dir2) = (
                    c.location(),
                    c.position().x_direction(),
                    c.position().y_direction(),
                );
                (self.param1, self.nbint) = (c.radius(), 1);
            }
            ResultType::Hyperbola => {
                let h = section.hyperbola(1)?;
                self.typeres = ResultType::Hyperbola;
                (self.pt1, self.pt2) = (h.location(), h.location());
                (self.dir1, self.dir2) = (h.position().direction(), h.position().x_direction());
                (self.param1, self.param2) = (h.major_radius(), h.major_radius());
                (self.param1bis, self.param2bis, self.nbint) =
                    (h.minor_radius(), h.minor_radius(), 2);
            }
            ResultType::Parabola if parabola => {
                let p = section.parabola(1)?;
                self.typeres = ResultType::Parabola;
                (self.pt1, self.dir1, self.dir2) = (
                    p.location(),
                    p.position().direction(),
                    p.position().x_direction(),
                );
                (self.param1, self.nbint) = (p.focal(), 1);
            }
            ResultType::Line if !parabola => {
                let l = section.line(1)?;
                self.typeres = ResultType::Line;
                (self.pt1, self.pt2) = (l.location(), l.location());
                (self.dir1, self.dir2) = (l.position().direction(), l.position().direction());
                (
                    self.param1,
                    self.param2,
                    self.param1bis,
                    self.param2bis,
                    self.nbint,
                ) = (0.0, 0.0, 0.0, 0.0, 2);
            }
            _ => self.typeres = ResultType::NoGeometricSolution,
        }
        Ok(())
    }

    /// Cones with a common generatrix through both apexes: the section of the first
    /// cone by the plane through the characteristic point that contains the
    /// intersection curve; no geometric solution when an apex is off the other cone.
    fn common_generatrix_cones(
        &mut self,
        first: Cone3,
        second: Cone3,
        axes: &AxeOperator,
        tol: f64,
    ) -> Result<(), Failure> {
        let tol2: f64 = tol * tol;
        let (apex1, apex2) = (first.apex(), second.apex());
        if !apex_on_cone(apex1, second, tol2) || !apex_on_cone(apex2, first, tol2) {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        self.my_common_gen = true;
        let generatrix = Line3::new(
            apex1,
            Direction3::from_vector(Vector3::between(apex1, apex2))?,
        );
        let u: f64 = elclib::parameter(&generatrix, axes.pt_intersect())?;
        self.my_pchar = elclib::value(u, &generatrix);
        // The other generatrices of the cones in the plane of the axes.
        let gen1: Line3 = generatrix.rotated(first.axis(), PI)?;
        let gen2: Line3 = generatrix.rotated(second.axis(), PI)?;
        let Some(normal) = self.section_normal(generatrix, gen1, gen2, tol2)? else {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        };
        let section =
            QuadQuadGeo::plane_cone(Plane3::new(self.my_pchar, normal)?, first, tol, tol)?;
        self.copy_cone_section(&section, true)
    }

    /// The normal of the plane of the intersection curves: from the meeting point
    /// of the two other generatrices, or across them when they are parallel;
    /// `None` when they are skew beyond `tol²`.
    fn section_normal(
        &self,
        generatrix: Line3,
        gen1: Line3,
        gen2: Line3,
        tol2: f64,
    ) -> Result<Option<Direction3>, Failure> {
        let d1: Direction3 = gen1.direction();
        let d2: Direction3 = d1.crossed(generatrix.direction())?;
        if d1.is_parallel(gen2.direction(), precision::angular()) {
            return Ok(Some(d1.crossed(d2)?));
        }
        if gen1.square_distance_line(gen2)? > tol2 {
            return Ok(None);
        }
        let (e1, e2) = (gen1.position().direction(), gen2.position().direction());
        let o1o2 = Vector3::between(gen1.location(), gen2.location());
        let e1_dot_e2: f64 = e1.dot(e2);
        let sin: f64 = 1. - e1_dot_e2 * e1_dot_e2;
        let mut u2: f64 = (e1.xyz() * o1o2.dot(Vector3::from(e1)) - *o1o2.xyz()).dot(e2.xyz());
        u2 /= sin;
        let meeting: Point3 = elclib::value(u2, &gen2);
        let towards: Direction3 =
            Direction3::from_vector(Vector3::between(meeting, self.my_pchar))?;
        Ok(Some(towards.crossed(d2)?))
    }
}

/// The apex lies on the cone within the square tolerance.
fn apex_on_cone(apex: Point3, cone: Cone3, tol2: f64) -> bool {
    let (u, v) = elslib::parameters(&cone, apex);
    let projection: Point3 = elslib::value(u, v, &cone);
    let square_distance: f64 = apex.square_distance(&projection);
    square_distance.is_nan() || square_distance <= tol2
}

impl QuadQuadGeo {
    /// Cones with a common apex: in the plane of the axes, the circle of the
    /// second cone at the height where the first has radius `tan(beta1)` misses,
    /// touches or cuts the first (`tol`): empty, one line or two lines through the
    /// apex, found in 3D on the line of the two sections at equal generatrix length.
    fn common_apex_cones(&mut self, first: Cone3, second: Cone3, tol: f64) -> Result<(), Failure> {
        let (ax1, mut ax2) = (first.axis(), second.axis());
        let tgb1: f64 = first.semi_angle().tan().abs();
        let tgb2: f64 = second.semi_angle().tan().abs();
        let Some(two_lines) = apex_configuration(ax1.angle(ax2), tgb1, tgb2, tol)? else {
            self.typeres = ResultType::Empty;
            return Ok(());
        };
        let d1: f64 = 1.;
        let apex: Point3 = first.apex();
        let dir1: Direction3 = ax1.direction();
        let qa1: Point3 = offset(apex, d1, dir1);
        if dir1.dot(ax2.direction()) < 0. {
            ax2.reverse();
        }
        let dir2: Direction3 = ax2.direction();
        let d2: f64 = d1 * ((1. + tgb1 * tgb1) / (1. + tgb2 * tgb2)).sqrt();
        let qa2: Point3 = offset(apex, d2, dir2);
        let mut sections = QuadQuadGeo::new();
        sections.perform_planes(Plane3::new(qa1, dir1)?, Plane3::new(qa2, dir2)?, tol, tol)?;
        if !sections.is_done() || 0 == sections.nb_solutions()? {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        let line: Line3 = sections.line(1)?;
        let v_lin = Vector3::from(line.direction());
        let origin: Point3 = line.location();
        let qx: Point3 = origin.translated(v_lin.dot(Vector3::between(qa1, origin)) * v_lin);
        self.typeres = ResultType::Line;
        (self.param1, self.param2, self.param1bis, self.param2bis) = (0., 0., 0., 0.);
        self.pt1 = apex;
        if !two_lines {
            self.nbint = 1;
            self.dir1 = Direction3::from_vector(Vector3::between(apex, qx))?;
            return Ok(());
        }
        self.nbint = 2;
        let r1: f64 = d1 * tgb1;
        let da: f64 = qa1.distance(&qx);
        let dx: f64 = (r1 * r1 - da * da).sqrt();
        let (qx1, qx2) = (qx.translated(dx * v_lin), qx.translated(-dx * v_lin));
        self.pt2 = apex;
        self.dir1 = Direction3::from_vector(Vector3::between(apex, qx1))?;
        self.dir2 = Direction3::from_vector(Vector3::between(apex, qx2))?;
        Ok(())
    }
}

/// The 2D analysis of cones with a common apex and axes at `gamma`: `None` when
/// the second circle misses the first generatrix point by more than `tol`, else
/// whether the cones cut (two lines) rather than touch (one line).
fn apex_configuration(gamma: f64, tgb1: f64, tgb2: f64, tol: f64) -> Result<Option<bool>, Failure> {
    let half_pi: f64 = 0.5 * PI;
    let gamma: f64 = if gamma > half_pi { PI - gamma } else { gamma };
    let d1: f64 = 1.;
    let origin = Point2::new(0., 0.);
    let p1 = Point2::new(d1, d1 * tgb1);
    let v_ax2 = Vector2::new(gamma.cos(), gamma.sin());
    let d_ax2: Direction2 = Direction2::from_vector(v_ax2)?;
    let dx: f64 = v_ax2.dot(Vector2::between(origin, p1));
    let pa2: Point2 = origin.translated(dx * Vector2::from(d_ax2));
    let r2: f64 = pa2.distance(origin) * tgb2;
    let rd2: f64 = pa2.distance(p1);
    if rd2 > (r2 + tol) {
        return Ok(None);
    }
    Ok(Some(rd2 < (r2 - tol)))
}
