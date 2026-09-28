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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Ellipse.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Ellipse`: `P(u) = C + MajR * cos(u) * X + MinR * sin(u) * Y` on a grid.
use super::conic::{
    ConicTerms, conic_grid, conic_grid_d1, conic_grid_d2, conic_grid_d3, conic_grid_dn,
    conic_grid_members,
};
use super::frame::{GridFrame3, RadiiFrame3};
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use crate::geom::GeomEllipse;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::{cyclic_coefficients, empty};
use rustycad_tkmath::gp::{Ellipse3, Point3, Vector3};

/// Batch evaluator of a `Geom_Ellipse` (`GeomGridEval_Ellipse`).
///
/// The ellipse is read once per call (`Elips()`, X and Y from `XAxis()`/`YAxis()`); a
/// null ellipse or empty parameters give an empty result;
/// `GeomGridEvalEllipse::new(ellipse).evaluate_grid(&params)?`.
pub struct GeomGridEvalEllipse {
    geom: Handle<GeomEllipse>,
}

impl ConicTerms for RadiiFrame3 {
    type Local = (f64, f64);
    fn local(&self, u: f64) -> (f64, f64) {
        (u.cos(), u.sin())
    }
    fn point(&self, (c, s): (f64, f64)) -> Point3 {
        RadiiFrame3::point(self, c, s)
    }
    fn d1(&self, (c, s): (f64, f64)) -> Vector3 {
        self.vector(-s, c)
    }
    fn d2(&self, (c, s): (f64, f64)) -> Vector3 {
        self.vector(-c, -s)
    }
    fn d3(&self, (c, s): (f64, f64)) -> Vector3 {
        self.vector(s, -c)
    }
    // `aMajR * aCoeffMajR * aXX + aMinR * aCoeffMinR * aYX`, cyclic with period 4.
    fn dn(&self, u: f64, n: i32) -> Vector3 {
        let (a, b) = cyclic_coefficients(n, u.cos(), u.sin());
        self.vector(a, b)
    }
}

impl GeomGridEvalEllipse {
    /// Evaluator of `ellipse` (`GeomGridEval_Ellipse(theEllipse)`); a null handle is
    /// accepted; `GeomGridEvalEllipse::new(ellipse)`.
    pub fn new(ellipse: Handle<GeomEllipse>) -> Self {
        Self { geom: ellipse }
    }
    fn terms(&self, params: &Array1<'_, f64>) -> Result<Option<RadiiFrame3>, Failure> {
        let Some(ellipse) = self.geom.get() else {
            return Ok(None);
        };
        if params.is_empty() {
            return Ok(None);
        }
        let elips: Ellipse3 = ellipse.elips()?;
        let (x_dir, y_dir) = (elips.x_axis().direction(), elips.y_axis().direction());
        Ok(Some(RadiiFrame3 {
            frame: GridFrame3::new(elips.location(), x_dir, y_dir),
            major: elips.major_radius(),
            minor: elips.minor_radius(),
        }))
    }
    conic_grid_members!(GeomEllipse);
}
