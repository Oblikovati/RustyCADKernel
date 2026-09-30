// Rust translation of the Adaptor2d_Curve2d members of ProjLib_ProjectedCurve
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Adaptor2d_Curve2d` interface of `ProjLib_ProjectedCurve`: the bounds of the
//! 3D curve and the kind and data of the result. The continuity, intervals, trim,
//! closure, period and resolution raise Standard_NotImplemented, as natively, and
//! the point evaluators keep the Standard_NotImplemented of the base class.
use super::perform::live_curve;
use super::{ProjectedCurve, ProjectedState};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};

fn not_implemented(member: &str) -> Failure {
    failure(
        "Standard_NotImplemented",
        &format!("ProjLib_ProjectedCurve::{member}() - method is not implemented"),
    )
}

impl ProjectedCurve {
    // The spline result when the kind is one of `kinds`, Standard_NoSuchObject
    // otherwise (the native `Standard_NoSuchObject_Raise_if`).
    fn spline_state(&self, kinds: &[CurveType], member: &str) -> Result<ProjectedState, Failure> {
        let state: ProjectedState = self.snapshot();
        if !kinds.contains(&state.result.get_type()) {
            return Err(failure(
                "Standard_NoSuchObject",
                &format!("ProjLib_ProjectedCurve:{member}"),
            ));
        }
        Ok(state)
    }
}

const SPLINES: [CurveType; 2] = [CurveType::BSplineCurve, CurveType::BezierCurve];

fn bspline_of(state: &ProjectedState) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let spline = state.result.bspline();
    if spline.is_null() {
        return Err(failure(
            "Standard_NullObject",
            "ProjLib_ProjectedCurve: the B-spline result is null",
        ));
    }
    Ok(spline)
}

fn bezier_of(state: &ProjectedState) -> Result<Handle<Geom2dBezierCurve>, Failure> {
    let bezier = state.result.bezier();
    if bezier.is_null() {
        return Err(failure(
            "Standard_NullObject",
            "ProjLib_ProjectedCurve: the Bezier result is null",
        ));
    }
    Ok(bezier)
}

impl Adaptor2dCurve2d for ProjectedCurve {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        Ok(Handle::new(self.shallow_projection()?).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        live_curve(&self.snapshot().curve)?.first_parameter()
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        live_curve(&self.snapshot().curve)?.last_parameter()
    }
    fn continuity(&self) -> Result<Shape, Failure> {
        Err(not_implemented("Continuity"))
    }
    fn nb_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Err(not_implemented("NbIntervals"))
    }
    fn intervals(&self, _bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        Err(not_implemented("Intervals"))
    }
    fn trim(
        &self,
        _first: f64,
        _last: f64,
        _tol: f64,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        Err(not_implemented("Trim"))
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        Err(not_implemented("IsClosed"))
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        Ok(self.snapshot().result.is_periodic())
    }
    fn period(&self) -> Result<f64, Failure> {
        Err(not_implemented("Period"))
    }
    fn resolution(&self, _r3d: f64) -> Result<f64, Failure> {
        Err(not_implemented("Resolution"))
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(self.snapshot().result.get_type())
    }
    fn line(&self) -> Result<Line2, Failure> {
        self.snapshot().result.line()
    }
    fn circle(&self) -> Result<Circle2, Failure> {
        self.snapshot().result.circle()
    }
    fn ellipse(&self) -> Result<Ellipse2, Failure> {
        self.snapshot().result.ellipse()
    }
    fn hyperbola(&self) -> Result<Hyperbola2, Failure> {
        self.snapshot().result.hyperbola()
    }
    fn parabola(&self) -> Result<Parabola2, Failure> {
        self.snapshot().result.parabola()
    }
    fn degree(&self) -> Result<i32, Failure> {
        let state: ProjectedState = self.spline_state(&SPLINES, "Degree")?;
        if state.result.get_type() == CurveType::BSplineCurve {
            return Ok(bspline_of(&state)?.degree());
        }
        Ok(bezier_of(&state)?.degree())
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        let state: ProjectedState = self.spline_state(&SPLINES, "IsRational")?;
        if state.result.get_type() == CurveType::BSplineCurve {
            return Ok(bspline_of(&state)?.is_rational());
        }
        Ok(bezier_of(&state)?.is_rational())
    }
    fn nb_poles(&self) -> Result<i32, Failure> {
        let state: ProjectedState = self.spline_state(&SPLINES, "NbPoles")?;
        if state.result.get_type() == CurveType::BSplineCurve {
            return Ok(bspline_of(&state)?.nb_poles());
        }
        Ok(bezier_of(&state)?.nb_poles())
    }
    fn nb_knots(&self) -> Result<i32, Failure> {
        let state: ProjectedState = self.spline_state(&[CurveType::BSplineCurve], "NbKnots")?;
        Ok(bspline_of(&state)?.nb_knots())
    }
    /// The stored Bezier, not copied nor trimmed.
    fn bezier(&self) -> Result<Handle<Geom2dBezierCurve>, Failure> {
        Ok(self.snapshot().result.bezier())
    }
    /// The stored B-spline, not copied nor trimmed.
    fn bspline(&self) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        Ok(self.snapshot().result.bspline())
    }
}
