// Rust translation of ModelingAlgorithms/TKGeomAlgo/AppBlend/AppBlend_AppSurf.gxx/.lxx (the
// constructors, settings and queries) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppBlend_AppSurf`: the approximation of a swept surface by a rational B-spline
//! surface, built from the sections of a line (the `perform` methods live in
//! `internal_perform` and `tronc_perform`).
use super::{AppBlendApprox, AppBlendCurve2d, AppBlendSurface};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};

/// The surface approximation of `AppBlend_AppSurf`, independent of the line and
/// section generator types, which only its `perform` methods take.
///
/// `GeomFill_AppSurf`, `GeomFill_AppSweep` and `BRepBlend_AppSurf` are this type;
/// `AppBlendAppSurf::new(3, 8, 1e-4, 1e-5, 5, false)`.
#[derive(Clone)]
pub struct AppBlendAppSurf {
    pub(super) done: bool,
    pub(super) dmin: i32,
    pub(super) dmax: i32,
    pub(super) tol3d: f64,
    pub(super) tol2d: f64,
    pub(super) nbit: i32,
    pub(super) udeg: i32,
    pub(super) vdeg: i32,
    pub(super) knownp: bool,
    pub(super) tab_poles: Array2<'static, Point3>,
    pub(super) tab_weights: Array2<'static, f64>,
    pub(super) tab_u_knots: Array1<'static, f64>,
    pub(super) tab_v_knots: Array1<'static, f64>,
    pub(super) tab_u_mults: Array1<'static, i32>,
    pub(super) tab_v_mults: Array1<'static, i32>,
    pub(super) seq_poles2d: Vec<Array1<'static, Point2>>,
    pub(super) tol3dreached: f64,
    pub(super) tol2dreached: f64,
    pub(super) paramtype: ParametrizationType,
    pub(super) continuity: Shape,
    pub(super) critweights: [f64; 3],
}

impl Default for AppBlendAppSurf {
    /// The default constructor: degrees 0, tolerances 0, no iteration.
    fn default() -> Self {
        Self::new(0, 0, 0.0, 0.0, 0, false)
    }
}

impl AppBlendAppSurf {
    /// An approximation of V degrees `degmin..=degmax` within `tol3d` and `tol2d`
    /// with `nb_it` iterations; `known_parameters` uses the section parameters.
    /// Chord-length parametrization, C2 continuity and the criterion weights
    /// (0.4, 0.2, 0.4); `AppBlendAppSurf::new(3, 8, 1e-4, 1e-5, 5, false)`.
    pub fn new(
        degmin: i32,
        degmax: i32,
        tol3d: f64,
        tol2d: f64,
        nb_it: i32,
        known_parameters: bool,
    ) -> Self {
        Self {
            done: false,
            dmin: degmin,
            dmax: degmax,
            tol3d,
            tol2d,
            nbit: nb_it,
            udeg: 0,
            vdeg: 0,
            knownp: known_parameters,
            tab_poles: Array2::default(),
            tab_weights: Array2::default(),
            tab_u_knots: Array1::default(),
            tab_v_knots: Array1::default(),
            tab_u_mults: Array1::default(),
            tab_v_mults: Array1::default(),
            seq_poles2d: Vec::new(),
            tol3dreached: 0.0,
            tol2dreached: 0.0,
            paramtype: ParametrizationType::ChordLength,
            continuity: Shape::C2,
            critweights: [0.4, 0.2, 0.4],
        }
    }
    /// `Init(...)`: the settings of `new`; the degrees, reached tolerances and
    /// results of an earlier computation are kept, as natively.
    pub fn init(
        &mut self,
        degmin: i32,
        degmax: i32,
        tol3d: f64,
        tol2d: f64,
        nb_it: i32,
        known_parameters: bool,
    ) {
        self.done = false;
        (self.dmin, self.dmax, self.tol3d, self.tol2d) = (degmin, degmax, tol3d, tol2d);
        (self.nbit, self.knownp) = (nb_it, known_parameters);
        self.continuity = Shape::C2;
        self.paramtype = ParametrizationType::ChordLength;
        self.critweights = [0.4, 0.2, 0.4];
    }
    /// `SetParType`; `approx.set_par_type(ParametrizationType::Centripetal)`.
    pub fn set_par_type(&mut self, par_type: ParametrizationType) {
        self.paramtype = par_type;
    }
    /// `SetContinuity`; `approx.set_continuity(Shape::C1)`.
    pub fn set_continuity(&mut self, continuity: Shape) {
        self.continuity = continuity;
    }
    /// `SetCriteriumWeight`: the weights of the smoothing criteria; a negative one
    /// raises `Standard_DomainError`; `approx.set_criterium_weight(0.5, 0.2, 0.3)?`.
    pub fn set_criterium_weight(&mut self, w1: f64, w2: f64, w3: f64) -> Result<(), Failure> {
        if w1 < 0.0 || w2 < 0.0 || w3 < 0.0 {
            return Err(errors::domain(&format!(
                "AppBlend_AppSurf::SetCriteriumWeight: ({w1}, {w2}, {w3}); expected non-negative weights"
            )));
        }
        self.critweights = [w1, w2, w3];
        Ok(())
    }
    /// `ParType()`; `approx.par_type()`.
    pub fn par_type(&self) -> ParametrizationType {
        self.paramtype
    }
    /// `Continuity()`; `approx.continuity()`.
    pub fn continuity(&self) -> Shape {
        self.continuity
    }
    /// `CriteriumWeight(W1, W2, W3)`; `approx.criterium_weight()`.
    pub fn criterium_weight(&self) -> [f64; 3] {
        self.critweights
    }
    pub(super) fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "AppBlend_AppSurf: the approximation is not done",
            ));
        }
        Ok(())
    }
    /// `NotDone`, then `Standard_DomainError` without 2d curves.
    fn check_curves2d(&self) -> Result<(), Failure> {
        self.check_done()?;
        if self.seq_poles2d.is_empty() {
            return Err(errors::domain("AppBlend_AppSurf: no 2d curve"));
        }
        Ok(())
    }
    /// `seqPoles2d(Index)`; `Standard_OutOfRange` outside `1..=NbCurves2d`.
    fn poles2d(&self, index: i32) -> Result<&Array1<'static, Point2>, Failure> {
        usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.seq_poles2d.get(i))
            .ok_or_else(|| {
                errors::out_of_range(&format!(
                    "AppBlend_AppSurf: 2d curve {index}; expected 1..={}",
                    self.seq_poles2d.len()
                ))
            })
    }
}

