// Rust translation of the ComputationMethods class (stCoeffsValue, CylCylMonotonicity,
// CylCylComputeParameters) and ShortCosForm of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The equation of the intersection of two cylinders.
//!
//! Every cylinder is `S(U,V) = L + R*cos(U)*Xd + R*sin(U)*Yd + V*Zd`. The system
//! `S1(U1,V1) = S2(U2,V2)` becomes `Ck1*V1 + Ck2*V2 = Ak1*cos(U1) + Bk1*sin(U1) +
//! Ak2*cos(U2) + Bk2*sin(U2) + Dk` (k = 1..3); two of its equations give V1 and V2 as
//! `K*cos(U1-FIV) + L*cos(U2-PSIV) + M`, and the third one `cos(U2-FI2) = B*cos(U1-FI1)
//! + C`, whose two solutions `U2 = FI2 +/- f(U1)` are the two intersection lines.
use super::inscribe::inscribe_point;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{epsilon, is_equal, real_small};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::gp::{Cylinder3, Xyz};
use std::f64::consts::PI;

/// A coefficient vector of the system (`math_Vector` natively, indices 1..3 here 0..2).
pub(in super::super) type Coeff3 = [f64; 3];

/// If `|a| <= NUL_VALUE` then `a` is considered to be 0.
pub(in super::super) const NUL_VALUE: f64 = 1.0e-11;

/// `ComputationMethods::stCoeffsValue`: the coefficients of the system of two cylinders.
#[derive(Clone, Copy, Debug)]
pub(in super::super) struct CoeffsValue {
    pub(in super::super) vec_a1: Coeff3,
    pub(in super::super) vec_a2: Coeff3,
    pub(in super::super) vec_b1: Coeff3,
    pub(in super::super) vec_b2: Coeff3,
    pub(in super::super) vec_c1: Coeff3,
    pub(in super::super) vec_c2: Coeff3,
    pub(in super::super) vec_d: Coeff3,
    /// sinU2, sinU1, cosU2, cosU1 and free member of V1.
    pub(in super::super) k21: f64,
    pub(in super::super) k11: f64,
    pub(in super::super) l21: f64,
    pub(in super::super) l11: f64,
    pub(in super::super) m1: f64,
    /// sinU2, sinU1, cosU2, cosU1 and free member of V2.
    pub(in super::super) k22: f64,
    pub(in super::super) k12: f64,
    pub(in super::super) l22: f64,
    pub(in super::super) l12: f64,
    pub(in super::super) m2: f64,
    pub(in super::super) k1: f64,
    pub(in super::super) l1: f64,
    pub(in super::super) k2: f64,
    pub(in super::super) l2: f64,
    pub(in super::super) fiv1: f64,
    pub(in super::super) psiv1: f64,
    pub(in super::super) fiv2: f64,
    pub(in super::super) psiv2: f64,
    pub(in super::super) b: f64,
    pub(in super::super) c: f64,
    pub(in super::super) fi1: f64,
    pub(in super::super) fi2: f64,
}

fn scaled(k: f64, v: Xyz) -> Coeff3 {
    [k * v.x(), k * v.y(), k * v.z()]
}

/// The couple of equations chosen for V1 and V2 (the greatest determinant).
#[derive(Clone, Copy, PartialEq, Eq)]
enum CoupleOfEquation {
    Coe12,
    Coe23,
    Coe13,
}

/// The couple and its determinant.
fn choose_couple(c1: &Coeff3, c2: &Coeff3) -> (CoupleOfEquation, f64) {
    let delta1: f64 = c1[0] * c2[1] - c1[1] * c2[0]; // 1-2
    let delta2: f64 = c1[1] * c2[2] - c1[2] * c2[1]; // 2-3
    let delta3: f64 = c1[0] * c2[2] - c1[2] * c2[0]; // 1-3
    let (abs1, abs2, abs3) = (delta1.abs(), delta2.abs(), delta3.abs());
    if abs1 >= abs2 {
        if abs3 > abs1 {
            return (CoupleOfEquation::Coe13, delta3);
        }
        return (CoupleOfEquation::Coe12, delta1);
    }
    if abs3 > abs2 {
        return (CoupleOfEquation::Coe13, delta3);
    }
    (CoupleOfEquation::Coe23, delta2)
}

