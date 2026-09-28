// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of IntAna2d_AnaIntersection_3.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_AnaIntersection::Perform(gp_Lin2d, gp_Circ2d).
use super::AnaIntersection;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Line2, Point2};

impl AnaIntersection {
    /// Perform(gp_Lin2d, gp_Circ2d): no point, a tangency point or two points,
    /// decided by the signed distance of the center to the line against
    /// Epsilon(Radius); the parameters come from ElCLib::Parameter on both curves.
    /// `inter.perform_line_circle(line, circle)?`.
    pub fn perform_line_circle(&mut self, line: Line2, circle: Circle2) -> Result<(), Failure> {
        self.done = false;
        self.iden = false;
        self.para = false;
        let (a, b, c0) = line.coefficients();
        let center: Point2 = circle.location();
        let d: f64 = a * center.x() + b * center.y() + c0;
        let radius: f64 = circle.radius();
        if d.abs() - radius > epsilon(radius) {
            self.empt = true;
            self.nbp = 0;
        } else if (d.abs() - radius).abs() <= epsilon(radius) {
            self.empt = false;
            self.nbp = 1;
            let tangency = Point2::new(center.x() - d * a, center.y() - d * b);
            let (u, angle) = parameters(tangency, line, circle)?;
            self.lpnt[0].set_value(tangency.x(), tangency.y(), u, angle);
        } else {
            self.empt = false;
            self.nbp = 2;
            let h: f64 = (radius * radius - d * d).sqrt();
            let first = Point2::new(center.x() - d * a - h * b, center.y() - d * b + h * a);
            let second = Point2::new(center.x() - d * a + h * b, center.y() - d * b - h * a);
            // Every parameter is computed before the points are stored, as natively.
            let u1: f64 = line_parameter(line, first)?;
            let u2: f64 = line_parameter(line, second)?;
            let angle1: f64 = elclib::parameter(&circle, first)?;
            let angle2: f64 = elclib::parameter(&circle, second)?;
            self.lpnt[0].set_value(first.x(), first.y(), u1, angle1);
            self.lpnt[1].set_value(second.x(), second.y(), u2, angle2);
        }
        self.done = true;
        Ok(())
    }
}

/// ElCLib::Parameter(L, P) then ElCLib::Parameter(C, P).
fn parameters(point: Point2, line: Line2, circle: Circle2) -> Result<(f64, f64), Failure> {
    let u: f64 = line_parameter(line, point)?;
    Ok((u, elclib::parameter(&circle, point)?))
}

/// ElCLib::Parameter(L, P).
fn line_parameter(line: Line2, point: Point2) -> Result<f64, Failure> {
    elclib::parameter(&line, point)
}