impl AppBlendApprox for AppBlendAppSurf {
    fn is_done(&self) -> bool {
        self.done
    }
    fn surf_shape(&self) -> Result<[i32; 6], Failure> {
        self.check_done()?;
        let (poles, knots) = (&self.tab_poles, (&self.tab_u_knots, &self.tab_v_knots));
        let (nb_u, nb_v) = (poles.col_length(), poles.row_length());
        Ok([
            self.udeg,
            self.vdeg,
            nb_u,
            nb_v,
            knots.0.length(),
            knots.1.length(),
        ])
    }
    fn surface(&self) -> Result<AppBlendSurface, Failure> {
        self.check_done()?;
        Ok(AppBlendSurface {
            poles: self.tab_poles.clone(),
            weights: self.tab_weights.clone(),
            u_knots: self.tab_u_knots.clone(),
            v_knots: self.tab_v_knots.clone(),
            u_mults: self.tab_u_mults.clone(),
            v_mults: self.tab_v_mults.clone(),
        })
    }
    fn u_degree(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.udeg)
    }
    fn v_degree(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.vdeg)
    }
    fn surf_poles(&self) -> Result<&Array2<'static, Point3>, Failure> {
        self.check_done()?;
        Ok(&self.tab_poles)
    }
    fn surf_weights(&self) -> Result<&Array2<'static, f64>, Failure> {
        self.check_done()?;
        Ok(&self.tab_weights)
    }
    fn surf_u_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.check_done()?;
        Ok(&self.tab_u_knots)
    }
    fn surf_v_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.check_done()?;
        Ok(&self.tab_v_knots)
    }
    fn surf_u_mults(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.check_done()?;
        Ok(&self.tab_u_mults)
    }
    fn surf_v_mults(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.check_done()?;
        Ok(&self.tab_v_mults)
    }
    fn nb_curves2d(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.seq_poles2d.len() as i32)
    }
    /// Quirk kept: the pole count is the U pole count of the surface.
    fn curves2d_shape(&self) -> Result<[i32; 3], Failure> {
        self.check_curves2d()?;
        Ok([
            self.vdeg,
            self.tab_poles.col_length(),
            self.tab_v_knots.length(),
        ])
    }
    fn curve2d(&self, index: i32) -> Result<AppBlendCurve2d, Failure> {
        self.check_curves2d()?;
        Ok(AppBlendCurve2d {
            poles: self.poles2d(index)?.clone(),
            knots: self.tab_v_knots.clone(),
            mults: self.tab_v_mults.clone(),
        })
    }
    fn curves2d_degree(&self) -> Result<i32, Failure> {
        self.check_curves2d()?;
        Ok(self.vdeg)
    }
    fn curve2d_poles(&self, index: i32) -> Result<&Array1<'static, Point2>, Failure> {
        self.check_curves2d()?;
        self.poles2d(index)
    }
    fn curves2d_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.check_curves2d()?;
        Ok(&self.tab_v_knots)
    }
    fn curves2d_mults(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.check_curves2d()?;
        Ok(&self.tab_v_mults)
    }
    fn tol_reached(&self) -> Result<(f64, f64), Failure> {
        self.check_done()?;
        Ok((self.tol3dreached, self.tol2dreached))
    }
    /// The 3d tolerance reached, whatever the curve ("On ne s'embete pas !!").
    fn tol_curve_on_surf(&self, _index: i32) -> f64 {
        self.tol3dreached
    }
}