/// Reorders the three equations so that the chosen couple is the first two.
fn reorder(v: &mut Coeff3, couple: CoupleOfEquation) {
    match couple {
        CoupleOfEquation::Coe12 => {}
        CoupleOfEquation::Coe23 => *v = [v[1], v[2], v[0]],
        CoupleOfEquation::Coe13 => *v = [v[0], v[2], v[1]],
    }
}

impl CoeffsValue {
    /// `stCoeffsValue(theCyl1, theCyl2)`: raises Standard_Failure when the axes are
    /// parallel (the determinant of V1, V2 is under the angular precision, as the
    /// sine of the angle between the axes).
    pub(in super::super) fn new(cyl1: &Cylinder3, cyl2: &Cylinder3) -> Result<Self, Failure> {
        let mut vecs: [Coeff3; 7] = [
            scaled(-cyl1.radius(), cyl1.x_axis().direction().xyz()),
            scaled(cyl2.radius(), cyl2.x_axis().direction().xyz()),
            scaled(-cyl1.radius(), cyl1.y_axis().direction().xyz()),
            scaled(cyl2.radius(), cyl2.y_axis().direction().xyz()),
            scaled(1.0, cyl1.axis().direction().xyz()),
            scaled(1.0, cyl2.axis().direction().xyz().reversed()),
            scaled(1.0, cyl2.location().xyz() - cyl1.location().xyz()),
        ];
        let (couple, det) = choose_couple(&vecs[4], &vecs[5]);
        if det.abs() < precision::angular() {
            return Err(Failure::with_message(
                "Standard_Failure",
                &format!(
                    "Error. Exception in divide by zerro (IntCyCyTrim)!!!! (determinant {det}; expected at least the angular precision)"
                ),
            ));
        }
        vecs.iter_mut().for_each(|v| reorder(v, couple));
        let [a1, a2, b1, b2, c1, c2, d] = vecs;
        let mut out = Self::with_vectors([a1, a2, b1, b2, c1, c2, d]);
        out.solve_v(det);
        out.solve_u2();
        Ok(out)
    }

    fn with_vectors([a1, a2, b1, b2, c1, c2, d]: [Coeff3; 7]) -> Self {
        Self {
            vec_a1: a1,
            vec_a2: a2,
            vec_b1: b1,
            vec_b2: b2,
            vec_c1: c1,
            vec_c2: c2,
            vec_d: d,
            k21: 0.0,
            k11: 0.0,
            l21: 0.0,
            l11: 0.0,
            m1: 0.0,
            k22: 0.0,
            k12: 0.0,
            l22: 0.0,
            l12: 0.0,
            m2: 0.0,
            k1: 0.0,
            l1: 0.0,
            k2: 0.0,
            l2: 0.0,
            fiv1: 0.0,
            psiv1: 0.0,
            fiv2: 0.0,
            psiv2: 0.0,
            b: 0.0,
            c: 0.0,
            fi1: 0.0,
            fi2: 0.0,
        }
    }

