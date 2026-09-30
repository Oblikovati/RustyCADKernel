//! The 2d representation of a 3d tangent on a surface, in the regular and the singular
//! cases (parallel or null first derivatives).
use rustycad_tkmath::gp::{Vector2, Vector3};

/// `IsSingular(theDU, theDV, theSqLinTol, theAngTol)`: whether a derivative is null
/// (square magnitude below `sq_lin_tol`) or the normalized derivatives are parallel.
pub(super) fn is_singular(du: Vector3, dv: Vector3, sq_lin_tol: f64, ang_tol: f64) -> bool {
    let (mut a_du, mut a_dv) = (du, dv);
    let sq_magn_du: f64 = a_du.square_magnitude();
    let sq_magn_dv: f64 = a_dv.square_magnitude();
    if sq_magn_du < sq_lin_tol {
        return true;
    }
    a_du /= sq_magn_du.sqrt();
    if sq_magn_dv < sq_lin_tol {
        return true;
    }
    a_dv /= sq_magn_dv.sqrt();
    // Here aDU and aDV have magnitude 1.
    a_du.crossed(a_dv).square_magnitude() < ang_tol * ang_tol
}

/// The data of a `SingularProcessing` call: the derivatives, the tolerances and
/// whether the 3d tangent is to be computed (else it is checked).
#[derive(Clone, Copy)]
pub(super) struct Singular {
    pub(super) du: Vector3,
    pub(super) dv: Vector3,
    pub(super) is_to_3d_tg_compute: bool,
    pub(super) lin_tol: f64,
    pub(super) ang_tol: f64,
}

impl Singular {
    /// Whether `tg3d` is parallel to `d` (of square magnitude `sq_magn_d`).
    fn parallel(&self, d: Vector3, sq_magn_d: f64, tg3d: Vector3, sq_tan: f64) -> bool {
        d.crossed(tg3d).square_magnitude() < self.ang_tol * self.ang_tol * sq_magn_d * sq_tan
    }

    /// Cases 2 and 3: one derivative `d` (of square magnitude `sq_magn`) is not null;
    /// the tangent is along it, as the U (`along_u`) or V coordinate of the 2d tangent.
    fn along_one(
        &self,
        (d, sq_magn, along_u): (Vector3, f64, bool),
        tg3d: &mut Vector3,
        tg2d: &mut Vector2,
    ) -> bool {
        let coord: f64 = if self.is_to_3d_tg_compute {
            // theTg3D will be normalized; its magnitude is 1.
            let norm: f64 = sq_magn.sqrt();
            *tg3d = d / norm;
            1.0 / norm
        } else {
            let sq_tan: f64 = tg3d.square_magnitude();
            if !self.parallel(d, sq_magn, *tg3d, sq_tan) {
                // theTg3D is not parallel to the derivative: abnormal.
                return false;
            }
            // "+" when theTg3D and the derivative are codirectional, "-" if opposite.
            (sq_tan / sq_magn).sqrt().copysign(tg3d.dot(d))
        };
        if along_u {
            tg2d.set_coordinates(coord, 0.0);
        } else {
            tg2d.set_coordinates(0.0, coord);
        }
        true
    }

    /// Case 4: both derivatives are not null but parallel; `sign` is 1 when they are
    /// codirectional, -1 when opposite.
    fn parallel_both(
        &self,
        (sq_magn_du, len_sum, sign): (f64, f64, f64),
        tg3d: &mut Vector3,
        tg2d: &mut Vector2,
    ) -> bool {
        if self.is_to_3d_tg_compute {
            // With opposite derivatives theDU gives the direction; the true one is
            // fixed later from the walking points.
            tg2d.set_coordinates(1.0 / len_sum, sign / len_sum);
            *tg3d = self.du * tg2d.x() + self.dv * tg2d.y();
            return true;
        }
        let sq_tan: f64 = tg3d.square_magnitude();
        if !self.parallel(self.du, sq_magn_du, *tg3d, sq_tan) {
            return false;
        }
        let len_tg: f64 = sq_tan.sqrt().copysign(tg3d.dot(self.du));
        let second: f64 = if sign > 0.0 {
            len_tg / len_sum
        } else {
            -len_tg / len_sum
        };
        tg2d.set_coordinates(len_tg / len_sum, second);
        true
    }

    /// `SingularProcessing(theDU, theDV, theIsTo3DTgCompute, theLinTol, theAngTol,
    /// theTg3D, theTg2D)`: the 2d tangent (and the 3d one when asked) when the cross
    /// product of the derivatives is null; false for two null derivatives or a given
    /// tangent that is not along them.
    pub(super) fn process(&self, tg3d: &mut Vector3, tg2d: &mut Vector2) -> bool {
        let sq_magn_du: f64 = self.du.square_magnitude();
        let sq_magn_dv: f64 = self.dv.square_magnitude();
        // 1. Both are null (osculating surfaces are not processed).
        if sq_magn_du < self.lin_tol && sq_magn_dv < self.lin_tol {
            return false;
        }
        // 2. The tangent is along theDV.
        if sq_magn_du < self.lin_tol {
            return self.along_one((self.dv, sq_magn_dv, false), tg3d, tg2d);
        }
        // 3. The tangent is along theDU.
        if sq_magn_dv < self.lin_tol {
            return self.along_one((self.du, sq_magn_du, true), tg3d, tg2d);
        }
        // 4. theDV || theDU; aLenSum > 0 definitely.
        let len_sum: f64 = sq_magn_du.sqrt() + sq_magn_dv.sqrt();
        let sign: f64 = if self.dv.dot(self.du) > 0.0 {
            1.0
        } else {
            -1.0
        };
        self.parallel_both((sq_magn_du, len_sum, sign), tg3d, tg2d)
    }
}

/// `NonSingularProcessing(theDU, theDV, theTg3D, theLinTol, theAngTol, theTg2D)`: the
/// 2d tangent of `tg3d` from `A x T = (A x B) V` and `B x T = (B x A) U`; the singular
/// processing when the derivatives are singular.
pub(super) fn non_singular(
    (du, dv): (Vector3, Vector3),
    tg3d: Vector3,
    (lin_tol, ang_tol): (f64, f64),
    tg2d: &mut Vector2,
) -> bool {
    let normal: Vector3 = du.crossed(dv);
    let sq_magn: f64 = normal.square_magnitude();
    if is_singular(du, dv, lin_tol, ang_tol) {
        let mut tg: Vector3 = tg3d;
        let singular = Singular {
            du,
            dv,
            is_to_3d_tg_compute: false,
            lin_tol,
            ang_tol,
        };
        return singular.process(&mut tg, tg2d);
    }
    let tg_u: Vector3 = tg3d.crossed(du);
    let tg_v: Vector3 = tg3d.crossed(dv);
    let delta_u: f64 = tg_v.square_magnitude() / sq_magn;
    let delta_v: f64 = tg_u.square_magnitude() / sq_magn;
    tg2d.set_coordinates(
        delta_u.sqrt().copysign(tg_v.dot(normal)),
        -(delta_v.sqrt().copysign(tg_u.dot(normal))),
    );
    true
}
