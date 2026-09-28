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
// Rust translation of IntAna_IntLinTorus.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_IntLinTorus: the intersection of a line and a torus.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Torus3, Transform3, Vector3};
use rustycad_tkmath::math::DirectPolynomialRoots;
use rustycad_tkmath::{elclib, elslib};

/// The square distance above which a root is not a point of the torus.
const BAD_ROOT_SQUARE_DISTANCE: f64 = 0.0000000001;

/// IntAna_IntLinTorus: the points of a line on a torus, with their parameter on
/// the line and their (FI, THETA) parameters on the torus.
///
/// `let inter = IntLinTorus::with(line, torus)?; let n = inter.nb_points()?;`
#[derive(Clone, Copy, Debug)]
pub struct IntLinTorus {
    done: bool,
    nbpt: i32,
    the_point: [Point3; 4],
    the_param: [f64; 4],
    the_fi: [f64; 4],
    the_theta: [f64; 4],
}

impl Default for IntLinTorus {
    /// IntAna_IntLinTorus(): not done, no point, null parameters.
    fn default() -> Self {
        Self {
            done: false,
            nbpt: 0,
            the_point: [Point3::default(); 4],
            the_param: [0.0; 4],
            the_fi: [0.0; 4],
            the_theta: [0.0; 4],
        }
    }
}

impl IntLinTorus {
    /// IntAna_IntLinTorus(): the empty constructor. `IntLinTorus::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// IntAna_IntLinTorus(gp_Lin, gp_Torus). `IntLinTorus::with(line, torus)?`.
    pub fn with(line: Line3, torus: Torus3) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform(line, torus)?;
        Ok(inter)
    }

    /// Perform(gp_Lin, gp_Torus): the roots of the quartic of the line, first
    /// re-located at its point nearest the torus center, in the torus frame. A root
    /// whose line point is farther than 1e-5 from its projection on the torus is
    /// dropped; when every root is dropped the intersection is not done.
    /// `inter.perform(line, torus)?`.
    pub fn perform(&mut self, line: Line3, torus: Torus3) -> Result<(), Failure> {
        let (shift, coefficients) = line_quartic(line, torus)?;
        let roots = DirectPolynomialRoots::quartic(
            coefficients[0],
            coefficients[1],
            coefficients[2],
            coefficients[3],
            coefficients[4],
        );
        if !roots.is_done() {
            self.nbpt = 0;
            self.done = false;
            return Ok(());
        }
        let count: i32 = roots.solutions()?;
        let mut valid: usize = 0;
        for index in 1..=count {
            let t: f64 = roots.value(index)? + shift;
            let on_line: Point3 = elclib::value(t, &line);
            let (u, v) = elslib::parameters(&torus, on_line);
            let on_torus: Point3 = elslib::value(u, v, &torus);
            if on_torus.square_distance(&on_line) > BAD_ROOT_SQUARE_DISTANCE {
                continue;
            }
            self.the_param[valid] = t;
            self.the_fi[valid] = u;
            self.the_theta[valid] = v;
            self.the_point[valid] = on_line;
            valid += 1;
        }
        // Not done only when there were roots and all of them were rejected.
        self.done = !(count > 0 && valid == 0);
        self.nbpt = valid as i32;
        Ok(())
    }

    /// IsDone(): true when the computation succeeded. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// NbPoints(): the number of points; StdFail_NotDone when not done.
    /// `inter.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.check_done("NbPoints")?;
        Ok(self.nbpt)
    }

    /// Value(Index): the point of range `index`; StdFail_NotDone when not done,
    /// Standard_OutOfRange outside `1..=NbPoints()`. `inter.value(1)?`.
    pub fn value(&self, index: i32) -> Result<Point3, Failure> {
        Ok(self.the_point[self.check_index("Value", index)?])
    }

    /// ParamOnLine(Index): the parameter on the line of the point of range
    /// `index`, with the errors of [`Self::value`]. `inter.param_on_line(1)?`.
    pub fn param_on_line(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.the_param[self.check_index("ParamOnLine", index)?])
    }

    /// ParamOnTorus(Index, FI, THETA): the parameters on the torus of the point of
    /// range `index`, with the errors of [`Self::value`].
    /// `let (fi, theta) = inter.param_on_torus(1)?;`
    pub fn param_on_torus(&self, index: i32) -> Result<(f64, f64), Failure> {
        let slot: usize = self.check_index("ParamOnTorus", index)?;
        Ok((self.the_fi[slot], self.the_theta[slot]))
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(Failure::with_type(
            "StdFail_NotDone",
            &format!(
                "IntAna_IntLinTorus::{member}() - the intersection is not done; expected IsDone()"
            ),
            "",
        ))
    }

    fn check_index(&self, member: &str, index: i32) -> Result<usize, Failure> {
        self.check_done(member)?;
        if index <= 0 || index > self.nbpt {
            return Err(Failure::out_of_range(&format!(
                "IntAna_IntLinTorus::{member}() - index {index} outside [1, {}]",
                self.nbpt
            )));
        }
        Ok((index - 1) as usize)
    }
}

/// The parameter shift that re-locates the line at its point nearest the torus
/// center, and the quartic `[a4, a3, a2, a1, a0]` in the parameter from there.
fn line_quartic(line: Line3, torus: Torus3) -> Result<(f64, [f64; 5]), Failure> {
    let location: Point3 = line.location();
    let mut direction: Direction3 = line.direction();
    let shift: f64 = Vector3::between(location, torus.location()).dot(Vector3::from(direction));
    let mut near = Point3::from(location.xyz() + shift * direction.xyz());
    let transform: Transform3 = Transform3::transformation_to(torus.position());
    near.transform(transform);
    direction.transform(transform);
    let [x1, y1, z1] = *direction.coordinates();
    let [x0, y0, z0] = *near.coordinates();
    let (big, small) = (torus.major_radius(), torus.minor_radius());
    let (big2, small2) = (big * big, small * small);
    let a: f64 = x1 * x1 + y1 * y1 + z1 * z1;
    let b: f64 = 2.0 * (x1 * x0 + y1 * y0 + z1 * z0);
    let c: f64 = x0 * x0 + y0 * y0 + z0 * z0 - (big2 + small2);
    Ok((
        shift,
        [
            a * a,
            2.0 * a * b,
            2.0 * a * c + 4.0 * big2 * z1 * z1 + b * b,
            2.0 * b * c + 8.0 * big2 * z1 * z0,
            c * c + 4.0 * big2 * (z0 * z0 - small2),
        ],
    ))
}
