// Rust translation of Extrema_GenExtPS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GenExtPS`: all the extrema of the distance between a point and a
//! surface, from the extrema of the distance to a sampling grid (or, with
//! [`ExtAlgo::Tree`], from the nearest and farthest grid nodes of a sphere tree),
//! each refined by math_FunctionSetRoot on [`FuncPSNorm`].
mod candidates;
mod grid;
mod param_grid;
mod tree;

use super::func_ps_norm::FuncPSNorm;
use crate::extrema::failures::{extremum_index, not_done, null_object};
use crate::extrema::pon::{POnSurf, POnSurfParams};
use crate::extrema::surface_ref::SurfaceRef;
use crate::extrema::{ExtAlgo, ExtFlag};
use crate::native_failure::failure;
use param_grid::ParamGrid;
use rustycad_tkernel::ncollection::UBTree;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::bnd::BoundingSphere;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{FunctionSetRoot, Vector};
use tree::TreeSphere;

/// `Extrema_GenExtPS`; `GenExtPS::with_point(point, &surface, sampling,
/// ExtFlag::MinMax, ExtAlgo::Grad)?.nb_ext()?`.
pub struct GenExtPS<'a> {
    done: bool,
    init: bool,
    umin: f64,
    usup: f64,
    vmin: f64,
    vsup: f64,
    usample: i32,
    vsample: i32,
    tolu: f64,
    tolv: f64,
    points: ParamGrid,
    sphere_tree: Option<UBTree<i32, TreeSphere>>,
    sphere_array: Vec<BoundingSphere>,
    f: FuncPSNorm<'a>,
    s: Option<SurfaceRef<'a>>,
    flag: ExtFlag,
    algo: ExtAlgo,
    u_params: Option<Vec<f64>>,
    v_params: Option<Vec<f64>>,
    face_pnt_params: ParamGrid,
    u_edge_pnt_params: ParamGrid,
    v_edge_pnt_params: ParamGrid,
}

impl Default for GenExtPS<'_> {
    fn default() -> Self {
        Self {
            done: false,
            init: false,
            umin: 0.0,
            usup: 0.0,
            vmin: 0.0,
            vsup: 0.0,
            usample: 0,
            vsample: 0,
            tolu: 0.0,
            tolv: 0.0,
            points: ParamGrid::default(),
            sphere_tree: None,
            sphere_array: Vec::new(),
            f: FuncPSNorm::new(),
            s: None,
            flag: ExtFlag::MinMax,
            algo: ExtAlgo::Grad,
            u_params: None,
            v_params: None,
            face_pnt_params: ParamGrid::default(),
            u_edge_pnt_params: ParamGrid::default(),
            v_edge_pnt_params: ParamGrid::default(),
        }
    }
}

/// The sampling of a [`GenExtPS`]: the number of samples and the parameter
/// tolerance in each direction; `Sampling { nb_u: 20, nb_v: 20, tol_u: 1e-9, tol_v: 1e-9 }`.
#[derive(Clone, Copy, Debug)]
pub struct Sampling {
    /// `NbU`.
    pub nb_u: i32,
    /// `NbV`.
    pub nb_v: i32,
    /// `TolU`.
    pub tol_u: f64,
    /// `TolV`.
    pub tol_v: f64,
}