    /// V1 and V2 from the first two equations (Cramer), in short cosine forms.
    fn solve_v(&mut self, det: f64) {
        let (c1, c2) = (self.vec_c1, self.vec_c2);
        let (a1, a2, b1, b2, d) = (
            self.vec_a1,
            self.vec_a2,
            self.vec_b1,
            self.vec_b2,
            self.vec_d,
        );
        self.k21 = (c2[1] * b2[0] - c2[0] * b2[1]) / det;
        self.k11 = (c2[1] * b1[0] - c2[0] * b1[1]) / det;
        self.l21 = (c2[1] * a2[0] - c2[0] * a2[1]) / det;
        self.l11 = (c2[1] * a1[0] - c2[0] * a1[1]) / det;
        self.m1 = (c2[1] * d[0] - c2[0] * d[1]) / det;
        self.k22 = (c1[0] * b2[1] - c1[1] * b2[0]) / det;
        self.k12 = (c1[0] * b1[1] - c1[1] * b1[0]) / det;
        self.l22 = (c1[0] * a2[1] - c1[1] * a2[0]) / det;
        self.l12 = (c1[0] * a1[1] - c1[1] * a1[0]) / det;
        self.m2 = (c1[0] * d[1] - c1[1] * d[0]) / det;
        (self.k1, self.fiv1) = short_cos_form(self.l11, self.k11);
        (self.l1, self.psiv1) = short_cos_form(self.l21, self.k21);
        (self.k2, self.fiv2) = short_cos_form(self.l12, self.k12);
        (self.l2, self.psiv2) = short_cos_form(self.l22, self.k22);
    }

    /// `cos(U2-FI2) = B*cos(U1-FI1) + C` from the third equation.
    fn solve_u2(&mut self) {
        let (c1, c2) = (self.vec_c1[2], self.vec_c2[2]);
        let a1: f64 = c1 * self.k21 + c2 * self.k22 - self.vec_b2[2]; // sinU2
        let a2: f64 = c1 * self.l21 + c2 * self.l22 - self.vec_a2[2]; // cosU2
        let b1: f64 = self.vec_b1[2] - c1 * self.k11 - c2 * self.k12; // sinU1
        let b2: f64 = self.vec_a1[2] - c1 * self.l11 - c2 * self.l12; // cosU1
        self.c = self.vec_d[2] - c1 * self.m1 - c2 * self.m2; // Free
        (self.b, self.fi1) = short_cos_form(b2, b1);
        let (a, fi2) = short_cos_form(a2, a1);
        self.fi2 = fi2;
        self.b /= a;
        self.c /= a;
    }
}

/// `ShortCosForm(theCosFactor, theSinFactor, theCoeff, theAngle)`: represents
/// `cos_factor*cos(A) + sin_factor*sin(A)` as `coeff*cos(A - angle)`.
pub(in super::super) fn short_cos_form(cos_factor: f64, sin_factor: f64) -> (f64, f64) {
    let coeff: f64 = (cos_factor * cos_factor + sin_factor * sin_factor).sqrt();
    if is_equal(coeff, 0.0) {
        return (coeff, 0.0);
    }
    let mut angle: f64 = (cos_factor / coeff).abs().acos();
    if sin_factor > 0.0 {
        if is_equal(cos_factor, 0.0) {
            angle = PI / 2.0;
        } else if cos_factor < 0.0 {
            angle = PI - angle;
        }
    } else if is_equal(sin_factor, 0.0) && cos_factor < 0.0 {
        angle = PI;
    }
    if sin_factor < 0.0 {
        if cos_factor > 0.0 {
            angle = 2.0 * PI - angle;
        } else if is_equal(cos_factor, 0.0) {
            angle = 3.0 * PI / 2.0;
        } else if cos_factor < 0.0 {
            angle += PI;
        }
    }
    (coeff, angle)
}

/// `CylCylMonotonicity(theU1par, theWLIndex, theCoeffs, thePeriod, theIsIncreasing)`:
/// whether `U2(U1) = FI2 +/- acos(B*cos(U1-FI1) + C)` increases at `u1par` on the line
/// `wl_index` (0 for "+", 1 for "-"); `None` for another index.
pub(in super::super) fn cyl_cyl_monotonicity(
    u1par: f64,
    wl_index: i32,
    coeffs: &CoeffsValue,
    period: f64,
) -> Option<bool> {
    let is_plus: bool = match wl_index {
        0 => true,
        1 => false,
        _ => return None,
    };
    let mut u1_temp: f64 = u1par - coeffs.fi1;
    inscribe_point(0.0, period, &mut u1_temp, 0.0, period, false);
    let mut is_increasing: bool = !((PI - u1_temp) < real_small() && u1_temp < period);
    if coeffs.b < 0.0 {
        is_increasing = !is_increasing;
    }
    if !is_plus {
        is_increasing = !is_increasing;
    }
    Some(is_increasing)
}

