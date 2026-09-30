// Rust translation of CPnts_UniformDeflection::Perform (CPnts_UniformDeflection.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CPnts_UniformDeflection::Perform`: the next (up to three) points of the iterator.
use super::UniformDeflection;
use crate::cpnts::PntsCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{Point3, Vector3};

impl<'a> UniformDeflection<'a> {
    fn curve(&self) -> Result<PntsCurve<'a>, Failure> {
        self.my_curve.ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "CPnts_UniformDeflection::Perform() - no curve, expected Initialize()",
                "",
            )
        })
    }

    /// `Perform`: store the points from `myFirstParam` until three are stored or the
    /// end is reached, then place the last point.
    pub(super) fn perform(&mut self) -> Result<(), Failure> {
        let curve: PntsCurve = self.curve()?;
        self.my_i_point = -1;
        self.my_nb_points = -1;
        let an_espilon: f64 = epsilon(self.my_first_param);
        while self.my_nb_points < 2 && !self.my_finish {
            self.my_nb_points += 1;
            let index: usize = self.my_nb_points as usize;
            self.my_params[index] = self.my_first_param;
            let (point, v1, v2) = curve.lifted_d2(self.my_first_param)?;
            self.my_points[index] = point;
            self.estimate_step(v1, v2);
            if self.my_control {
                self.control_step(curve, point)?;
            }
            self.my_first_param += self.my_du;
            // The last test avoids an endless increment below double precision.
            self.my_finish = self.my_last_param - self.my_first_param < self.my_tol_cur
                || self.my_du.abs() < self.my_tol_cur
                || self.my_du < an_espilon;
        }
        if self.my_finish {
            self.place_last_point(curve)?;
        }
        Ok(())
    }

    // The step sqrt(8 Deflection |D1| / |D2 ^ D1|), bounded by [TolCur, Dwmax]; the
    // step grows by half at a singular tangent, a null curvature or collinear derivatives.
    fn estimate_step(&mut self, v1: Vector3, v2: Vector3) {
        let norm_d1: f64 = v1.magnitude();
        if norm_d1 < self.my_tol_cur || v2.magnitude() < self.my_tol_cur {
            self.my_du = std_min(self.my_dwmax, 1.5 * self.my_du);
            return;
        }
        let norm_d2: f64 = v2.cross_magnitude(v1);
        if norm_d2 / norm_d1 < self.my_deflection {
            self.my_du = std_min(self.my_dwmax, 1.5 * self.my_du);
            return;
        }
        self.my_du = (8.0 * self.my_deflection * norm_d1 / norm_d2).sqrt();
        self.my_du = std_min(std_max(self.my_du, self.my_tol_cur), self.my_dwmax);
    }

    // WithControl: shrink the step when the mid point deviates from the chord by more
    // than Deflection / 5 ("arbitrary" threshold, which fails on inflections natively).
    fn control_step(&mut self, curve: PntsCurve<'_>, start: Point3) -> Result<(), Failure> {
        self.my_du = std_min(self.my_du, self.my_last_param - self.my_first_param);
        let p: Point3 = curve.lifted_d0(self.my_first_param + self.my_du)?;
        let p1: Point3 = curve.lifted_d0(self.my_first_param + (self.my_du / 2.0))?;
        let v1: Vector3 = Vector3::between(start, p);
        let norm_d1: f64 = v1.magnitude();
        if norm_d1 < self.my_deflection {
            return Ok(());
        }
        let mut norm_d2: f64 = Vector3::between(start, p1).cross_magnitude(v1) / norm_d1;
        if norm_d2 > self.my_deflection / 5.0 {
            norm_d2 = std_max(norm_d2, 1.1 * self.my_deflection);
            self.my_du *= (self.my_deflection / norm_d2).sqrt();
            self.my_du = std_min(std_max(self.my_du, self.my_tol_cur), self.my_dwmax);
        }
        Ok(())
    }

    // The end of the curve: with control and two stored points, the last point is
    // corrected; otherwise LastParameter is appended (over the third point when full).
    fn place_last_point(&mut self, curve: PntsCurve<'_>) -> Result<(), Failure> {
        if self.my_control && self.my_nb_points == 1 {
            return self.correct_controlled_end(curve);
        }
        self.my_nb_points += 1;
        if self.my_nb_points >= 3 {
            self.my_nb_points = 2;
        }
        let index: usize = self.my_nb_points as usize;
        self.my_params[index] = self.my_last_param;
        self.my_points[index] = curve.lifted_d0(self.my_last_param)?;
        Ok(())
    }

    // Two controlled points: a last span shorter than a third of the previous step is
    // merged by moving the first point to the middle; otherwise the end point replaces
    // the second point or is appended after it.
    fn correct_controlled_end(&mut self, curve: PntsCurve<'_>) -> Result<(), Failure> {
        let un1: f64 = self.my_params[0];
        if self.my_last_param - un1 < 0.33 * (self.my_last_param - self.my_first_param) {
            self.my_first_param = (self.my_last_param + un1) / 2.0;
            self.my_params[0] = self.my_first_param;
            self.my_params[1] = self.my_last_param;
            self.my_points[0] = curve.lifted_d0(self.my_params[0])?;
            self.my_points[1] = curve.lifted_d0(self.my_params[1])?;
            return Ok(());
        }
        let (p1, _, _) = curve.lifted_d2(self.my_last_param)?;
        let p: Point3 = self.my_points[0];
        let vv: Vector3 = Vector3::between(p1, p);
        let norm_d1: f64 = vv.magnitude();
        if norm_d1 < self.my_deflection {
            self.my_params[1] = self.my_last_param;
            self.my_points[1] = p1;
            return Ok(());
        }
        self.split_controlled_end(curve, un1, (p, p1, vv, norm_d1))
    }

    // The point interpolated between the first point and the end: dropped when it is
    // within Deflection of the chord and the first point is within Dwmax of the end.
    fn split_controlled_end(
        &mut self,
        curve: PntsCurve<'_>,
        un1: f64,
        (p, p1, vv, norm_d1): (Point3, Point3, Vector3, f64),
    ) -> Result<(), Failure> {
        self.my_first_param = (self.my_last_param * (self.my_params[1] - un1) + un1 * self.my_du)
            / (self.my_first_param - un1);
        let p2: Point3 = curve.lifted_d0(self.my_first_param)?;
        if vv.cross_magnitude(Vector3::between(p2, p)) / norm_d1 < self.my_deflection
            && un1 >= self.my_last_param - self.my_dwmax
        {
            // point n is removed
            self.my_params[1] = self.my_last_param;
            self.my_points[1] = p1;
            return Ok(());
        }
        self.my_params[1] = self.my_first_param;
        self.my_points[1] = p2;
        self.my_params[2] = self.my_last_param;
        self.my_points[2] = p1;
        self.my_nb_points += 1;
        Ok(())
    }
}
