// Rust translation of ModelingAlgorithms/TKHelix/HelixGeom/HelixGeom_BuilderHelix.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HelixGeom_BuilderHelix`: the helix as one B-spline coil per full turn (plus the
//! remaining partial turn), placed in a coordinate system.
use super::builder_approx_curve::{HelixGeomBuilderApproxCurve, HelixGeomBuilderApproxCurveBase};
use super::builder_helix_coil::HelixGeomBuilderHelixCoil;
use super::builder_helix_gen::{HelixGeomBuilderHelixGen, HelixGeomBuilderHelixGenBase};
use super::errors::{c_int_truncation, null_object};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::{Axis2, Axis3, Point3, Transform3};
use std::f64::consts::PI;

/// `HelixGeom_BuilderHelix`: approximates the helix of its parameters coil by coil,
/// then moves the curves from the standard frame to [`position`](Self::position)
/// (docs/helix-design.md section 2).
///
/// ```
/// use rustycad_tkhelix::helix_geom::{HelixGeomBuilderApproxCurve, HelixGeomBuilderHelix, HelixGeomBuilderHelixGen};
/// let mut builder = HelixGeomBuilderHelix::new();
/// builder.set_curve_parameters(0.0, 6.0 * std::f64::consts::PI, 10.0, 5.0, 0.0, true);
/// builder.perform()?;
/// assert_eq!(builder.curves().length(), 3);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct HelixGeomBuilderHelix {
    base: HelixGeomBuilderHelixGenBase,
    position: Axis2,
}

impl HelixGeomBuilderHelix {
    /// `HelixGeom_BuilderHelix()`: the default helix in the standard frame;
    /// `HelixGeomBuilderHelix::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetPosition(aAx2)`: the frame of the helix axis; `builder.set_position(frame)`.
    pub fn set_position(&mut self, ax2: Axis2) {
        self.position = ax2;
    }

    /// `Position()`; `builder.position().location()`.
    pub fn position(&self) -> &Axis2 {
        &self.position
    }

    // A coil builder with this builder's tolerance and approximation settings.
    fn coil_builder(&self) -> HelixGeomBuilderHelixCoil {
        let mut bhc = HelixGeomBuilderHelixCoil::new();
        let approx = &self.base.approx;
        bhc.set_tolerance(approx.tolerance);
        bhc.set_approx_parameters(approx.cont, approx.max_degree, approx.max_seg);
        bhc
    }

    // Approximates [t1, t2] with `bhc` and appends its curve; false (error status 2)
    // when the approximation failed.
    fn append_coil(
        &mut self,
        bhc: &mut HelixGeomBuilderHelixCoil,
        t1: f64,
        t2: f64,
    ) -> Result<bool, Failure> {
        let b = &self.base;
        bhc.set_curve_parameters(t1, t2, b.pitch, b.r_start, b.taper_angle, b.is_clock_wise);
        bhc.perform()?;
        if bhc.error_status() != 0 {
            self.base.approx.error_status = 2;
            return Ok(false);
        }
        let curve: Handle<dyn GeomCurve> = bhc.curves().value(1)?.clone();
        self.base.approx.curves.append(curve);
        Ok(true)
    }

    // The largest reached tolerance of the coils.
    fn keep_max_tolerance(&mut self, reached: f64) {
        if reached > self.base.approx.tol_reached {
            self.base.approx.tol_reached = reached;
        }
    }

    // Case: less than one full turn, one coil.
    fn perform_single(&mut self, bhc: &mut HelixGeomBuilderHelixCoil) -> Result<bool, Failure> {
        let (t1, t2) = (self.base.t1, self.base.t2);
        if !self.append_coil(bhc, t1, t2)? {
            return Ok(false);
        }
        self.base.approx.tol_reached = bhc.tolerance_reached();
        Ok(true)
    }

    // The i-th coil of a cylindrical helix, i > 1: the first coil raised by (i - 1)
    // pitches, which skips its approximation.
    fn append_translated_coil(&mut self, i: i32) -> Result<(), Failure> {
        let c1: Handle<dyn GeomCurve> = self.base.approx.curves.value(1)?.clone();
        let curve: &dyn GeomCurve = c1.get().ok_or_else(|| null_object("aC1"))?;
        let p1: Point3 = curve.d0(curve.first_parameter())?;
        let pi = Point3::new(p1.x(), p1.y(), p1.z() + f64::from(i - 1) * self.base.pitch);
        let ci: Handle<dyn GeomCurve> = curve.translated_between(p1, pi)?.cast();
        self.base.approx.curves.append(ci);
        Ok(())
    }

