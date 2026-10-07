//! The 2D curve-curve intersection of HLRBRep: HLRBRep_CInter and its 12 helper
//! classes, the instances of the IntCurve, IntImpParGen and Extrema templates on
//! [`CurveTool`] (HLRBRep_CInter_0.cxx and the `*OfCInter*` headers), OCCT
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. docs/geomalgo-design.md sections 3-5
//! describe the generics; only the projection of a point is a class of its own.
use super::{Curve, CurveTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_curve::{
    DistBetweenPCurvesGen, ExactIntersectionPoint, IConicTool, IntConicCurveGen, IntCurveCurveGen,
    IntPolyPolyGen, Polygon2dGen, UserIntConicCurveGen,
};
use rustycad_tkgeomalgo::int_imp_par_gen::{
    Intersector, IntersectorTypes, MyImpParTool, ProjectOnPCurveTool,
};
use rustycad_tkgeombase::extrema::{GCurveLocator, GFuncExtPC, GenLocateExtPC, POnCurv2d};
use rustycad_tkmath::gp::Point2;

/// The IntImpParGen instantiation of HLRBRep: a gp conic ([`IConicTool`]) against a
/// projected [`Curve`] ([`CurveTool`], [`TheProjPCurOfCInter`]).
pub struct CInterTypes;

impl IntersectorTypes for CInterTypes {
    type Imp = IConicTool;
    type Par = CurveTool;
    type Project = TheProjPCurOfCInter;
}

/// `HLRBRep_TheCurveLocatorOfTheProjPCurOfCInter`: Extrema_GCurveLocator on [`CurveTool`].
pub type TheCurveLocatorOfTheProjPCurOfCInter = GCurveLocator<CurveTool>;
/// `HLRBRep_TheLocateExtPCOfTheProjPCurOfCInter`: Extrema_GenLocateExtPC on [`CurveTool`].
pub type TheLocateExtPCOfTheProjPCurOfCInter<'a> = GenLocateExtPC<'a, CurveTool>;
/// `HLRBRep_PCLocFOfTheLocateExtPCOfTheProjPCurOfCInter`: Extrema_GFuncExtPC on [`CurveTool`].
pub type PCLocFOfTheLocateExtPCOfTheProjPCurOfCInter<'a> = GFuncExtPC<'a, CurveTool>;
/// `HLRBRep_TheIntersectorOfTheIntConicCurveOfCInter`: IntImpParGen_Intersector.
pub type TheIntersectorOfTheIntConicCurveOfCInter = Intersector<CInterTypes>;
/// `HLRBRep_MyImpParToolOfTheIntersectorOfTheIntConicCurveOfCInter`.
pub type MyImpParToolOfTheIntersectorOfTheIntConicCurveOfCInter<'a> = MyImpParTool<'a, CInterTypes>;
/// `HLRBRep_TheIntConicCurveOfCInter`: IntCurve_IntConicCurveGen.
pub type TheIntConicCurveOfCInter = IntConicCurveGen<CInterTypes>;
/// `HLRBRep_IntConicCurveOfCInter`: IntCurve_UserIntConicCurveGen.
pub type IntConicCurveOfCInter = UserIntConicCurveGen<CInterTypes>;
/// `HLRBRep_TheDistBetweenPCurvesOfTheIntPCurvePCurveOfCInter`.
pub type TheDistBetweenPCurvesOfTheIntPCurvePCurveOfCInter<'a> =
    DistBetweenPCurvesGen<'a, CurveTool>;
/// `HLRBRep_ThePolygon2dOfTheIntPCurvePCurveOfCInter`: IntCurve_Polygon2dGen.
pub type ThePolygon2dOfTheIntPCurvePCurveOfCInter = Polygon2dGen<CurveTool>;
/// `HLRBRep_ExactIntersectionPointOfTheIntPCurvePCurveOfCInter`.
pub type ExactIntersectionPointOfTheIntPCurvePCurveOfCInter<'a> =
    ExactIntersectionPoint<'a, CurveTool>;
/// `HLRBRep_TheIntPCurvePCurveOfCInter`: IntCurve_IntPolyPolyGen.
pub type TheIntPCurvePCurveOfCInter = IntPolyPolyGen<CInterTypes>;
/// `HLRBRep_CInter`: IntCurve_IntCurveCurveGen on projected curves.
pub type CInter = IntCurveCurveGen<CInterTypes>;

/// HLRBRep_TheProjPCurOfCInter: the parameter of a point near a projected curve, the
/// nearest of `NbSamples(C)` samples between the bounds refined by Extrema's local
/// search (kept when that fails or finds a maximum); the tolerance is unused;
/// `TheProjPCurOfCInter::find_parameter(&curve, point, 1e-7)?`.
pub struct TheProjPCurOfCInter;

impl TheProjPCurOfCInter {
    /// FindParameter(C, Pnt, LowParameter, HighParameter, Tol);
    /// `TheProjPCurOfCInter::find_parameter_in(&curve, point, 0.0, 1.0, 1e-7)?`.
    pub fn find_parameter_in(
        c: &Curve,
        p: Point2,
        low_parameter: f64,
        high_parameter: f64,
        _tol: f64,
    ) -> Result<f64, Failure> {
        let nb_pts = CurveTool::nb_samples(c)?;
        let the_eps_x = CurveTool::eps_x(c);
        let pon_c: POnCurv2d = TheCurveLocatorOfTheProjPCurOfCInter::locate_range(
            p,
            c,
            nb_pts,
            low_parameter,
            high_parameter,
        )?;
        let defaultparam = pon_c.parameter();
        let loc = TheLocateExtPCOfTheProjPCurOfCInter::with_curve(p, c, defaultparam, the_eps_x)?;
        if !loc.is_done() || !loc.is_min()? {
            return Ok(defaultparam);
        }
        Ok(loc.point()?.parameter())
    }

    /// FindParameter(C, Pnt, Tol): between the bounds of the curve;
    /// `TheProjPCurOfCInter::find_parameter(&curve, point, 1e-7)?`.
    pub fn find_parameter(c: &Curve, p: Point2, tol: f64) -> Result<f64, Failure> {
        let (first, last) = (
            CurveTool::first_parameter(c)?,
            CurveTool::last_parameter(c)?,
        );
        Self::find_parameter_in(c, p, first, last, tol)
    }
}

impl ProjectOnPCurveTool for TheProjPCurOfCInter {
    type Curve = Curve;
    fn find_parameter(curve: &Curve, pnt: Point2, tol: f64) -> Result<f64, Failure> {
        TheProjPCurOfCInter::find_parameter(curve, pnt, tol)
    }
    fn find_parameter_in(
        curve: &Curve,
        pnt: Point2,
        low_parameter: f64,
        high_parameter: f64,
        tol: f64,
    ) -> Result<f64, Failure> {
        TheProjPCurOfCInter::find_parameter_in(curve, pnt, low_parameter, high_parameter, tol)
    }
}
