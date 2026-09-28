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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Circle.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Circle`: `P(u) = C + R * (cos(u) * X + sin(u) * Y)` on a grid.
use super::conic::{
    ConicTerms, conic_grid, conic_grid_d1, conic_grid_d2, conic_grid_d3, conic_grid_dn,
    conic_grid_members,
};
use super::frame::GridFrame3;
use super::{GeomGridEvalCurveD1, GeomGridEvalCurveD2, GeomGridEvalCurveD3};
use crate::geom::GeomCircle;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::{cyclic_coefficients, empty};
use rustycad_tkmath::gp::{Circle3, Point3, Vector3};

/// Batch evaluator of a `Geom_Circle` (`GeomGridEval_Circle`).
///
/// The circle is read once per call (`Circ()`); a null circle or empty parameters
/// give an empty result; `GeomGridEvalCircle::new(circle).evaluate_grid(&params)?`.
pub struct GeomGridEvalCircle {
    geom: Handle<GeomCircle>,
}

// The frame and radius of the circle.
struct CircleCoords {
    frame: GridFrame3,
    radius: f64,
}

impl CircleCoords {
    // `R * (a * X + b * Y)`, as `aRadius * (a * aXX + b * aYX)`.
    fn vector(&self, a: f64, b: f64) -> Vector3 {
        let v: Vector3 = self.frame.combined(a, b);
        Vector3::new(
            self.radius * v.x(),
            self.radius * v.y(),
            self.radius * v.z(),
        )
    }
}

impl ConicTerms for CircleCoords {
    type Local = (f64, f64);
    fn local(&self, u: f64) -> (f64, f64) {
        (u.cos(), u.sin())
    }
    fn point(&self, (c, s): (f64, f64)) -> Point3 {
        self.frame.translated(self.vector(c, s))
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
    // Cyclic with period 4: `(theN - 1) % 4` selects the coefficients.
    fn dn(&self, u: f64, n: i32) -> Vector3 {
        let (a, b) = cyclic_coefficients(n, u.cos(), u.sin());
        self.vector(a, b)
    }
}

impl GeomGridEvalCircle {
    /// Evaluator of `circle` (`GeomGridEval_Circle(theCircle)`); a null handle is
    /// accepted; `GeomGridEvalCircle::new(circle)`.
    pub fn new(circle: Handle<GeomCircle>) -> Self {
        Self { geom: circle }
    }
    fn terms(&self, params: &Array1<'_, f64>) -> Result<Option<CircleCoords>, Failure> {
        let Some(circle) = self.geom.get() else {
            return Ok(None);
        };
        if params.is_empty() {
            return Ok(None);
        }
        let circ: Circle3 = circle.circ()?;
        let axis = circ.position();
        Ok(Some(CircleCoords {
            frame: GridFrame3::new(circ.location(), axis.x_direction(), axis.y_direction()),
            radius: circ.radius(),
        }))
    }
    conic_grid_members!(GeomCircle);
}