    // Case: at least one full turn, one coil per turn and the rest; `n` full turns.
    fn perform_turns(
        &mut self,
        bhc: &mut HelixGeomBuilderHelixCoil,
        n: i32,
    ) -> Result<bool, Failure> {
        let two_pi: f64 = 2. * PI;
        let tol_angle: f64 = 1.0e-4;
        let is_cylindrical: bool = self.base.taper_angle.abs() < tol_angle;
        let mut t1x: f64 = self.base.t1;
        let mut t2x: f64 = self.base.t1 + two_pi;
        for i in 1..=n {
            if i > 1 && is_cylindrical {
                self.append_translated_coil(i)?;
            } else {
                if !self.append_coil(bhc, t1x, t2x)? {
                    return Ok(false);
                }
                self.keep_max_tolerance(bhc.tolerance_reached());
            }
            t1x = t2x;
            t2x = t1x + two_pi;
        }
        self.perform_rest(bhc, t1x)
    }

    // The remaining partial turn [t1x, T2], when longer than 1e-7 * 2*PI.
    fn perform_rest(
        &mut self,
        bhc: &mut HelixGeomBuilderHelixCoil,
        t1x: f64,
    ) -> Result<bool, Failure> {
        let t2x: f64 = self.base.t2;
        let eps: f64 = 1.0e-7 * (2. * PI);
        if (t2x - t1x).abs() <= eps {
            return Ok(true);
        }
        if !self.append_coil(bhc, t1x, t2x)? {
            return Ok(false);
        }
        self.keep_max_tolerance(bhc.tolerance_reached());
        Ok(true)
    }

    // Moves every curve from the standard frame to the position.
    fn place_curves(&self) -> Result<(), Failure> {
        let mut trsf = Transform3::default();
        trsf.set_displacement(Axis3::default(), Axis3::from_axis2(self.position));
        for curve in self.base.approx.curves.iter() {
            curve
                .get()
                .ok_or_else(|| null_object("aC"))?
                .transform(trsf)?;
        }
        Ok(())
    }
}

impl HelixGeomBuilderApproxCurve for HelixGeomBuilderHelix {
    fn approx_curve_base(&self) -> &HelixGeomBuilderApproxCurveBase {
        &self.base.approx
    }
    fn approx_curve_base_mut(&mut self) -> &mut HelixGeomBuilderApproxCurveBase {
        &mut self.base.approx
    }

    /// `Perform()`: the number of full turns is `(int)((T2 - T1) / 2*PI)`; without a
    /// full turn one coil covers the range, otherwise one coil per turn (a cylindrical
    /// helix translates its first coil instead of approximating the others) and one
    /// for the rest. The error status is 2 when a coil fails, and the curves built so
    /// far then stay unplaced; the reached tolerance is -1 until a coil sets it. An
    /// exception of a coil (invalid parameters) propagates.
    fn perform(&mut self) -> Result<(), Failure> {
        self.base.approx.error_status = 0;
        self.base.approx.warning_status = 0;
        self.base.approx.curves.clear(None);
        self.base.approx.tol_reached = -1.;
        let mut bhc = self.coil_builder();
        let dt: f64 = self.base.t2 - self.base.t1;
        // (int) truncation, INT_MIN for NaN as on the pinned x86-64 profile.
        let n: i32 = c_int_truncation(dt / (2. * PI));
        let done: bool = if n == 0 {
            self.perform_single(&mut bhc)?
        } else {
            self.perform_turns(&mut bhc, n)?
        };
        if !done {
            return Ok(());
        }
        self.place_curves()
    }
}

impl HelixGeomBuilderHelixGen for HelixGeomBuilderHelix {
    fn helix_gen_base(&self) -> &HelixGeomBuilderHelixGenBase {
        &self.base
    }
    fn helix_gen_base_mut(&mut self) -> &mut HelixGeomBuilderHelixGenBase {
        &mut self.base
    }
}
