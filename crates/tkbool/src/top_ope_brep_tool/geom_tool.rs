// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_OutCurveType.hxx and
// TopOpeBRepTool_GeomTool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_GeomTool`: the kind of the 3d curves the topological operations
//! build (B-spline of degree 1, approximation or interpolation), whether the 3d curves
//! and the p-curves on both faces are computed, and the approximation tolerances.
use rustycad_tkernel::precision;

/// The kind of the computed 3d curves (`TopOpeBRepTool_OutCurveType`);
/// `TopOpeBRepToolOutCurveType::Approx as i32 == 1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum TopOpeBRepToolOutCurveType {
    /// `TopOpeBRepTool_BSPLINE1`: a B-spline of degree 1 through the points.
    #[default]
    BSpline1 = 0,
    /// `TopOpeBRepTool_APPROX`: an approximation.
    Approx = 1,
    /// `TopOpeBRepTool_INTERPOL`: an interpolation.
    Interpol = 2,
}

/// The curve options of the topological operations (`TopOpeBRepTool_GeomTool`);
/// `let gt = TopOpeBRepToolGeomTool::new(TopOpeBRepToolOutCurveType::Approx, true, true, true);`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopOpeBRepToolGeomTool {
    /// `myTypeC3D` (protected).
    pub my_type_c3d: TopOpeBRepToolOutCurveType,
    /// `myCompC3D` (protected).
    pub my_comp_c3d: bool,
    /// `myCompPC1` (protected).
    pub my_comp_pc1: bool,
    /// `myCompPC2` (protected).
    pub my_comp_pc2: bool,
    my_tol3d: f64,
    my_tol2d: f64,
    my_nb_pnt_max: i32,
}

impl Default for TopOpeBRepToolGeomTool {
    /// The native defaults: B-spline of degree 1 and every curve computed.
    fn default() -> Self {
        Self::new(TopOpeBRepToolOutCurveType::BSpline1, true, true, true)
    }
}

impl TopOpeBRepToolGeomTool {
    /// `TopOpeBRepTool_GeomTool(TypeC3D, CompC3D, CompPC1, CompPC2)` (natively
    /// defaulted to BSPLINE1 and true): the tolerances Precision::Approximation() and
    /// Precision::PApproximation(), 30 points at most.
    pub fn new(
        type_c3d: TopOpeBRepToolOutCurveType,
        comp_c3d: bool,
        comp_pc1: bool,
        comp_pc2: bool,
    ) -> Self {
        Self {
            my_type_c3d: type_c3d,
            my_comp_c3d: comp_c3d,
            my_comp_pc1: comp_pc1,
            my_comp_pc2: comp_pc2,
            my_tol3d: precision::approximation(),
            my_tol2d: precision::p_approximation(),
            my_nb_pnt_max: 30,
        }
    }

    /// `Define(TypeC3D, CompC3D, CompPC1, CompPC2)`; `gt.define(TopOpeBRepToolOutCurveType::Approx, true, false, true)`.
    pub fn define(
        &mut self,
        type_c3d: TopOpeBRepToolOutCurveType,
        comp_c3d: bool,
        comp_pc1: bool,
        comp_pc2: bool,
    ) {
        (
            self.my_type_c3d,
            self.my_comp_c3d,
            self.my_comp_pc1,
            self.my_comp_pc2,
        ) = (type_c3d, comp_c3d, comp_pc1, comp_pc2);
    }

    /// `Define(TypeC3D)`; `gt.define_type(TopOpeBRepToolOutCurveType::Interpol)`.
    pub fn define_type(&mut self, type_c3d: TopOpeBRepToolOutCurveType) {
        self.my_type_c3d = type_c3d;
    }

    /// `DefineCurves(CompC3D)`; `gt.define_curves(false)`.
    pub fn define_curves(&mut self, comp_c3d: bool) {
        self.my_comp_c3d = comp_c3d;
    }

    /// `DefinePCurves1(CompPC1)`; `gt.define_pcurves1(false)`.
    pub fn define_pcurves1(&mut self, comp_pc1: bool) {
        self.my_comp_pc1 = comp_pc1;
    }

    /// `DefinePCurves2(CompPC2)`; `gt.define_pcurves2(false)`.
    pub fn define_pcurves2(&mut self, comp_pc2: bool) {
        self.my_comp_pc2 = comp_pc2;
    }

    /// `Define(GT)`: a copy of every option and tolerance; `gt.define_from(&other)`.
    pub fn define_from(&mut self, gt: &TopOpeBRepToolGeomTool) {
        *self = *gt;
    }

    /// `GetTolerances(tol3d, tol2d)`; `let (t3, t2) = gt.get_tolerances();`.
    pub fn get_tolerances(&self) -> (f64, f64) {
        (self.my_tol3d, self.my_tol2d)
    }

    /// `SetTolerances(tol3d, tol2d)`; `gt.set_tolerances(1e-7, 1e-9)`.
    pub fn set_tolerances(&mut self, tol3d: f64, tol2d: f64) {
        (self.my_tol3d, self.my_tol2d) = (tol3d, tol2d);
    }

    /// `NbPntMax()`; `gt.nb_pnt_max()`.
    pub fn nb_pnt_max(&self) -> i32 {
        self.my_nb_pnt_max
    }

    /// `SetNbPntMax(NbPntMax)`; `gt.set_nb_pnt_max(50)`.
    pub fn set_nb_pnt_max(&mut self, nb_pnt_max: i32) {
        self.my_nb_pnt_max = nb_pnt_max;
    }

    /// `TypeC3D()`; `gt.type_c3d()`.
    pub fn type_c3d(&self) -> TopOpeBRepToolOutCurveType {
        self.my_type_c3d
    }

    /// `CompC3D()`; `gt.comp_c3d()`.
    pub fn comp_c3d(&self) -> bool {
        self.my_comp_c3d
    }

    /// `CompPC1()`; `gt.comp_pc1()`.
    pub fn comp_pc1(&self) -> bool {
        self.my_comp_pc1
    }

    /// `CompPC2()`; `gt.comp_pc2()`.
    pub fn comp_pc2(&self) -> bool {
        self.my_comp_pc2
    }
}
