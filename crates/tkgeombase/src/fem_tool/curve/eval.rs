// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_Curve.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluation of `FEmTool_Curve`: D0, D1 and D2 on the element found by a
//! cached span search.
use super::Curve;
use super::state::{CurveState, SpanCache};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::plib;

/// Which canonical coefficients an evaluation reads.
#[derive(Clone, Copy)]
enum Order {
    Value,
    First,
    Second,
}

impl Curve {
    /// `D0(U, Pnt)`: the point at `u`, written from `Pnt.Lower()`; `curve.d0(0.5, &mut pnt)?`.
    pub fn d0(&self, u: f64, pnt: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.evaluate(u, pnt, Order::Value)
    }

    /// `D1(U, Vec)`: the first derivative at `u`; every entry of `Vec` (not only
    /// the first Dimension ones) is scaled by `2 / (Ul - Uf)`; `curve.d1(0.5, &mut vec)?`.
    pub fn d1(&self, u: f64, vec: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.evaluate(u, vec, Order::First)
    }

    /// `D2(U, Vec)`: the second derivative at `u`; every entry of `Vec` is scaled
    /// by `4 / (Ul - Uf)^2`; `curve.d2(0.5, &mut vec)?`.
    pub fn d2(&self, u: f64, vec: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.evaluate(u, vec, Order::Second)
    }

    fn evaluate(&self, u: f64, out: &mut Array1<'_, f64>, order: Order) -> Result<(), Failure> {
        let mut state = self.state.write();
        self.locate(&mut state.span, u)?;
        let index: i32 = state.span.index;
        let degree: i32 = state.degree(index)?;
        let ready: bool = match order {
            Order::Value => state.has_poly[(index - 1) as usize],
            Order::First => state.has_deri[(index - 1) as usize],
            Order::Second => state.has_secn[(index - 1) as usize],
        };
        if !ready {
            self.update(&mut state, index, order as i32)?;
        }
        let span: SpanCache = state.span;
        let s: f64 = (2. * u - span.usum) * span.denom;
        let (coefficients, degree, start) = self.coefficients(&state, order, degree, index);
        let coefficients: &[f64] = start_at(coefficients, start)?;
        let dimension: i32 = self.dimension;
        plib::no_derivative_eval_polynomial(
            s,
            degree,
            dimension,
            degree * dimension,
            coefficients,
            out.as_mut_slice(),
        )?;
        let scale: f64 = match order {
            Order::Value => return Ok(()),
            Order::First => 2. * span.denom,
            Order::Second => 4. * span.denom * span.denom,
        };
        out.iter_mut().for_each(|value| *value *= scale);
        Ok(())
    }

    // The coefficient table, degree and 1-based start of the element for `order`.
    fn coefficients<'s>(
        &self,
        state: &'s CurveState,
        order: Order,
        degree: i32,
        index: i32,
    ) -> (&'s [f64], i32, i32) {
        let (dimension, work_degree) = (self.dimension, state.work_degree);
        match order {
            Order::Value => (&state.poly, degree, state.span.ptr),
            Order::First => (
                &state.deri,
                degree - 1,
                1 + (index - 1) * work_degree * dimension,
            ),
            Order::Second => (
                &state.dsecn,
                degree - 2,
                1 + (index - 1) * (work_degree - 1) * dimension,
            ),
        }
    }

    // The span search: kept while `u` is inside the cached element and its knots are
    // unchanged; else element 1 up to Knots(2), the first element containing `u`, or
    // the last element.
    fn locate(&self, span: &mut SpanCache, u: f64) -> Result<(), Failure> {
        if span.index != 0
            && u >= span.uf
            && u <= span.ul
            && self.knot(span.index)? == span.uf
            && self.knot(span.index + 1)? == span.ul
        {
            return Ok(());
        }
        span.index = if u <= self.knot(2)? {
            1
        } else {
            self.containing(u)?
        };
        span.uf = self.knot(span.index)?;
        span.ul = self.knot(span.index + 1)?;
        span.denom = 1. / (span.ul - span.uf);
        span.usum = span.uf + span.ul;
        span.ptr = (span.index - 1) * (self.base.work_degree() + 1) * self.dimension + 1;
        Ok(())
    }

    // The first element 2..=NbElements whose knots enclose `u`, else NbElements.
    fn containing(&self, u: f64) -> Result<i32, Failure> {
        for index in 2..=self.nb_elements {
            if u >= self.knot(index)? && u <= self.knot(index + 1)? {
                return Ok(index);
            }
        }
        Ok(self.nb_elements)
    }
}

// The coefficients from the 1-based `start`.
fn start_at(values: &[f64], start: i32) -> Result<&[f64], Failure> {
    usize::try_from(start - 1)
        .ok()
        .and_then(|start| values.get(start..))
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "FEmTool_Curve: coefficient address {start} outside the curve"
            ))
        })
}
