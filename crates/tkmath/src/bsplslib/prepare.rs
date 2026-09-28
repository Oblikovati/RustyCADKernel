// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (validateBSplineDegree, BSplSLib_DataContainer, PrepareEval) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Extraction of the local knots and (homogeneous) poles of a surface patch.
//!
//! The direction of lower degree comes first ("u-first" when UDegree <= VDegree):
//! the patch is stored as rows along that direction, each row holding the poles
//! of the other direction, so that the first scheme runs on whole rows.
use crate::bsplclib::native::epsilon;
use crate::bsplclib::{MAX_DEGREE, build_knots, locate_parameter_for_eval, pole_index};
use crate::gp::Point3;
use crate::plib::access::grid_value;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Parameters, knots and flags shared by the surface evaluation functions.
#[derive(Clone, Copy)]
pub struct SurfaceSpan<'r, 'a> {
    /// U parameter.
    pub u: f64,
    /// V parameter.
    pub v: f64,
    /// U knot index hint, used as is when inside the knot array bounds.
    pub u_index: i32,
    /// V knot index hint, used as is when inside the knot array bounds.
    pub v_index: i32,
    /// U knots (flat knots when `u_mults` is None).
    pub u_knots: &'r Array1<'a, f64>,
    /// V knots (flat knots when `v_mults` is None).
    pub v_knots: &'r Array1<'a, f64>,
    /// U multiplicities.
    pub u_mults: Option<&'r Array1<'a, i32>>,
    /// V multiplicities.
    pub v_mults: Option<&'r Array1<'a, i32>>,
    /// U degree.
    pub u_degree: i32,
    /// V degree.
    pub v_degree: i32,
    /// Rationality in U.
    pub u_rat: bool,
    /// Rationality in V.
    pub v_rat: bool,
    /// Periodicity in U.
    pub u_per: bool,
    /// Periodicity in V.
    pub v_per: bool,
}

/// Raises the native Standard_OutOfRange for degrees above MaxDegree;
/// `validate_degrees(3, 3)?`.
pub(crate) fn validate_degrees(u_degree: i32, v_degree: i32) -> Result<(), Failure> {
    if u_degree > MAX_DEGREE || v_degree > MAX_DEGREE {
        return Err(Failure::out_of_range(
            "BSplSLib: bspline degree is greater than maximum supported",
        ));
    }
    Ok(())
}

// BSplSLib_DataContainer and the outputs of PrepareEval.
pub(crate) struct Patch {
    pub(crate) u_first: bool,
    pub(crate) u1: f64,
    pub(crate) u2: f64,
    pub(crate) d1: i32,
    pub(crate) d2: i32,
    pub(crate) rational: bool,
    pub(crate) knots1: Vec<f64>,
    pub(crate) knots2: Vec<f64>,
    pub(crate) poles: Vec<f64>,
}

impl Patch {
    /// Values per homogeneous or Cartesian pole.
    pub(crate) fn dim(&self) -> usize {
        if self.rational { 4 } else { 3 }
    }
}

// A span of one direction: located index and parameter.
struct Located {
    index: i32,
    parameter: f64,
}

fn locate(
    degree: i32,
    knots: &Array1<'_, f64>,
    mults: Option<&Array1<'_, i32>>,
    parameter: f64,
    periodic: bool,
    hint: i32,
) -> Result<Located, Failure> {
    let mut located = Located {
        index: hint,
        parameter,
    };
    // Only a hint outside the knot array is located again.
    if hint < knots.lower() || hint > knots.upper() {
        locate_parameter_for_eval(
            degree,
            knots,
            mults,
            parameter,
            periodic,
            &mut located.index,
            &mut located.parameter,
        )?;
    }
    Ok(located)
}

fn span_pole_index(
    degree: i32,
    index: i32,
    periodic: bool,
    knots: &Array1<'_, f64>,
    mults: Option<&Array1<'_, i32>>,
) -> Result<i32, Failure> {
    match mults {
        None => Ok(index - (knots.lower() + degree)),
        Some(mults) => pole_index(degree, index, periodic, mults),
    }
}

/// The native PrepareEval: locates both spans, builds the local knots and copies
/// the (homogeneous when the patch weights truly differ) poles; `u_first` tells
/// which direction comes first.
pub(crate) fn prepare_eval(
    span: &SurfaceSpan<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<Patch, Failure> {
    let u_first: bool = span.u_degree <= span.v_degree;
    let lu = locate(
        span.u_degree,
        span.u_knots,
        span.u_mults,
        span.u,
        span.u_per,
        span.u_index,
    )?;
    let lv = locate(
        span.v_degree,
        span.v_knots,
        span.v_mults,
        span.v,
        span.v_per,
        span.v_index,
    )?;
    let max: usize = MAX_DEGREE as usize;
    let mut knots_u: Vec<f64> = vec![0.0; 2 * max];
    let mut knots_v: Vec<f64> = vec![0.0; 2 * max];
    build_knots(
        span.u_degree,
        lu.index,
        span.u_per,
        span.u_knots,
        span.u_mults,
        &mut knots_u,
    )?;
    build_knots(
        span.v_degree,
        lv.index,
        span.v_per,
        span.v_knots,
        span.v_mults,
        &mut knots_v,
    )?;
    let uindex: i32 = span_pole_index(
        span.u_degree,
        lu.index,
        span.u_per,
        span.u_knots,
        span.u_mults,
    )?;
    let vindex: i32 = span_pole_index(
        span.v_degree,
        lv.index,
        span.v_per,
        span.v_knots,
        span.v_mults,
    )?;
    let grid = PatchGrid {
        poles,
        weights,
        uindex,
        vindex,
        u_degree: span.u_degree,
        v_degree: span.v_degree,
    };
    let rational: bool = (span.u_rat || span.v_rat) && grid.locally_rational()?;
    let (knots1, knots2) = if u_first {
        (knots_u, knots_v)
    } else {
        (knots_v, knots_u)
    };
    let (u1, u2) = if u_first {
        (lu.parameter, lv.parameter)
    } else {
        (lv.parameter, lu.parameter)
    };
    let (d1, d2) = if u_first {
        (span.u_degree, span.v_degree)
    } else {
        (span.v_degree, span.u_degree)
    };
    let poles_buffer: Vec<f64> = if u_first {
        grid.u_first_poles(rational)?
    } else {
        grid.v_first_poles(rational)?
    };
    Ok(Patch {
        u_first,
        u1,
        u2,
        d1,
        d2,
        rational,
        knots1,
        knots2,
        poles: poles_buffer,
    })
}

struct PatchGrid<'r, 'a, 'w> {
    poles: &'r Array2<'a, Point3>,
    weights: Option<&'r Array2<'w, f64>>,
    uindex: i32,
    vindex: i32,
    u_degree: i32,
    v_degree: i32,
}

