// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_CheckBSplineCurve.hxx/.cxx/.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_CheckBSplineCurve`: the detection and repair of end tangents that turn
//! back along the control polygon of a 3D B-spline.
use super::live;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomGeometry};
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

/// The end tangent check of a non-periodic B-spline of at least 4 poles: an end
/// needs a fix when a pole aligned with its end tangent (within the angular
/// tolerance) lies behind the end.
///
/// `let mut check = CheckBSplineCurve::new(&curve, 1e-7, 1e-7)?; let (first, last) = check.need_tangent_fix();`
#[derive(Clone)]
pub struct CheckBSplineCurve {
    my_curve: Handle<GeomBSplineCurve>,
    my_done: bool,
    my_fix_first_tangent: bool,
    my_fix_last_tangent: bool,
    my_angular_tolerance: f64,
    my_tolerance: f64,
    my_ind_second_pole: i32,
    my_ind_prelast_pole: i32,
}

impl CheckBSplineCurve {
    /// Checks both ends of `curve` (`GeomLib_CheckBSplineCurve(Curve, Tolerance,
    /// AngularTolerance)`); a periodic curve or one of fewer than 4 poles is done at
    /// once; `CheckBSplineCurve::new(&curve, 1e-7, 1e-7)?`.
    pub fn new(
        curve: &Handle<GeomBSplineCurve>,
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
        let c: &GeomBSplineCurve = live(curve, "GeomLib_CheckBSplineCurve")?;
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

    // Walks the poles `others` from the end pole `end`: the first pole off the end
    // tangent (the pole next to `end`) stops the walk, the first one behind the end
    // is the index to fix; None when no pole is behind.
    fn reversed_index(
        &self,
        c: &GeomBSplineCurve,
        end: i32,
        others: Vec<i32>,
    ) -> Result<Option<(bool, i32)>, Failure> {
        let neighbour: i32 = if end == 1 { 2 } else { end - 1 };
        let end_pole: Point3 = c.pole(end)?;
        let tangent = Vector3::between(end_pole, c.pole(neighbour)?);
        let tangent_magnitude: f64 = tangent.magnitude();
        let tangent_normalized: Vector3 = if tangent_magnitude > self.my_tolerance {
            tangent / tangent_magnitude
        } else {
            Vector3::default()
        };
        let cross_prod_sq_tol: f64 = self.my_angular_tolerance * self.my_angular_tolerance;
        for ii in others {
            let a_vector = Vector3::between(end_pole, c.pole(ii)?);
            let vector_magnitude: f64 = a_vector.magnitude();
            if !(tangent_magnitude > self.my_tolerance && vector_magnitude > self.my_tolerance) {
                continue;
            }
            let avector_normalized: Vector3 = a_vector / vector_magnitude;
            let cross_prod: Vector3 = tangent_normalized.crossed(avector_normalized);
            if cross_prod.square_magnitude() > cross_prod_sq_tol {
                break;
            }
            if tangent.dot(a_vector) < 0.0 {
                return Ok(Some((true, ii)));
            }
        }
        Ok(None)
    }

    /// Whether the check is over: at construction when nothing needs a fix, after
    /// any repair otherwise (`IsDone()`); `check.is_done()`.
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
        let curve: Handle<GeomBSplineCurve> = self.my_curve.clone();
        self.fix_tangent_on_curve(&curve, first_flag, last_flag)
    }

    /// A repaired copy of the curve, null when no asked end needs a fix
    /// (`FixedTangent(FirstFlag, LastFlag)`); `check.fixed_tangent(true, true)?`.
    pub fn fixed_tangent(
        &mut self,
        first_flag: bool,
        last_flag: bool,
    ) -> Result<Handle<GeomBSplineCurve>, Failure> {
        if !((self.my_fix_first_tangent && first_flag) || (self.my_fix_last_tangent && last_flag)) {
            return Ok(Handle::default());
        }
        let new_curve: Handle<GeomBSplineCurve> =
            live(&self.my_curve, "GeomLib_CheckBSplineCurve")?
                .copy()?
                .cast();
        self.fix_tangent_on_curve(&new_curve, first_flag, last_flag)?;
        Ok(new_curve)
    }

    // The poles between the end and the pole behind it spread evenly on the segment
    // joining them; the check is then done.
    fn fix_tangent_on_curve(
        &mut self,
        curve: &Handle<GeomBSplineCurve>,
        first_flag: bool,
        last_flag: bool,
    ) -> Result<(), Failure> {
        let c: &GeomBSplineCurve = live(curve, "GeomLib_CheckBSplineCurve")?;
        if self.my_fix_first_tangent && first_flag {
            let (xyz1, xyz2): (Xyz, Xyz) =
                (c.pole(1)?.xyz(), c.pole(self.my_ind_second_pole)?.xyz());
            let nb_samples: f64 = f64::from(self.my_ind_second_pole - 1);
            for i in 2..self.my_ind_second_pole {
                let ii: f64 = f64::from(i - 1);
                c.set_pole(
                    i,
                    Point3::from((1.0 - ii / nb_samples) * xyz1 + ii / nb_samples * xyz2),
                )?;
            }
        }
        if self.my_fix_last_tangent && last_flag {
            let num_poles: i32 = c.nb_poles();
            let (xyz1, xyz2): (Xyz, Xyz) = (
                c.pole(num_poles)?.xyz(),
                c.pole(self.my_ind_prelast_pole)?.xyz(),
            );
            let nb_samples: f64 = f64::from(num_poles - self.my_ind_prelast_pole);
            for i in (self.my_ind_prelast_pole + 1..num_poles).rev() {
                let ii: f64 = f64::from(num_poles - i);
                c.set_pole(
                    i,
                    Point3::from((1.0 - ii / nb_samples) * xyz1 + ii / nb_samples * xyz2),
                )?;
            }
        }
        self.my_done = true;
        Ok(())
    }
}
