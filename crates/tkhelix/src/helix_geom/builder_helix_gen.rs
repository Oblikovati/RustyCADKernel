// Rust translation of ModelingAlgorithms/TKHelix/HelixGeom/HelixGeom_BuilderHelixGen.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HelixGeom_BuilderHelixGen`: the abstract helix builder that adds the helix
//! parameters to the approximation settings (docs/helix-design.md section 2).
use super::builder_approx_curve::{HelixGeomBuilderApproxCurve, HelixGeomBuilderApproxCurveBase};
use super::helix_curve::HelixParameters;
use std::f64::consts::PI;

/// The protected data of `HelixGeom_BuilderHelixGen` with that of its parent; its
/// `Default` is the protected constructor: one clockwise turn `[0, 2*PI]` of pitch 1
/// and radius 1, untapered.
#[derive(Clone)]
pub struct HelixGeomBuilderHelixGenBase {
    /// The data of `HelixGeom_BuilderApproxCurve`.
    pub approx: HelixGeomBuilderApproxCurveBase,
    /// `myT1`.
    pub t1: f64,
    /// `myT2`.
    pub t2: f64,
    /// `myPitch`.
    pub pitch: f64,
    /// `myRStart`.
    pub r_start: f64,
    /// `myTaperAngle`.
    pub taper_angle: f64,
    /// `myIsClockWise`.
    pub is_clock_wise: bool,
}

impl Default for HelixGeomBuilderHelixGenBase {
    fn default() -> Self {
        Self {
            approx: HelixGeomBuilderApproxCurveBase::default(),
            t1: 0.0,
            t2: 2.0 * PI,
            pitch: 1.0,
            r_start: 1.0,
            taper_angle: 0.0,
            is_clock_wise: true,
        }
    }
}

/// `HelixGeom_BuilderHelixGen`: the helix parameters of the concrete builders; the
/// provided methods are the native non-virtual members.
///
/// ```
/// use rustycad_tkhelix::helix_geom::{HelixGeomBuilderHelixCoil, HelixGeomBuilderHelixGen};
/// let mut builder = HelixGeomBuilderHelixCoil::new();
/// builder.set_curve_parameters(0.5, 5.5, 12.5, 3.5, 0.15, false);
/// assert_eq!(builder.curve_parameters().pitch, 12.5);
/// ```
pub trait HelixGeomBuilderHelixGen: HelixGeomBuilderApproxCurve {
    /// The protected data; `builder.helix_gen_base().pitch`.
    fn helix_gen_base(&self) -> &HelixGeomBuilderHelixGenBase;
    /// The protected data, mutable; `builder.helix_gen_base_mut().pitch = 2.0`.
    fn helix_gen_base_mut(&mut self) -> &mut HelixGeomBuilderHelixGenBase;

    /// `SetCurveParameters(aT1, aT2, aPitch, aRStart, aTaperAngle, bIsClockwise)`:
    /// stores the values unchecked (the helix `Load` of `Perform` checks them);
    /// `builder.set_curve_parameters(0.0, 6.28, 5.0, 2.0, 0.0, true)`.
    fn set_curve_parameters(
        &mut self,
        t1: f64,
        t2: f64,
        pitch: f64,
        r_start: f64,
        taper_angle: f64,
        is_cw: bool,
    ) {
        let base = self.helix_gen_base_mut();
        (base.t1, base.t2, base.pitch) = (t1, t2, pitch);
        (base.r_start, base.taper_angle, base.is_clock_wise) = (r_start, taper_angle, is_cw);
    }
    /// `CurveParameters(aT1, aT2, aPitch, aRStart, aTaperAngle, bIsClockwise)`: the six
    /// values; `builder.curve_parameters().t2`.
    fn curve_parameters(&self) -> HelixParameters {
        let base = self.helix_gen_base();
        HelixParameters {
            t1: base.t1,
            t2: base.t2,
            pitch: base.pitch,
            r_start: base.r_start,
            taper_angle: base.taper_angle,
            is_cw: base.is_clock_wise,
        }
    }
}
