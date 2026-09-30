// Rust translation of ModelingData/TKGeomBase/Approx/Approx_SameParameter.hxx/.cxx (fields, constructors, accessors, data)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_SameParameter`: a pcurve reparametrized so that, on its surface, it has the
//! parameter of the 3D curve within a tolerance.
mod build;
mod distribution;
mod fit;

use crate::native_failure::failure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};

/// `myNbSamples`: the initial samples, as CheckShape uses.
const NB_SAMPLES: i32 = 22;
/// `myMaxArraySize`: the capacity of the parameter arrays.
const MAX_ARRAY_SIZE: i32 = 1000;

/// `Approx_SameParameter`: the reparametrized pcurve, the tolerance reached, and
/// whether the input already had the same parameter.
///
/// When the initial distribution cannot be built (more than 999 samples on the C1
/// breaks), the native object leaves `myTolReached` uninitialized; the port reads 0.
///
/// ```ignore
/// let same = SameParameter::from_adaptors(&c3d, &pcurve, &surface, 1e-6)?;
/// if same.is_done() { let pcurve = same.curve_2d(); }
/// ```
#[derive(Clone)]
pub struct SameParameter {
    delta_min: f64,
    same_parameter: bool,
    done: bool,
    tol_reached: f64,
    curve_2d: Handle<dyn Geom2dCurve>,
    hcurve_2d: Handle<dyn Adaptor2dCurve2d>,
    c3d: Handle<dyn Adaptor3dCurve>,
    surf: Handle<dyn Adaptor3dSurface>,
    curve_on_surface: Handle<Adaptor3dCurveOnSurface>,
}

/// `Approx_SameParameter_Data`: the curve on surface, the sampled parameters on the
/// 3D and 2D curves (and a second pair of arrays for the loop over poles), the
/// parameter ranges and the working tolerance.
pub(super) struct SameParameterData {
    cons: Adaptor3dCurveOnSurface,
    nb_pnt: i32,
    pc3d: Vec<f64>,
    pc2d: Vec<f64>,
    new_pc3d: Vec<f64>,
    new_pc2d: Vec<f64>,
    c3d_pf: f64,
    c3d_pl: f64,
    c2d_pf: f64,
    c2d_pl: f64,
    tol: f64,
}

/// A value of a parameter array; beyond the native stack array Standard_OutOfRange.
pub(super) fn at(values: &[f64], index: i32) -> Result<f64, Failure> {
    usize::try_from(index)
        .ok()
        .and_then(|slot| values.get(slot))
        .copied()
        .ok_or_else(|| out_of_array(index))
}

/// Writes a parameter array; beyond the native stack array Standard_OutOfRange.
pub(super) fn put(values: &mut [f64], index: i32, value: f64) -> Result<(), Failure> {
    let slot: &mut f64 = usize::try_from(index)
        .ok()
        .and_then(|slot| values.get_mut(slot))
        .ok_or_else(|| out_of_array(index))?;
    *slot = value;
    Ok(())
}

fn out_of_array(index: i32) -> Failure {
    failure(
        "Standard_OutOfRange",
        &format!("Approx_SameParameter: sample {index}; expected 0..{MAX_ARRAY_SIZE}"),
    )
}

impl SameParameterData {
    /// `Swap(theNewNbPoints)`: the second arrays become the current ones.
    pub(super) fn swap(&mut self, new_nb_points: i32) {
        self.nb_pnt = new_nb_points;
        std::mem::swap(&mut self.pc3d, &mut self.new_pc3d);
        std::mem::swap(&mut self.pc2d, &mut self.new_pc2d);
    }
}

fn null_input(what: &str) -> Failure {
    failure(
        "Standard_NullObject",
        &format!("Approx_SameParameter: null {what}"),
    )
}

