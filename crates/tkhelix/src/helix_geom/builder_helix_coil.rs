// Rust translation of ModelingAlgorithms/TKHelix/HelixGeom/HelixGeom_BuilderHelixCoil.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HelixGeom_BuilderHelixCoil`: one B-spline approximation of the helix over the
//! whole parameter range.
use super::builder_approx_curve::{HelixGeomBuilderApproxCurve, HelixGeomBuilderApproxCurveBase};
use super::builder_helix_gen::{HelixGeomBuilderHelixGen, HelixGeomBuilderHelixGenBase};
use super::helix_curve::HelixGeomHelixCurve;
use super::tools::HelixGeomTools;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineCurve;

/// `HelixGeom_BuilderHelixCoil`: approximates the helix of its parameters with one
/// B-spline curve, in the standard frame (docs/helix-design.md section 2).
///
/// ```
/// use rustycad_tkhelix::helix_geom::{HelixGeomBuilderApproxCurve, HelixGeomBuilderHelixCoil, HelixGeomBuilderHelixGen};
/// let mut builder = HelixGeomBuilderHelixCoil::new();
/// builder.set_curve_parameters(0.0, 2.0 * std::f64::consts::PI, 5.0, 2.0, 0.0, true);
/// builder.perform()?;
/// assert_eq!(builder.curves().length(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct HelixGeomBuilderHelixCoil {
    base: HelixGeomBuilderHelixGenBase,
}

impl HelixGeomBuilderHelixCoil {
    /// `HelixGeom_BuilderHelixCoil()`: one clockwise turn `[0, 2*PI]` of pitch 1 and
    /// radius 1, untapered, with the default approximation settings;
    /// `HelixGeomBuilderHelixCoil::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl HelixGeomBuilderApproxCurve for HelixGeomBuilderHelixCoil {
    fn approx_curve_base(&self) -> &HelixGeomBuilderApproxCurveBase {
        &self.base.approx
    }
    fn approx_curve_base_mut(&mut self) -> &mut HelixGeomBuilderApproxCurveBase {
        &mut self.base.approx
    }

    /// `Perform()`: clears the curves, loads the helix (Standard_ConstructionError for
    /// invalid parameters, with the statuses already reset to 0) and approximates it;
    /// the error status is 2 when `ApprCurve3D` fails. The reached tolerance is the
    /// output of `ApprCurve3D`, so it keeps its previous value when AdvApprox is not
    /// done and is 0 when it has no result, as natively.
    fn perform(&mut self) -> Result<(), Failure> {
        let b = &mut self.base;
        b.approx.error_status = 0;
        b.approx.warning_status = 0;
        b.approx.curves.clear(None);
        let adaptor = HelixGeomHelixCurve::new();
        adaptor.load(
            b.t1,
            b.t2,
            b.pitch,
            b.r_start,
            b.taper_angle,
            b.is_clock_wise,
        )?;
        let mut bc: Handle<GeomBSplineCurve> = Handle::default();
        let i_err: i32 = HelixGeomTools::appr_curve_3d(
            &adaptor.to_adaptor(),
            b.approx.tolerance,
            b.approx.cont,
            b.approx.max_seg,
            b.approx.max_degree,
            &mut bc,
            &mut b.approx.tol_reached,
        )?;
        if i_err != 0 {
            b.approx.error_status = 2;
        } else {
            b.approx.curves.append(bc.cast());
        }
        Ok(())
    }
}

impl HelixGeomBuilderHelixGen for HelixGeomBuilderHelixCoil {
    fn helix_gen_base(&self) -> &HelixGeomBuilderHelixGenBase {
        &self.base
    }
    fn helix_gen_base_mut(&mut self) -> &mut HelixGeomBuilderHelixGenBase {
        &mut self.base
    }
}
