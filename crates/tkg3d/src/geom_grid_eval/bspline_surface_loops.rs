// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of the anonymous namespace of ModelingData/TKG3d/GeomGridEval/GeomGridEval_BSplineSurface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The B-spline surface grid loops (`SurfaceData`, `computeSpanCoeffs`,
//! `evaluateGridCached`, `evaluateGridDirect`).
use super::spans::{FlatKnotsView, SpanStep, SpanWalker};
use super::surface_grid::grid_array;
use crate::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplslib::{Cache, SurfaceSpan};
use rustycad_tkmath::gp::Point3;

/// Runs whose U x V cell count reaches this go through the cache (`THE_CACHE_THRESHOLD`).
const CACHE_THRESHOLD: i32 = 2;

/// The surface arrays of `extractSurfaceData`, copied once per call.
pub(super) struct SurfaceData {
    pub(super) u_flat: Array1<'static, f64>,
    pub(super) v_flat: Array1<'static, f64>,
    pub(super) poles: Array2<'static, Point3>,
    pub(super) weights: Option<Array2<'static, f64>>,
    pub(super) u_degree: i32,
    pub(super) v_degree: i32,
    u_periodic: bool,
    v_periodic: bool,
    u_rational: bool,
    v_rational: bool,
}

impl SurfaceData {
    /// Reads the flat knots, poles, weights, degrees, periodicity and rationality.
    pub(super) fn read(surface: &GeomBSplineSurface) -> Self {
        Self {
            u_flat: surface.u_knot_sequence(),
            v_flat: surface.v_knot_sequence(),
            poles: surface.poles(),
            weights: surface.weights(),
            u_degree: surface.u_degree(),
            v_degree: surface.v_degree(),
            u_periodic: surface.is_u_periodic(),
            v_periodic: surface.is_v_periodic(),
            u_rational: surface.is_u_rational(),
            v_rational: surface.is_v_rational(),
        }
    }
    fn u_view(&self) -> FlatKnotsView<'_> {
        FlatKnotsView {
            flat_knots: &self.u_flat,
            degree: self.u_degree,
            periodic: self.u_periodic,
        }
    }
    fn v_view(&self) -> FlatKnotsView<'_> {
        FlatKnotsView {
            flat_knots: &self.v_flat,
            degree: self.v_degree,
            periodic: self.v_periodic,
        }
    }
    /// The span of the direct `BSplSLib::D0`..`DN` calls: flat knots, no
    /// multiplicities, the located span indices.
    pub(super) fn span(&self, uv: (f64, f64), spans: (i32, i32)) -> SurfaceSpan<'_, 'static> {
        SurfaceSpan {
            u: uv.0,
            v: uv.1,
            u_index: spans.0,
            v_index: spans.1,
            u_knots: &self.u_flat,
            v_knots: &self.v_flat,
            u_mults: None,
            v_mults: None,
            u_degree: self.u_degree,
            v_degree: self.v_degree,
            u_rat: self.u_rational,
            v_rat: self.v_rational,
            u_per: self.u_periodic,
            v_per: self.v_periodic,
        }
    }
    fn new_cache(&self) -> Result<Cache, Failure> {
        let (u, v) = (self.u_view(), self.v_view());
        let weights = self.weights.as_ref();
        Cache::new(
            u.degree,
            u.periodic,
            u.flat_knots,
            v.degree,
            v.periodic,
            v.flat_knots,
            weights,
        )
    }
}

/// One cell handed to the evaluation functors: adjusted parameters and span indices.
#[derive(Clone, Copy)]
pub(super) struct GridCell {
    pub(super) uv: (f64, f64),
    pub(super) spans: (i32, i32),
}

// The run of one direction (`aXSpanSize`, `aXSpanMid`, `aXSpanHalfLen`).
#[derive(Clone, Copy)]
struct DirectionRun {
    size: i32,
    mid: f64,
    half: f64,
}

impl DirectionRun {
    const START: Self = Self {
        size: 0,
        mid: 0.0,
        half: 1.0,
    };
    // `countSpanSize` then `computeSpanCoeffs` for a new span.
    fn enter(
        view: FlatKnotsView<'_>,
        values: &[f64],
        index: usize,
        span: i32,
    ) -> Result<Self, Failure> {
        let size: i32 = view.count_span_size(values, index, span)?;
        let start: f64 = view.knot(span)?;
        let half: f64 = 0.5 * (view.knot(span + 1)? - start);
        Ok(Self {
            size,
            mid: start + half,
            half,
        })
    }
    // The surface cache works on the [-1, 1] local parameter of the span.
    fn local(&self, adjusted: f64) -> f64 {
        (adjusted - self.mid) / self.half
    }
}

