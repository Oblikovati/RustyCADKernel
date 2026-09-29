// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_SweepApproximation.lxx/.cxx (SurfShape ... Dump)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The results of `Approx_SweepApproximation`.
use super::{SweepApproximation, func_ref, null_table};
use crate::native_failure::{array_failure, failure};
use crate::native_text::stream_real;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use std::fmt;

/// `SurfShape(UDegree, VDegree, NbUPoles, NbVPoles, NbUKnots, NbVKnots)`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SweepSurfaceShape {
    /// `UDegree`: the section degree.
    pub u_degree: i32,
    /// `VDegree`: the degree along the path.
    pub v_degree: i32,
    /// `NbUPoles`.
    pub nb_u_poles: i32,
    /// `NbVPoles`.
    pub nb_v_poles: i32,
    /// `NbUKnots`.
    pub nb_u_knots: i32,
    /// `NbVKnots`.
    pub nb_v_knots: i32,
}

/// The copies `Surface(TPoles, TWeights, TUKnots, TVKnots, TUMults, TVMults)` assigns.
#[derive(Clone, Default)]
pub struct SweepSurface {
    /// `TPoles`.
    pub poles: Array2<'static, Point3>,
    /// `TWeights`.
    pub weights: Array2<'static, f64>,
    /// `TUKnots`.
    pub u_knots: Array1<'static, f64>,
    /// `TVKnots`.
    pub v_knots: Array1<'static, f64>,
    /// `TUMults`.
    pub u_mults: Array1<'static, i32>,
    /// `TVMults`.
    pub v_mults: Array1<'static, i32>,
}

/// The copies `Curve2d(Index, TPoles, TKnots, TMults)` assigns.
#[derive(Clone, Default)]
pub struct SweepCurve2d {
    /// `TPoles`.
    pub poles: Array1<'static, Point2>,
    /// `TKnots`.
    pub knots: Array1<'static, f64>,
    /// `TMults`.
    pub mults: Array1<'static, i32>,
}

fn table<'t, T>(value: &'t Option<T>, what: &str) -> Result<&'t T, Failure> {
    value.as_ref().ok_or_else(|| null_table(what))
}

fn value_at<T: Copy>(array: &Array1<'static, T>, index: i32) -> Result<T, Failure> {
    array
        .value(index)
        .copied()
        .map_err(|e| array_failure(e, "Approx_SweepApproximation"))
}

impl SweepApproximation {
    /// `SurfShape(...)`: StdFail_NotDone without a result; `approx.surf_shape()?`.
    pub fn surf_shape(&self) -> Result<SweepSurfaceShape, Failure> {
        self.check_done()?;
        let poles = table(&self.tab_poles, "tabPoles")?;
        Ok(SweepSurfaceShape {
            u_degree: self.udeg,
            v_degree: self.vdeg,
            nb_u_poles: poles.nb_rows() as i32,
            nb_v_poles: poles.nb_columns() as i32,
            nb_u_knots: table(&self.tab_u_knots, "tabUKnots")?.length(),
            nb_v_knots: table(&self.tab_v_knots, "tabVKnots")?.length(),
        })
    }

    /// `Surface(TPoles, TWeights, TUKnots, TVKnots, TUMults, TVMults)`: copies of the
    /// surface tables; `approx.surface()?`.
    pub fn surface(&self) -> Result<SweepSurface, Failure> {
        self.check_done()?;
        Ok(SweepSurface {
            poles: self.surf_poles()?.clone(),
            weights: self.surf_weights()?.clone(),
            u_knots: self.surf_u_knots()?.clone(),
            v_knots: self.surf_v_knots()?.clone(),
            u_mults: self.surf_u_mults()?.clone(),
            v_mults: self.surf_v_mults()?.clone(),
        })
    }