/// `CylCylComputeParameters(theU1par, theWLIndex, theCoeffs, theU2, theDelta)`: U2 of
/// the line `wl_index` at `u1par`, and (when asked) the estimated error of the arccosine
/// near +-1 (Taylor: `acos(p) - acos(p+x) = x/sqrt(1-p*p)`); `None` for another index.
/// Raises Standard_DivideByZero when the argument is too near +-1 for the estimate.
pub(in super::super) fn cyl_cyl_compute_u2(
    u1par: f64,
    wl_index: i32,
    coeffs: &CoeffsValue,
    delta: Option<&mut f64>,
) -> Result<Option<f64>, Failure> {
    // This formula is got from some experience and can be changed.
    let tol0: f64 = std_min(10.0 * epsilon(1.0) * coeffs.b, NUL_VALUE);
    let tol: f64 = 1.0 - tol0;
    if !(0..=1).contains(&wl_index) {
        return Ok(None);
    }
    let sign: f64 = if wl_index != 0 { -1.0 } else { 1.0 };
    let mut arg: f64 = (u1par - coeffs.fi1).cos();
    arg = coeffs.b * arg + coeffs.c;
    if arg >= tol {
        if let Some(d) = delta {
            *d = 0.0;
        }
        arg = 1.0;
    } else if arg <= -tol {
        if let Some(d) = delta {
            *d = 0.0;
        }
        arg = -1.0;
    } else if let Some(d) = delta {
        let dl: f64 = std_min(1.0 - arg, 1.0 + arg);
        if dl * dl < real_small() || dl >= 2.0 {
            return Err(Failure::with_message(
                "Standard_DivideByZero",
                &format!(
                    "IntPatch_ImpImpIntersection_4.gxx, CylCylComputeParameters() (delta {dl}; expected in ]0, 2[)"
                ),
            ));
        }
        *d = tol0 / (dl * (2.0 - dl)).sqrt();
    }
    Ok(Some(coeffs.fi2 + sign * arg.acos()))
}

/// `CylCylComputeParameters(theU1, theU2, theCoeffs, theV1, theV2)`: V1 and V2.
pub(in super::super) fn cyl_cyl_compute_v(u1: f64, u2: f64, coeffs: &CoeffsValue) -> (f64, f64) {
    let v1: f64 = coeffs.k21 * u2.sin()
        + coeffs.k11 * u1.sin()
        + coeffs.l21 * u2.cos()
        + coeffs.l11 * u1.cos()
        + coeffs.m1;
    let v2: f64 = coeffs.k22 * u2.sin()
        + coeffs.k12 * u1.sin()
        + coeffs.l22 * u2.cos()
        + coeffs.l12 * u1.cos()
        + coeffs.m2;
    (v1, v2)
}

/// `CylCylComputeParameters(theU1par, theWLIndex, theCoeffs, theU2, theV1, theV2)`:
/// `(U2, V1, V2)` of the line `wl_index` at `u1par`.
pub(in super::super) fn cyl_cyl_compute_uvv(
    u1par: f64,
    wl_index: i32,
    coeffs: &CoeffsValue,
) -> Result<Option<(f64, f64, f64)>, Failure> {
    let Some(u2) = cyl_cyl_compute_u2(u1par, wl_index, coeffs, None)? else {
        return Ok(None);
    };
    let (v1, v2) = cyl_cyl_compute_v(u1par, u2, coeffs);
    Ok(Some((u2, v1, v2)))
}
