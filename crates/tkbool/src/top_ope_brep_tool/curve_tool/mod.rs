// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_CurveTool.hxx/.cxx
// (the constructors, the accessors, MakeBSpline1fromPnt(2d), IsProjectable and the static
// MakePCurve) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_CurveTool`: the curves of the section edges of the topological
//! operations (approximated from the intersection lines, or projected on the faces).
mod curvature;
mod improve;
mod make_curves;
mod pcurve;

use super::{TopOpeBRepToolGeomTool, TopOpeBRepToolOutCurveType};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine,
    Geom2dParabola,
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Point2, Point3};

pub use make_curves::{MadeCurves, SectionCurves};

/// The curve tool of the topological operations (`TopOpeBRepTool_CurveTool`);
/// `let ct = TopOpeBRepToolCurveTool::with_type(TopOpeBRepToolOutCurveType::Approx);`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TopOpeBRepToolCurveTool {
    /// `myGeomTool` (protected).
    pub my_geom_tool: TopOpeBRepToolGeomTool,
}

/// An array error as a native exception.
pub(super) fn array_failure(e: impl std::fmt::Debug) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("TopOpeBRepTool_CurveTool: {e:?}; expected an index in the array bounds"),
        "NCollection_Array1",
    )
}

/// The knots 1..=n (the point index as parameter) and multiplicities 2, 1..., 2 of a
/// degree-1 B-spline through n points.
fn index_knots(nbknots: i32) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, nbknots).map_err(array_failure)?;
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, nbknots).map_err(array_failure)?;
    for i in 1..=nbknots {
        knots[i] = f64::from(i);
        mults[i] = 1;
    }
    (mults[1], mults[nbknots]) = (2, 2);
    Ok((knots, mults))
}

impl TopOpeBRepToolCurveTool {
    /// `TopOpeBRepTool_CurveTool()`: the default geometric tool;
    /// `TopOpeBRepToolCurveTool::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepTool_CurveTool(OCT)`: the default geometric tool of that curve type.
    pub fn with_type(oct: TopOpeBRepToolOutCurveType) -> Self {
        Self::with_geom_tool(&TopOpeBRepToolGeomTool::new(oct, true, true, true))
    }

    /// `TopOpeBRepTool_CurveTool(GT)`: a copy of the geometric tool.
    pub fn with_geom_tool(gt: &TopOpeBRepToolGeomTool) -> Self {
        let mut tool = Self::default();
        tool.set_geom_tool(gt);
        tool
    }

    /// `ChangeGeomTool()`; `ct.change_geom_tool().set_nb_pnt_max(40)`.
    pub fn change_geom_tool(&mut self) -> &mut TopOpeBRepToolGeomTool {
        &mut self.my_geom_tool
    }

    /// `GetGeomTool()`; `ct.get_geom_tool()`.
    pub fn get_geom_tool(&self) -> &TopOpeBRepToolGeomTool {
        &self.my_geom_tool
    }

    /// `SetGeomTool(GT)`; `ct.set_geom_tool(&gt)`.
    pub fn set_geom_tool(&mut self, gt: &TopOpeBRepToolGeomTool) {
        self.my_geom_tool.define_from(gt);
    }

    /// `MakeBSpline1fromPnt(P)`: the B-spline of degree 1 through the points, the point
    /// index as parameter; `TopOpeBRepToolCurveTool::make_bspline1_from_pnt(&points)?`.
    pub fn make_bspline1_from_pnt(
        points: &Array1<'_, Point3>,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        let (knots, mults) = index_knots(points.length())?;
        Ok(Handle::new(GeomBSplineCurve::new(points, &knots, &mults, 1, false)?).cast())
    }

    /// `MakeBSpline1fromPnt2d(P)`: the 2d B-spline of degree 1 through the points;
    /// `TopOpeBRepToolCurveTool::make_bspline1_from_pnt2d(&points)?`.
    pub fn make_bspline1_from_pnt2d(
        points: &Array1<'_, Point2>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let (knots, mults) = index_knots(points.length())?;
        Ok(Handle::new(Geom2dBSplineCurve::new(points, &knots, &mults, 1, false)?).cast())
    }

    /// `IsProjectable(S, C)`: false for a conic on a cone, an ellipse on a cylinder and
    /// a circle on a sphere or a torus (their projections may be partial);
    /// `TopOpeBRepToolCurveTool::is_projectable(&face, &curve)?`.
    pub fn is_projectable(s: &Shape, c3d: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
        let f = topods::face(s)?;
        let bas = BRepAdaptorSurface::with_face(&f, false)?;
        let suty = bas.get_type()?;
        let cuty = GeomAdaptorCurve::with_curve(c3d)?.get_type()?;
        Ok(match suty {
            SurfaceType::Cone => !matches!(
                cuty,
                CurveType::Ellipse | CurveType::Hyperbola | CurveType::Parabola
            ),
            SurfaceType::Cylinder => cuty != CurveType::Ellipse,
            SurfaceType::Sphere | SurfaceType::Torus => cuty != CurveType::Circle,
            _ => true,
        })
    }
}

/// `MakePCurve(PC)`: the 2d curve of a projected curve (a line, a conic or its
/// B-spline); Standard_NotImplemented for another type.
pub fn make_pcurve(pc: &ProjectedCurve) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(match pc.get_type()? {
        CurveType::Line => Handle::new(Geom2dLine::from_lin2d(pc.line()?)).cast(),
        CurveType::Circle => Handle::new(Geom2dCircle::from_circ2d(pc.circle()?)).cast(),
        CurveType::Ellipse => Handle::new(Geom2dEllipse::from_elips2d(pc.ellipse()?)).cast(),
        CurveType::Parabola => Handle::new(Geom2dParabola::from_parab2d(pc.parabola()?)).cast(),
        CurveType::Hyperbola => Handle::new(Geom2dHyperbola::from_hypr2d(pc.hyperbola()?)).cast(),
        CurveType::BSplineCurve => pc.bspline()?.cast(),
        _ => {
            return Err(Failure::with_type(
                "Standard_NotImplemented",
                "CurveTool::MakePCurve",
                "TopOpeBRepTool_CurveTool",
            ));
        }
    })
}
