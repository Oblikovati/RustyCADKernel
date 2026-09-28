// Copyright (c) 2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_CacheParams.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The span bookkeeping shared by the curve and surface caches.
use super::locate::locate_parameter_for_eval;
use super::native::{epsilon, std_min};
use crate::plib::access::array_value;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Span of a cached B-spline or Bezier parameterization (`BSplCLib_CacheParams`).
#[derive(Debug)]
pub struct CacheParams {
    /// Degree of the B-spline (or Bezier).
    pub degree: i32,
    /// Periodicity of the B-spline.
    pub is_periodic: bool,
    /// First valid parameter.
    pub first_parameter: f64,
    /// Last valid parameter.
    pub last_parameter: f64,
    /// Minimal span index.
    pub span_index_min: i32,
    /// Maximal span index.
    pub span_index_max: i32,
    /// Parameter of the first point of the span.
    pub span_start: f64,
    /// Length of the span.
    pub span_length: f64,
    /// Index of the span.
    pub span_index: i32,
}

impl CacheParams {
    /// Prepares the span bookkeeping of flat knots; `CacheParams::new(3, false, &flat)?`.
    pub fn new(degree: i32, periodic: bool, flat_knots: &Array1<'_, f64>) -> Result<Self, Failure> {
        Ok(Self {
            degree,
            is_periodic: periodic,
            first_parameter: array_value(flat_knots, flat_knots.lower() + degree)?,
            last_parameter: array_value(flat_knots, flat_knots.upper() - degree)?,
            span_index_min: flat_knots.lower() + degree,
            span_index_max: flat_knots.upper() - degree - 1,
            span_start: 0.0,
            span_length: 0.0,
            span_index: 0,
        })
    }

    /// Brings a parameter of a periodic B-spline into `[First, Last]` by whole periods
    /// (`PeriodicNormalization`); `params.periodic_normalization(7.0)`.
    pub fn periodic_normalization(&self, parameter: f64) -> f64 {
        if self.is_periodic {
            let period: f64 = self.last_parameter - self.first_parameter;
            if parameter < self.first_parameter {
                let scale: f64 = ((self.first_parameter - parameter) / period).trunc();
                return parameter + period * (scale + 1.0);
            }
            if parameter > self.last_parameter {
                let scale: f64 = ((parameter - self.last_parameter) / period).trunc();
                return parameter - period * (scale + 1.0);
            }
        }
        parameter
    }

    /// Tests whether `parameter` lies in the cached span, treating a parameter within
    /// Epsilon of the next knot as belonging to the next span (`IsCacheValid`);
    /// `params.is_cache_valid(0.5)`.
    pub fn is_cache_valid(&self, parameter: f64) -> bool {
        let new_param: f64 = self.periodic_normalization(parameter);
        let delta: f64 = new_param - self.span_start;
        if (delta < 0.0 && self.span_index != self.span_index_min)
            || (delta >= self.span_length && self.span_index != self.span_index_max)
        {
            return false;
        }
        if self.span_index == self.span_index_max {
            return true;
        }
        // From BSplCLib::LocateParameter: hitting the next knot within double precision.
        let eps: f64 = epsilon(std_min(self.last_parameter.abs(), new_param.abs()));
        (delta - self.span_length).abs() > eps
    }

    /// Locates the span of `parameter`, which may be wrapped, and records it
    /// (`LocateParameter`); `params.locate_parameter(&mut u, &flat)?`.
    pub fn locate_parameter(
        &mut self,
        parameter: &mut f64,
        flat_knots: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.span_index = 0;
        let u: f64 = *parameter;
        locate_parameter_for_eval(
            self.degree,
            flat_knots,
            None,
            u,
            self.is_periodic,
            &mut self.span_index,
            parameter,
        )?;
        self.span_start = array_value(flat_knots, self.span_index)?;
        self.span_length = array_value(flat_knots, self.span_index + 1)? - self.span_start;
        Ok(())
    }
}