impl<'a> GenExtPS<'a> {
    /// `Extrema_GenExtPS()`: no surface; `GenExtPS::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_GenExtPS(P, S, NbU, NbV, TolU, TolV, F, A)`: Initialize and Perform
    /// (the native defaults are MinMax and Grad);
    /// `GenExtPS::with_point(point, &surface, sampling, ExtFlag::MinMax, ExtAlgo::Grad)?`.
    pub fn with_point(
        point: Point3,
        surface: &'a dyn Adaptor3dSurface,
        sampling: Sampling,
        flag: ExtFlag,
        algo: ExtAlgo,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self {
            f: FuncPSNorm::with_point_surface(point, surface),
            flag,
            algo,
            ..Self::default()
        };
        extrema.initialize(surface, sampling)?;
        extrema.perform(point)?;
        Ok(extrema)
    }
    /// `Extrema_GenExtPS(P, S, NbU, NbV, Umin, Usup, Vmin, Vsup, TolU, TolV, F, A)`;
    /// `GenExtPS::with_range(point, &surface, sampling, (0., 1.), (0., 1.), flag, algo)?`.
    pub fn with_range(
        point: Point3,
        surface: &'a dyn Adaptor3dSurface,
        sampling: Sampling,
        u_range: (f64, f64),
        v_range: (f64, f64),
        (flag, algo): (ExtFlag, ExtAlgo),
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self {
            f: FuncPSNorm::with_point_surface(point, surface),
            flag,
            algo,
            ..Self::default()
        };
        extrema.initialize_range(surface, sampling, u_range, v_range)?;
        extrema.perform(point)?;
        Ok(extrema)
    }
    /// `Initialize(S, NbU, NbV, TolU, TolV)`: over the surface bounds;
    /// `extrema.initialize(&surface, sampling)?`.
    pub fn initialize(
        &mut self,
        surface: &'a dyn Adaptor3dSurface,
        sampling: Sampling,
    ) -> Result<(), Failure> {
        let u_range = (surface.first_u_parameter()?, surface.last_u_parameter()?);
        let v_range = (surface.first_v_parameter()?, surface.last_v_parameter()?);
        (self.umin, self.usup, self.vmin, self.vsup) = (u_range.0, u_range.1, v_range.0, v_range.1);
        self.initialize_range(surface, sampling, u_range, v_range)
    }
    /// `Initialize(S, NbU, NbV, Umin, Usup, Vmin, Vsup, TolU, TolV)`: Standard_OutOfRange
    /// for fewer than two samples in a direction; drops the grid and the tree;
    /// `extrema.initialize_range(&surface, sampling, (0., 1.), (0., 1.))?`.
    pub fn initialize_range(
        &mut self,
        surface: &'a dyn Adaptor3dSurface,
        sampling: Sampling,
        (umin, usup): (f64, f64),
        (vmin, vsup): (f64, f64),
    ) -> Result<(), Failure> {
        self.initialize_range_ref(
            SurfaceRef::Borrowed(surface),
            sampling,
            (umin, usup),
            (vmin, vsup),
        )
    }
    /// `Initialize(S, NbU, NbV, Umin, Usup, Vmin, Vsup, TolU, TolV)` on a borrowed or a
    /// shared surface.
    pub(crate) fn initialize_range_ref(
        &mut self,
        surface: SurfaceRef<'a>,
        sampling: Sampling,
        (umin, usup): (f64, f64),
        (vmin, vsup): (f64, f64),
    ) -> Result<(), Failure> {
        self.s = Some(surface.clone());
        (self.usample, self.vsample) = (sampling.nb_u, sampling.nb_v);
        (self.tolu, self.tolv) = (sampling.tol_u, sampling.tol_v);
        (self.umin, self.usup, self.vmin, self.vsup) = (umin, usup, vmin, vsup);
        if self.usample < 2 || self.vsample < 2 {
            return Err(failure(
                "Standard_OutOfRange",
                &format!(
                    "Extrema_GenExtPS::Initialize - samples ({}, {}); expected at least 2 in each direction",
                    self.usample, self.vsample
                ),
            ));
        }
        self.f.initialize_ref(surface);
        self.sphere_tree = None;
        self.u_params = None;
        self.v_params = None;
        self.init = false;
        Ok(())
    }
    /// `SetFlag(F)`; `extrema.set_flag(ExtFlag::Min)`.
    pub fn set_flag(&mut self, flag: ExtFlag) {
        self.flag = flag;
    }
    /// `SetAlgo(A)`: a new algorithm rebuilds the grid (not the parameters nor the
    /// tree); `extrema.set_algo(ExtAlgo::Tree)`.
    pub fn set_algo(&mut self, algo: ExtAlgo) {
        if self.algo != algo {
            self.init = false;
        }
        self.algo = algo;
    }
    // The surface, Standard_NullObject before Initialize (a null dereference natively).
    fn surface(&self) -> Result<&dyn Adaptor3dSurface, Failure> {
        match &self.s {
            Some(surface) => Ok(surface.get()),
            None => Err(null_object("Extrema_GenExtPS::Perform()")),
        }
    }
    // A copy of the surface reference, for the members that change the grid.
    fn surface_ref(&self) -> Result<SurfaceRef<'a>, Failure> {
        self.s
            .clone()
            .ok_or_else(|| null_object("Extrema_GenExtPS::Perform()"))
    }
    /// `Perform(P)`; `extrema.perform(point)?`.
    pub fn perform(&mut self, point: Point3) -> Result<(), Failure> {
        self.done = false;
        self.f.set_point(point);
        if self.algo == ExtAlgo::Grad {
            self.build_grid(point)?;
            if matches!(self.flag, ExtFlag::Min | ExtFlag::MinMax) {
                self.find_grid_minima()?;
            }
            if matches!(self.flag, ExtFlag::Max | ExtFlag::MinMax) {
                self.find_grid_maxima()?;
            }
            return Ok(());
        }
        self.build_tree()?;
        if matches!(self.flag, ExtFlag::Min | ExtFlag::MinMax) {
            self.find_tree_extremum(point, true)?;
        }
        if matches!(self.flag, ExtFlag::Max | ExtFlag::MinMax) {
            self.find_tree_extremum(point, false)?;
        }
        Ok(())
    }
    // `FindSolution(P, theParams)`: math_FunctionSetRoot from the parameters of
    // `params` within the bounds; the function saves the root. Done in any case.
    fn find_solution(&mut self, params: &POnSurfParams) -> Result<(), Failure> {
        let mut tol: Vector<'static> = Vector::with_bounds(1, 2)?;
        (tol[1], tol[2]) = (self.tolu, self.tolv);
        let mut uv: Vector<'static> = Vector::with_bounds(1, 2)?;
        (uv[1], uv[2]) = params.parameter();
        let mut uv_inf: Vector<'static> = Vector::with_bounds(1, 2)?;
        let mut uv_sup: Vector<'static> = Vector::with_bounds(1, 2)?;
        (uv_inf[1], uv_inf[2], uv_sup[1], uv_sup[2]) = (self.umin, self.vmin, self.usup, self.vsup);
        let mut solver = FunctionSetRoot::new(&self.f, &tol, 100)?;
        solver.perform_bounded(&mut self.f, &uv, &uv_inf, &uv_sup, false)?;
        self.f.take_failure()?;
        self.done = true;
        Ok(())
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GenExtPS::NbExt()"));
        }
        Ok(self.f.nb_ext())
    }
    /// `SquareDistance(N)`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        extremum_index("Extrema_GenExtPS::SquareDistance()", n, || self.nb_ext())?;
        self.f.square_distance(n)
    }
    /// `Point(N)`; `extrema.point(1)?`.
    pub fn point(&self, n: i32) -> Result<POnSurf, Failure> {
        extremum_index("Extrema_GenExtPS::Point()", n, || self.nb_ext())?;
        self.f.point(n)
    }
}
