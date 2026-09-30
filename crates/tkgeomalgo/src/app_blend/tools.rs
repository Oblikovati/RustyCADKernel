// Rust translation of the TheLine and TheSectionGenerator macros of
// ModelingAlgorithms/TKGeomAlgo/AppBlend/AppBlend_AppSurf.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The type parameters of `AppBlend_AppSurf.gxx`: the calls it makes on `TheLine`
//! and `TheSectionGenerator`, under their native names.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// `TheLine`: the ordered section points of the sweep (`GeomFill_Line`,
/// `BRepBlend_Line`); `line.point(1)`.
pub trait TheLine {
    /// What `Point(Index)` returns and the section generator reads.
    type Point;
    /// `NbPoints()`.
    fn nb_points(&self) -> i32;
    /// `Point(Index)`, 1-based.
    fn point(&self, index: i32) -> Self::Point;
}

/// The arrays a section generator fills (1-based, sized by `GetShape`): the poles
/// and weights of a section, their derivatives along the line, and the 2d poles.
#[derive(Clone)]
pub struct AppBlendSection {
    /// `Poles`.
    pub poles: Array1<'static, Point3>,
    /// `DPoles`.
    pub d_poles: Array1<'static, Vector3>,
    /// `Poles2d` (at least one entry, as natively).
    pub poles2d: Array1<'static, Point2>,
    /// `DPoles2d`.
    pub d_poles2d: Array1<'static, Vector2>,
    /// `Weigths`.
    pub weights: Array1<'static, f64>,
    /// `DWeigths`.
    pub d_weights: Array1<'static, f64>,
}

/// `TheSectionGenerator`: the section curves of the sweep at the line points, as
/// rational B-splines of one knot vector, with optional 2d curves.
pub trait TheSectionGenerator {
    /// The line point a section is computed at.
    type Point;
    /// `GetShape(NbPoles, NbKnots, Degree, NbPoles2d)`, in that order; a `Result`
    /// because a generator can raise (`GeomFill_SectionGenerator` reads its first
    /// section, Standard_OutOfRange when it has none).
    fn get_shape(&self) -> Result<[i32; 4], Failure>;
    /// `Knots(TKnots)`: the knots of the sections.
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure>;
    /// `Mults(TMults)`: the multiplicities of the sections.
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure>;
    /// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)`: the section
    /// and its derivatives along the line into `section`; false when the
    /// derivatives are not computed.
    fn section_d1(
        &mut self,
        p: &Self::Point,
        section: &mut AppBlendSection,
    ) -> Result<bool, Failure>;
    /// `Section(P, Poles, Poles2d, Weigths)`: the poles, 2d poles and weights only.
    fn section(&mut self, p: &Self::Point, section: &mut AppBlendSection) -> Result<(), Failure>;
    /// `Parameter(P)`: the sweep parameter of a line point; a `Result` because a
    /// generator can raise (`GeomFill_SectionGenerator` reads its parameter array).
    fn parameter(&self, p: &Self::Point) -> Result<f64, Failure>;
}