impl SameParameter {
    fn with_inputs(
        c3d: Handle<dyn Adaptor3dCurve>,
        hcurve_2d: Handle<dyn Adaptor2dCurve2d>,
        surf: Handle<dyn Adaptor3dSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut same = Self {
            delta_min: precision::p_confusion(),
            same_parameter: true,
            done: false,
            tol_reached: 0.0,
            curve_2d: Handle::default(),
            hcurve_2d,
            c3d,
            surf,
            curve_on_surface: Handle::default(),
        };
        same.build(tol)?;
        Ok(same)
    }

    /// `Approx_SameParameter(C3D, C2D, S, Tol)` on Geom objects (adapted by
    /// GeomAdaptor/Geom2dAdaptor); the curves must share their parametric domain;
    /// `SameParameter::from_geometry(&c3d, &pcurve, &surface, 1e-6)?`.
    pub fn from_geometry(
        c3d: &Handle<dyn GeomCurve>,
        c2d: &Handle<dyn Geom2dCurve>,
        s: &Handle<dyn GeomSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let hcurve_2d = Handle::new(Geom2dAdaptorCurve::with_curve(c2d)?).cast();
        let c3d = Handle::new(GeomAdaptorCurve::with_curve(c3d)?).cast();
        let surf = Handle::new(GeomAdaptorSurface::with_surface(s)?).cast();
        Self::with_inputs(c3d, hcurve_2d, surf, tol)
    }

    /// `Approx_SameParameter(C3D, C2D, S, Tol)` with adapted 3D curve and surface and
    /// a Geom2d pcurve; `SameParameter::from_adaptors_and_pcurve(&c3d, &pcurve, &s, 1e-6)?`.
    pub fn from_adaptors_and_pcurve(
        c3d: &Handle<dyn Adaptor3dCurve>,
        c2d: &Handle<dyn Geom2dCurve>,
        s: &Handle<dyn Adaptor3dSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let hcurve_2d = Handle::new(Geom2dAdaptorCurve::with_curve(c2d)?).cast();
        Self::with_inputs(c3d.clone(), hcurve_2d, s.clone(), tol)
    }

    /// `Approx_SameParameter(C3D, C2D, S, Tol)` on adaptors;
    /// `SameParameter::from_adaptors(&c3d, &pcurve, &s, 1e-6)?`.
    pub fn from_adaptors(
        c3d: &Handle<dyn Adaptor3dCurve>,
        c2d: &Handle<dyn Adaptor2dCurve2d>,
        s: &Handle<dyn Adaptor3dSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        Self::with_inputs(c3d.clone(), c2d.clone(), s.clone(), tol)
    }

    /// `IsDone()`; `same.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `TolReached()`: the largest distance between the 3D curve and the curve on
    /// surface; `same.tol_reached()`.
    pub fn tol_reached(&self) -> f64 {
        self.tol_reached
    }

    /// `IsSameParameter()`: the input already had the same parameter (then nothing
    /// is built and Curve2d() is null); `same.is_same_parameter()`.
    pub fn is_same_parameter(&self) -> bool {
        self.same_parameter
    }

    /// `Curve2d()`: the pcurve with the same parameter; `same.curve_2d()`.
    pub fn curve_2d(&self) -> Handle<dyn Geom2dCurve> {
        self.curve_2d.clone()
    }

    /// `Curve3d()`: the 3D curve; `same.curve_3d()`.
    pub fn curve_3d(&self) -> Handle<dyn Adaptor3dCurve> {
        self.c3d.clone()
    }

    /// `CurveOnSurface()`: the curve on surface of the last approximation (null
    /// unless an approximation was built); `same.curve_on_surface()`.
    pub fn curve_on_surface(&self) -> Handle<Adaptor3dCurveOnSurface> {
        self.curve_on_surface.clone()
    }

    fn c3d(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        self.c3d.get().ok_or_else(|| null_input("3D curve"))
    }

    fn hcurve_2d(&self) -> Result<&dyn Adaptor2dCurve2d, Failure> {
        self.hcurve_2d.get().ok_or_else(|| null_input("2D curve"))
    }

    fn surf(&self) -> Result<&dyn Adaptor3dSurface, Failure> {
        self.surf.get().ok_or_else(|| null_input("surface"))
    }
}