    /// `UDegree()`; `approx.u_degree()?`.
    pub fn u_degree(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.udeg)
    }

    /// `VDegree()`; `approx.v_degree()?`.
    pub fn v_degree(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.vdeg)
    }

    /// `SurfPoles()`: the poles, one row per section pole; `approx.surf_poles()?`.
    pub fn surf_poles(&self) -> Result<&Array2<'static, Point3>, Failure> {
        self.check_done()?;
        table(&self.tab_poles, "tabPoles")
    }

    /// `SurfWeights()`; `approx.surf_weights()?`.
    pub fn surf_weights(&self) -> Result<&Array2<'static, f64>, Failure> {
        self.check_done()?;
        table(&self.tab_weights, "tabWeights")
    }

    /// `SurfUKnots()`: the section knots; `approx.surf_u_knots()?`.
    pub fn surf_u_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.check_done()?;
        table(&self.tab_u_knots, "tabUKnots")
    }

    /// `SurfVKnots()`: the knots along the path; `approx.surf_v_knots()?`.
    pub fn surf_v_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.check_done()?;
        table(&self.tab_v_knots, "tabVKnots")
    }

    /// `SurfUMults()`; `approx.surf_u_mults()?`.
    pub fn surf_u_mults(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.check_done()?;
        table(&self.tab_u_mults, "tabUMults")
    }

    /// `SurfVMults()`; `approx.surf_v_mults()?`.
    pub fn surf_v_mults(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.check_done()?;
        table(&self.tab_v_mults, "tabVMults")
    }

    /// `MaxErrorOnSurf()`: the largest 3D error (for a rational section
    /// `(Size E1d + E3d) / Wmin` per pole); `approx.max_error_on_surf()?`.
    pub fn max_error_on_surf(&self) -> Result<f64, Failure> {
        self.check_done()?;
        let mut max_error: f64 = 0.0;
        for err in self.pole_errors(true)? {
            if err > max_error {
                max_error = err;
            }
        }
        Ok(max_error)
    }

    /// `AverageErrorOnSurf()`: the mean of the average errors of the poles;
    /// `approx.average_error_on_surf()?`.
    pub fn average_error_on_surf(&self) -> Result<f64, Failure> {
        self.check_done()?;
        let mut sum: f64 = 0.0;
        for err in self.pole_errors(false)? {
            sum += err;
        }
        Ok(sum / f64::from(self.cache.num_3d))
    }

    // The maximal (or average) error of each section pole.
    fn pole_errors(&self, maximal: bool) -> Result<Vec<f64>, Failure> {
        let func = func_ref(&self.cache.func)?;
        let (e3d, e1d) = if maximal {
            (&self.m_error_3d, &self.m_error_1d)
        } else {
            (&self.a_error_3d, &self.a_error_1d)
        };
        let e3d = table(e3d, "Error3d")?;
        let num_3d: i32 = self.cache.num_3d;
        if !func.is_rational()? {
            return (1..=num_3d).map(|ii| value_at(e3d, ii)).collect();
        }
        let mut wmin: Array1<'static, f64> = Array1::with_bounds(1, self.cache.num_1d)
            .map_err(|e| array_failure(e, "Approx_SweepApproximation"))?;
        func.get_minimal_weight(&mut wmin)?;
        let size: f64 = func.maximal_section()?;
        let e1d = table(e1d, "Error1d")?;
        (1..=num_3d)
            .map(|ii| Ok((size * value_at(e1d, ii)? + value_at(e3d, ii)?) / value_at(&wmin, ii)?))
            .collect()
    }

    /// `NbCurves2d()`; `approx.nb_curves_2d()?`.
    pub fn nb_curves_2d(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.cache.num_2d)
    }

    /// `Curves2dShape(Degree, NbPoles, NbKnots)`: Standard_DomainError without 2D
    /// curves; `approx.curves_2d_shape()?`.
    pub fn curves_2d_shape(&self) -> Result<(i32, i32, i32), Failure> {
        self.check_curves_2d()?;
        let knots = table(&self.tab_2d_knots, "tab2dKnots")?;
        Ok((self.deg2d, self.seq_poles_2d[0].length(), knots.length()))
    }

    /// `Curve2d(Index, TPoles, TKnots, TMults)`: copies of 2D curve `index`;
    /// `approx.curve_2d(1)?`.
    pub fn curve_2d(&self, index: i32) -> Result<SweepCurve2d, Failure> {
        Ok(SweepCurve2d {
            poles: self.curve_2d_poles(index)?.clone(),
            knots: self.curves_2d_knots()?.clone(),
            mults: self.curves_2d_mults()?.clone(),
        })
    }

    /// `Curves2dDegree()`; `approx.curves_2d_degree()?`.
    pub fn curves_2d_degree(&self) -> Result<i32, Failure> {
        self.check_curves_2d()?;
        Ok(self.deg2d)
    }

    /// `Curve2dPoles(Index)`: Standard_OutOfRange outside the curves;
    /// `approx.curve_2d_poles(1)?`.
    pub fn curve_2d_poles(&self, index: i32) -> Result<&Array1<'static, Point2>, Failure> {
        self.check_curves_2d()?;
        usize::try_from(index - 1)
            .ok()
            .and_then(|slot| self.seq_poles_2d.get(slot))
            .ok_or_else(|| {
                Failure::out_of_range(&format!("NCollection_Sequence::Value: index {index}"))
            })
    }

    /// `Curves2dKnots()`; `approx.curves_2d_knots()?`.
    pub fn curves_2d_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.check_curves_2d()?;
        table(&self.tab_2d_knots, "tab2dKnots")
    }

    /// `Curves2dMults()`; `approx.curves_2d_mults()?`.
    pub fn curves_2d_mults(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.check_curves_2d()?;
        table(&self.tab_2d_mults, "tab2dMults")
    }

    /// `Max2dError(Index)`; `approx.max_2d_error(1)?`.
    pub fn max_2d_error(&self, index: i32) -> Result<f64, Failure> {
        self.check_done()?;
        value_at(table(&self.tab_2d_error, "tab2dError")?, index)
    }

    /// `Average2dError(Index)`; `approx.average_2d_error(1)?`.
    pub fn average_2d_error(&self, index: i32) -> Result<f64, Failure> {
        self.check_done()?;
        value_at(table(&self.ave_2d_error, "Ave2dError")?, index)
    }

    /// `TolCurveOnSurf(Index)`: the 3D error of 2D curve `index` on the surface
    /// (its 2D error scaled by the bound tolerance); `approx.tol_curve_on_surf(1)?`.
    pub fn tol_curve_on_surf(&self, index: i32) -> Result<f64, Failure> {
        self.check_done()?;
        value_at(table(&self.con_surf_err, "COnSurfErr")?, index)
    }

    /// `Dump(o)`: the errors and the spans (`%g`). As natively, the line end after
    /// the 2D errors and the " Not Done " text go to the standard output, not to
    /// `stream`; `approx.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut dyn fmt::Write) -> Result<(), Failure> {
        let io = |_| {
            failure(
                "Standard_Failure",
                "Approx_SweepApproximation::Dump: stream error",
            )
        };
        writeln!(stream, "Dump of SweepApproximation").map_err(io)?;
        if !self.done {
            println!(" Not Done ");
            return Ok(());
        }
        writeln!(
            stream,
            "Error 3d = {}",
            stream_real(self.max_error_on_surf()?)
        )
        .map_err(io)?;
        let num_2d: i32 = self.cache.num_2d;
        if num_2d > 0 {
            write!(stream, "Error 2d = ").map_err(io)?;
            for ii in 1..=num_2d {
                write!(stream, "{}", stream_real(self.max_2d_error(ii)?)).map_err(io)?;
                if ii < num_2d {
                    writeln!(stream, " , ").map_err(io)?;
                }
            }
            println!();
        }
        let spans: i32 = table(&self.tab_v_knots, "tabVKnots")?.length() - 1;
        writeln!(stream, "{spans} Segment(s) of degree {}", self.vdeg).map_err(io)
    }
}
