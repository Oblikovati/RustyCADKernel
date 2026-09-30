// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_Check2dBSplineCurve.hxx/.cxx/.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_Check2dBSplineCurve`: the detection and repair of end tangents that
//! turn back along the control polygon of a 2D B-spline.
use super::live;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkmath::gp::{Point2, Vector2, Xy};

/// The 2D twin of [`super::CheckBSplineCurve`]: the alignment test is on the
/// absolute 2D cross product against the angular tolerance (the 3D class compares
/// the squared cross magnitude with its square).
///
/// `let check = Check2dBSplineCurve::new(&curve, 1e-7, 1e-7)?; let (first, last) = check.need_tangent_fix();`
#[derive(Clone)]
pub struct Check2dBSplineCurve {
    my_curve: Handle<Geom2dBSplineCurve>,
    my_done: bool,
    my_fix_first_tangent: bool,
    my_fix_last_tangent: bool,
    my_angular_tolerance: f64,
    my_tolerance: f64,
    my_ind_second_pole: i32,
    my_ind_prelast_pole: i32,
}

impl Check2dBSplineCurve {
    /// Checks both ends of `curve` (`GeomLib_Check2dBSplineCurve(Curve, Tolerance,
    /// AngularTolerance)`); a periodic curve or one of fewer than 4 poles is done at
    /// once; `Check2dBSplineCurve::new(&curve, 1e-7, 1e-7)?`.
    pub fn new(
        curve: &Handle<Geom2dBSplineCurve>,
        tolerance: f64,
        angular_tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut check = Self {
            my_curve: curve.clone(),
            my_done: false,
            my_fix_first_tangent: false,
            my_fix_last_tangent: false,
            my_angular_tolerance: angular_tolerance.abs(),
            my_tolerance: tolerance.abs(),
            my_ind_second_pole: -1,
            my_ind_prelast_pole: -1,
        };
        let c: &Geom2dBSplineCurve = live(curve, "GeomLib_Check2dBSplineCurve")?;
        let num_poles: i32 = c.nb_poles();
        if c.is_periodic() || num_poles < 4 {
            check.my_done = true;
            return Ok(check);
        }
        let first = check.reversed_index(c, 1, (3..=num_poles).collect())?;
        (check.my_fix_first_tangent, check.my_ind_second_pole) = first.unwrap_or((false, -1));
        let last = check.reversed_index(c, num_poles, (1..=num_poles - 2).rev().collect())?;
        (check.my_fix_last_tangent, check.my_ind_prelast_pole) = last.unwrap_or((false, -1));
        Ok(check)
    }

    // Walks the poles `others` from the end pole `end` (see the 3D class).
    fn reversed_index(
        &self,
        c: &Geom2dBSplineCurve,
        end: i32,
        others: Vec<i32>,
    ) -> Result<Option<(bool, i32)>, Failure> {
        let neighbour: i32 = if end == 1 { 2 } else { end - 1 };
        let end_pole: Point2 = c.pole(end)?;
        let tangent = Vector2::between(end_pole, c.pole(neighbour)?);
        let tangent_magnitude: f64 = tangent.magnitude();
        let tangent_normalized: Vector2 = if tangent_magnitude > self.my_tolerance {
            tangent / tangent_magnitude
        } else {
            Vector2::default()
        };
        let cross_prod_tol: f64 = self.my_angular_tolerance;
        for ii in others {
            let a_vector = Vector2::between(end_pole, c.pole(ii)?);
            let vector_magnitude: f64 = a_vector.magnitude();
            if !(tangent_magnitude > self.my_tolerance && vector_magnitude > self.my_tolerance) {
                continue;
            }
            let avector_normalized: Vector2 = a_vector / vector_magnitude;
            let cross_prod: f64 = tangent_normalized.crossed(avector_normalized);
            if cross_prod.abs() > cross_prod_tol {
                break;
            }
            if tangent.dot(a_vector) < 0.0 {
                return Ok(Some((true, ii)));
            }
        }
        Ok(None)
    }

    /// Whether the check is over (`IsDone()`); `check.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }

    /// Whether the first and last tangents need a fix (`NeedTangentFix(FirstFlag,
    /// SecondFlag)`); `let (first, last) = check.need_tangent_fix();`.
    pub fn need_tangent_fix(&self) -> (bool, bool) {
        (self.my_fix_first_tangent, self.my_fix_last_tangent)
    }

    /// Repairs the checked curve itself (the caller's object) at the ends asked
    /// (`FixTangent(FirstFlag, LastFlag)`); `check.fix_tangent(true, true)?`.
    pub fn fix_tangent(&mut self, first_flag: bool, last_flag: bool) -> Result<(), Failure> {
        let curve: Handle<Geom2dBSplineCurve> = self.my_curve.clone();
        self.fix_tangent_on_curve(&curve, first_flag, last_flag)
    }

    /// A repaired copy of the curve, null when no asked end needs a fix
    /// (`FixedTangent(FirstFlag, LastFlag)`); `check.fixed_tangent(true, true)?`.
    pub fn fixed_tangent(
        &mut self,
        first_flag: bool,
        last_flag: bool,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        if !((self.my_fix_first_tangent && first_flag) || (self.my_fix_last_tangent && last_flag)) {
            return Ok(Handle::default());
        }
        let new_curve: Handle<Geom2dBSplineCurve> =
            live(&self.my_curve, "GeomLib_Check2dBSplineCurve")?
                .copy()?
                .cast();
        self.fix_tangent_on_curve(&new_curve, first_flag, last_flag)?;
        Ok(new_curve)
    }

    // The poles between the end and the pole behind it spread evenly on the segment
    // joining them; the check is then done.
    fn fix_tangent_on_curve(
        &mut self,
        curve: &Handle<Geom2dBSplineCurve>,
        first_flag: bool,
        last_flag: bool,
    ) -> Result<(), Failure> {
        let c: &Geom2dBSplineCurve = live(curve, "GeomLib_Check2dBSplineCurve")?;
        if self.my_fix_first_tangent && first_flag {
            let (xy1, xy2): (Xy, Xy) = (c.pole(1)?.xy(), c.pole(self.my_ind_second_pole)?.xy());
            let nb_samples: f64 = f64::from(self.my_ind_second_pole - 1);
            for i in 2..self.my_ind_second_pole {
                let ii: f64 = f64::from(i - 1);
                let xy: Xy = (1.0 - ii / nb_samples) * xy1 + ii / nb_samples * xy2;
                c.set_pole(i, Point2::new(xy.x(), xy.y()))?;
            }
        }
        if self.my_fix_last_tangent && last_flag {
            let num_poles: i32 = c.nb_poles();
            let (xy1, xy2): (Xy, Xy) = (
                c.pole(num_poles)?.xy(),
                c.pole(self.my_ind_prelast_pole)?.xy(),
            );
            let nb_samples: f64 = f64::from(num_poles - self.my_ind_prelast_pole);
            for i in (self.my_ind_prelast_pole + 1..num_poles).rev() {
                let ii: f64 = f64::from(num_poles - i);
                let xy: Xy = (1.0 - ii / nb_samples) * xy1 + ii / nb_samples * xy2;
                c.set_pole(i, Point2::new(xy.x(), xy.y()))?;
            }
        }
        self.my_done = true;
        Ok(())
    }
}