fn wrap_low(index: i32, lower: i32, upper: i32) -> i32 {
    if index < lower { upper } else { index }
}

fn wrap_high(index: i32, lower: i32, upper: i32) -> i32 {
    if index > upper { lower } else { index }
}

impl PatchGrid<'_, '_, '_> {
    fn weights(&self) -> Result<&Array2<'_, f64>, Failure> {
        self.weights.ok_or_else(|| {
            Failure::out_of_range("BSplSLib: rational evaluation needs Weights; received None")
        })
    }

    fn rows(&self) -> (i32, i32) {
        (self.poles.lower_row(), self.poles.upper_row())
    }

    fn cols(&self) -> (i32, i32) {
        (self.poles.lower_col(), self.poles.upper_col())
    }

    // True when a weight of the patch differs from the first by more than Epsilon.
    fn locally_rational(&self) -> Result<bool, Failure> {
        let weights = self.weights()?;
        let ((pl_row, pu_row), (pl_col, pu_col)) = (self.rows(), self.cols());
        let mut ip: i32 = wrap_low(pl_row + self.uindex, pl_row, pu_row);
        let first_col: i32 = wrap_low(pl_col + self.vindex, pl_col, pu_col);
        let w: f64 = grid_value(weights, ip, first_col)?;
        let eps: f64 = epsilon(w);
        for _ in 0..=self.u_degree {
            let mut jp: i32 = first_col;
            for _ in 0..=self.v_degree {
                let mut dw: f64 = grid_value(weights, ip, jp)? - w;
                if dw < 0.0 {
                    dw = -dw;
                }
                if dw > eps {
                    return Ok(true);
                }
                jp = wrap_high(jp + 1, pl_col, pu_col);
            }
            ip = wrap_high(ip + 1, pl_row, pu_row);
        }
        Ok(false)
    }

    fn push_pole(
        &self,
        buffer: &mut Vec<f64>,
        ip: i32,
        jp: i32,
        rational: bool,
    ) -> Result<(), Failure> {
        let point: Point3 = grid_value(self.poles, ip, jp)?;
        if !rational {
            buffer.extend_from_slice(&[point.x(), point.y(), point.z()]);
            return Ok(());
        }
        let w: f64 = grid_value(self.weights()?, ip, jp)?;
        buffer.extend_from_slice(&[point.x() * w, point.y() * w, point.z() * w, w]);
        Ok(())
    }

    // Rows along U, each holding VDegree+1 poles.
    fn u_first_poles(&self, rational: bool) -> Result<Vec<f64>, Failure> {
        let ((pl_row, pu_row), (pl_col, pu_col)) = (self.rows(), self.cols());
        let mut buffer: Vec<f64> = Vec::new();
        let mut ip: i32 = wrap_low(pl_row + self.uindex, pl_row, pu_row);
        for _ in 0..=self.u_degree {
            let mut jp: i32 = wrap_low(pl_col + self.vindex, pl_col, pu_col);
            for _ in 0..=self.v_degree {
                self.push_pole(&mut buffer, ip, jp, rational)?;
                jp = wrap_high(jp + 1, pl_col, pu_col);
            }
            ip = wrap_high(ip + 1, pl_row, pu_row);
        }
        Ok(padded(buffer))
    }

    // Rows along V, each holding UDegree+1 poles.
    fn v_first_poles(&self, rational: bool) -> Result<Vec<f64>, Failure> {
        let ((pl_row, pu_row), (pl_col, pu_col)) = (self.rows(), self.cols());
        let mut buffer: Vec<f64> = Vec::new();
        let mut jp: i32 = wrap_low(pl_col + self.vindex, pl_col, pu_col);
        for _ in 0..=self.v_degree {
            let mut ip: i32 = wrap_low(pl_row + self.uindex, pl_row, pu_row);
            // Only this copy also wraps a start index past the last row, as natively.
            if !rational {
                ip = wrap_high(ip, pl_row, pu_row);
            }
            for _ in 0..=self.u_degree {
                self.push_pole(&mut buffer, ip, jp, rational)?;
                ip = wrap_high(ip + 1, pl_row, pu_row);
            }
            jp = wrap_high(jp + 1, pl_col, pu_col);
        }
        Ok(padded(buffer))
    }
}

// The data container's pole buffer: 4 * 26 * 26 values.
fn padded(mut buffer: Vec<f64>) -> Vec<f64> {
    let size: usize = 4 * (MAX_DEGREE as usize + 1) * (MAX_DEGREE as usize + 1);
    if buffer.len() < size {
        buffer.resize(size, 0.0);
    }
    buffer
}
