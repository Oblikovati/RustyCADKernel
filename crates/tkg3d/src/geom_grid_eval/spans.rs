// Rust translation of locateSpan and countSpanSize of the anonymous namespaces of ModelingData/TKG3d/GeomGridEval/GeomGridEval_BSplineCurve.cxx and GeomGridEval_BSplineSurface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Span location on flat knots for the B-spline grid loops. The curve and surface
//! files carry identical copies of `locateSpan` and `countSpanSize`; the port keeps
//! one. Both assume ascending parameters: for a non-periodic spline a parameter below
//! the end of the current span stays in it without a new `LocateParameter` call, and a
//! run ends at the first parameter at or above the next flat knot. Unsorted grids
//! therefore evaluate some parameters with the polynomial of another span, as natively.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::at;
use rustycad_tkmath::bsplclib;

/// The flat knots of one parametric direction with its degree and periodicity.
#[derive(Clone, Copy)]
pub(super) struct FlatKnotsView<'k> {
    pub(super) flat_knots: &'k Array1<'static, f64>,
    pub(super) degree: i32,
    pub(super) periodic: bool,
}

impl FlatKnotsView<'_> {
    /// `locateSpan`: `BSplCLib::LocateParameter` with `NoMults()` from index 0; returns
    /// the adjusted parameter and the flat-knot span index.
    pub(super) fn locate_span(&self, u: f64) -> Result<(f64, i32), Failure> {
        let (mut adjusted, mut span): (f64, i32) = (u, 0);
        bsplclib::locate_parameter_for_eval(
            self.degree,
            self.flat_knots,
            None,
            u,
            self.periodic,
            &mut span,
            &mut adjusted,
        )?;
        Ok((adjusted, span))
    }
    /// `countSpanSize`: the parameters from `start` on that stay in span `target`.
    pub(super) fn count_span_size(
        &self,
        params: &[f64],
        start: usize,
        target: i32,
    ) -> Result<i32, Failure> {
        if !self.periodic {
            return self.count_below_next_knot(params, start, target);
        }
        let mut count: i32 = 1;
        for &u in &params[start + 1..] {
            if self.locate_span(u)?.1 != target {
                break;
            }
            count += 1;
        }
        Ok(count)
    }
    // The non-periodic fast path: compare against the next flat knot only. The negated
    // comparison is the native `if (param >= aNextKnot) break;`, which a NaN never ends.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn count_below_next_knot(
        &self,
        params: &[f64],
        start: usize,
        target: i32,
    ) -> Result<i32, Failure> {
        if target + 1 > self.flat_knots.upper() {
            return Ok(1);
        }
        let next_knot: f64 = at(self.flat_knots, target + 1)?;
        let run: usize = params[start + 1..]
            .iter()
            .take_while(|&&u| !(u >= next_knot))
            .count();
        Ok(1 + run as i32)
    }
    /// The flat knot at `index` (`FlatKnots->Value(index)`, range checked).
    pub(super) fn knot(&self, index: i32) -> Result<f64, Failure> {
        at(self.flat_knots, index)
    }
}

/// The span state of one direction of a grid loop (`aPrevSpan`, `aSpanEnd`).
pub(super) struct SpanWalker<'k> {
    view: FlatKnotsView<'k>,
    prev_span: i32,
    span_end: f64,
}

/// One located parameter: the adjusted value, the span and whether it differs from the
/// span of the previous parameter.
#[derive(Clone, Copy)]
pub(super) struct SpanStep {
    pub(super) adjusted: f64,
    pub(super) span: i32,
    pub(super) changed: bool,
}

impl<'k> SpanWalker<'k> {
    /// A walker before the first parameter (`aPrevSpan = -1`, `aSpanEnd = 0`).
    pub(super) fn new(view: FlatKnotsView<'k>) -> Self {
        Self {
            view,
            prev_span: -1,
            span_end: 0.0,
        }
    }
    /// The located parameter; a non-periodic parameter below the current span end
    /// keeps the previous span without calling `LocateParameter`. On a span change the
    /// new span end is read (`FlatKnots->Value(aSpan + 1)`).
    pub(super) fn step(&mut self, u: f64) -> Result<SpanStep, Failure> {
        let (adjusted, span) = if !self.view.periodic && self.prev_span >= 0 && u < self.span_end {
            (u, self.prev_span)
        } else {
            self.view.locate_span(u)?
        };
        let changed: bool = span != self.prev_span;
        if changed {
            self.prev_span = span;
            self.span_end = self.view.knot(span + 1)?;
        }
        Ok(SpanStep {
            adjusted,
            span,
            changed,
        })
    }
}