/// `evaluateGridCached`: every cell through the patch cache or the direct call; the
/// cache is rebuilt when a V span changes and the run covers enough cells.
pub(super) fn evaluate_cached<T: Default>(
    data: &SurfaceData,
    uv_params: (&Array1<'_, f64>, &Array1<'_, f64>),
    cache_eval: impl Fn(&Cache, f64, f64) -> Result<T, Failure>,
    direct_eval: impl Fn(&SurfaceData, GridCell) -> Result<T, Failure>,
) -> Result<Array2<'static, T>, Failure> {
    let (u_values, v_values) = (uv_params.0.as_slice(), uv_params.1.as_slice());
    let mut grid: Array2<'static, T> = grid_array(u_values.len(), v_values.len());
    let mut cache: Cache = data.new_cache()?;
    let mut u_walker = SpanWalker::new(data.u_view());
    let mut u_run = DirectionRun::START;
    for (ui, row) in grid.as_mut_slice().chunks_mut(v_values.len()).enumerate() {
        let u_step: SpanStep = u_walker.step(u_values[ui])?;
        if u_step.changed {
            u_run = DirectionRun::enter(data.u_view(), u_values, ui, u_step.span)?;
        }
        let row_state = RowState {
            data,
            u_step,
            u_run,
            v_values,
        };
        row_state.fill_cached(row, &mut cache, &cache_eval, &direct_eval)?;
    }
    Ok(grid)
}

// The U values that a row of the cached loop keeps fixed.
struct RowState<'a> {
    data: &'a SurfaceData,
    u_step: SpanStep,
    u_run: DirectionRun,
    v_values: &'a [f64],
}

impl RowState<'_> {
    fn fill_cached<T>(
        &self,
        row: &mut [T],
        cache: &mut Cache,
        cache_eval: &impl Fn(&Cache, f64, f64) -> Result<T, Failure>,
        direct_eval: &impl Fn(&SurfaceData, GridCell) -> Result<T, Failure>,
    ) -> Result<(), Failure> {
        let mut v_walker = SpanWalker::new(self.data.v_view());
        let (mut v_run, mut use_cache) = (DirectionRun::START, false);
        for (vi, slot) in row.iter_mut().enumerate() {
            let v_step: SpanStep = v_walker.step(self.v_values[vi])?;
            if v_step.changed {
                v_run = DirectionRun::enter(self.data.v_view(), self.v_values, vi, v_step.span)?;
                use_cache = self.u_run.size * v_run.size >= CACHE_THRESHOLD;
                if use_cache {
                    self.build(cache, v_step.adjusted)?;
                }
            }
            *slot = if use_cache {
                cache_eval(
                    cache,
                    self.u_run.local(self.u_step.adjusted),
                    v_run.local(v_step.adjusted),
                )?
            } else {
                direct_eval(self.data, self.cell(v_step))?
            };
        }
        Ok(())
    }
    fn build(&self, cache: &mut Cache, v_adjusted: f64) -> Result<(), Failure> {
        let d = self.data;
        let weights = d.weights.as_ref();
        cache.build_cache(
            self.u_step.adjusted,
            v_adjusted,
            &d.u_flat,
            &d.v_flat,
            &d.poles,
            weights,
        )
    }
    fn cell(&self, v_step: SpanStep) -> GridCell {
        GridCell {
            uv: (self.u_step.adjusted, v_step.adjusted),
            spans: (self.u_step.span, v_step.span),
        }
    }
}

/// `evaluateGridDirect`: every cell through the direct call, with the same span
/// location as the cached loop.
pub(super) fn evaluate_direct<T: Default>(
    data: &SurfaceData,
    uv_params: (&Array1<'_, f64>, &Array1<'_, f64>),
    eval: impl Fn(&SurfaceData, GridCell) -> Result<T, Failure>,
) -> Result<Array2<'static, T>, Failure> {
    let (u_values, v_values) = (uv_params.0.as_slice(), uv_params.1.as_slice());
    let mut grid: Array2<'static, T> = grid_array(u_values.len(), v_values.len());
    let mut u_walker = SpanWalker::new(data.u_view());
    for (row, &u) in grid.as_mut_slice().chunks_mut(v_values.len()).zip(u_values) {
        let u_step: SpanStep = u_walker.step(u)?;
        let mut v_walker = SpanWalker::new(data.v_view());
        for (slot, &v) in row.iter_mut().zip(v_values) {
            let v_step: SpanStep = v_walker.step(v)?;
            let cell = GridCell {
                uv: (u_step.adjusted, v_step.adjusted),
                spans: (u_step.span, v_step.span),
            };
            *slot = eval(data, cell)?;
        }
    }
    Ok(grid)
}
