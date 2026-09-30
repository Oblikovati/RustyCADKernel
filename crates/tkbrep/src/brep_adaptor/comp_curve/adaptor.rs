// Rust translation of the Adaptor3d_Curve members of BRepAdaptor_CompCurve
// (ModelingData/TKBRep/BRepAdaptor/BRepAdaptor_CompCurve.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Adaptor3d_Curve` members of `BRepAdaptor_CompCurve`.
use super::{BRepAdaptorCompCurve, CompState};
use crate::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3,
};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3};

impl BRepAdaptorCompCurve {
    /// A snapshot of the fields: the members call the edge adaptors without the lock.
    fn snapshot(&self) -> CompState {
        self.state.read().clone()
    }

    /// The first edge adaptor, which the canonical-kind members read.
    fn first_curve(&self) -> Result<Handle<BRepAdaptorCurve>, Failure> {
        let state = self.snapshot();
        state.curve(1)?;
        Ok(state.curves()?[0].clone())
    }
}

/// `T(index) = value` with the native index check.
fn set_bound(bounds: &mut Array1<'_, f64>, index: i32, value: f64) -> Result<(), Failure> {
    bounds
        .set_value(index, value)
        .map_err(|error| error.into_failure("BRepAdaptor_CompCurve::Intervals"))
}

/// The interval bounds of the edge adaptor of `index` (`Ti`).
fn edge_intervals(state: &CompState, index: i32, shape: Shape) -> Result<Vec<f64>, Failure> {
    let curve = state.curve(index)?;
    let count = curve.nb_intervals(shape)?;
    let mut bounds: Array1<'static, f64> = Array1::with_bounds(1, count + 1)
        .map_err(|error| error.into_failure("BRepAdaptor_CompCurve::Intervals"))?;
    curve.intervals(&mut bounds, shape)?;
    Ok(bounds.as_slice().to_vec())
}

impl Adaptor3dCurve for BRepAdaptorCompCurve {
    /// `ShallowCopy()`: shallow copies of the edge adaptors, the knots shared.
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let mut state = self.snapshot();
        let mut copies = Vec::new();
        for curve in state.curves()? {
            let copy = curve
                .get()
                .map(|curve| curve.shallow_copy())
                .transpose()?
                .unwrap_or_default();
            copies.push(copy.cast::<BRepAdaptorCurve>());
        }
        state.curves = Some(copies);
        Ok(Handle::new(Self::from_state(state)).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().t_first)
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().t_last)
    }
    /// C0 for several edges, else the continuity of the only edge.
    fn continuity(&self) -> Result<Shape, Failure> {
        let state = self.snapshot();
        if state.curves()?.len() > 1 {
            return Ok(Shape::C0);
        }
        state.curve(1)?.continuity()
    }
    /// The sum of the interval counts of the edges.
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let state = self.snapshot();
        let mut count = 0;
        for index in 1..=state.curves()?.len() as i32 {
            count += state.curve(index)?.nb_intervals(shape)?;
        }
        Ok(count)
    }
    /// The interval bounds of each edge mapped onto the wire parameter (in reverse order
    /// for an edge parsed backwards), the shared bound written once; the output is
    /// indexed from 1 whatever its lower bound, as natively.
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let state = self.snapshot();
        let mut k = 1;
        for index in 1..=state.curves()?.len() as i32 {
            let edge_bounds = edge_intervals(&state, index, shape)?;
            let (first, delta) = state.inv_prepare(index)?;
            let start = state.knot(index)?;
            // The first edge writes all its bounds; each next one skips its first bound.
            let skip = usize::from(index > 1);
            let mapped: Vec<f64> = if delta < 0.0 {
                edge_bounds.iter().rev().skip(skip).copied().collect()
            } else {
                edge_bounds.iter().skip(skip).copied().collect()
            };
            for value in mapped {
                set_bound(bounds, k, start + (value - first) * delta)?;
                k += 1;
            }
        }
        Ok(())
    }
    /// `BRepAdaptor_CompCurve(myWire, IsbyAC, First, Last, Tol)`.
    fn trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let state = self.snapshot();
        let trimmed = Self::with_range(&state.wire, state.is_by_ac, first, last, tolerance)?;
        Ok(Handle::new(trimmed).cast())
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        self.state.read().wire.closed()
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    /// `TLast - TFirst`, although the curve is never periodic.
    fn period(&self) -> Result<f64, Failure> {
        let state = self.state.read();
        Ok(state.t_last - state.t_first)
    }
    /// The smallest resolution of the edges (1e200 when there is none).
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        let state = self.snapshot();
        let mut result = 1.0e200;
        for index in 1..=state.curves()?.len() as i32 {
            let r = state.curve(index)?.resolution(r3d)?;
            if r < result {
                result = r;
            }
        }
        Ok(result)
    }
    /// Always `GeomAbs_OtherCurve` ("temporary" natively).
    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(CurveType::OtherCurve)
    }
    fn line(&self) -> Result<Line3, Failure> {
        self.first_curve()?.line()
    }
    fn circle(&self) -> Result<Circle3, Failure> {
        self.first_curve()?.circle()
    }
    fn ellipse(&self) -> Result<Ellipse3, Failure> {
        self.first_curve()?.ellipse()
    }
    fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        self.first_curve()?.hyperbola()
    }
    fn parabola(&self) -> Result<Parabola3, Failure> {
        self.first_curve()?.parabola()
    }
    fn degree(&self) -> Result<i32, Failure> {
        self.first_curve()?.degree()
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        self.first_curve()?.is_rational()
    }
    fn nb_poles(&self) -> Result<i32, Failure> {
        self.first_curve()?.nb_poles()
    }
    fn nb_knots(&self) -> Result<i32, Failure> {
        self.first_curve()?.nb_knots()
    }
    fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
        self.first_curve()?.bezier()
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        self.first_curve()?.bspline()
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let state = self.snapshot();
        let located = state.prepare(u, state.cur_index)?;
        state.curve(located.index)?.eval_d0(located.parameter)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let state = self.snapshot();
        let located = state.prepare(u, state.cur_index)?;
        let mut result = state.curve(located.index)?.eval_d1(located.parameter)?;
        result.d1 *= located.delta;
        Ok(result)
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let state = self.snapshot();
        let located = state.prepare(u, state.cur_index)?;
        let d = located.delta;
        let mut result = state.curve(located.index)?.eval_d2(located.parameter)?;
        result.d1 *= d;
        result.d2 *= d * d;
        Ok(result)
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let state = self.snapshot();
        let located = state.prepare(u, state.cur_index)?;
        let d = located.delta;
        let mut result = state.curve(located.index)?.eval_d3(located.parameter)?;
        result.d1 *= d;
        result.d2 *= d * d;
        result.d3 *= d * d * d;
        Ok(result)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        let state = self.snapshot();
        let located = state.prepare(u, state.cur_index)?;
        let derivative = state.curve(located.index)?.eval_dn(located.parameter, n)?;
        // `std::pow(d, theN)` promotes the exponent to double: the libm `pow`, not `powi`.
        Ok(derivative * located.delta.powf(f64::from(n)))
    }
}
