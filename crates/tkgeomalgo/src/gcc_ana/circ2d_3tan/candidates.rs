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
// Rust translation of the radius filters repeated in GccAna_Circ2d3Tan_1.cxx to
// GccAna_Circ2d3Tan_7.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The radius filters that the bisector-based constructors of GccAna_Circ2d3Tan
//! repeat for each candidate centre: the radii a qualified circle allows, and their
//! agreement with a second qualified circle. The `Radius(1, 2)` array lives across
//! the candidates as natively.
use crate::gcc_ent::QualifiedCirc;

/// The `Radius` array of a constructor and the `ok` flag of the current centre.
pub(super) struct RadiusCandidates {
    pub(super) radius: [f64; 2],
    pub(super) ok: bool,
}

impl RadiusCandidates {
    pub(super) fn new() -> Self {
        Self {
            radius: [0.0; 2],
            ok: false,
        }
    }

    /// Radius(i), 1-based.
    pub(super) fn at(&self, i: i32) -> f64 {
        self.radius[(i - 1) as usize]
    }

    fn set(&mut self, i: i32, value: f64) {
        self.radius[(i - 1) as usize] = value;
    }

    /// The radii circle 1 allows for a centre at `dist1` from its centre; returns
    /// nbsol1 and sets `ok`. An enclosing circle gives |R1 - dist1| in some native
    /// files and R1 + dist1 in others (`enclosing_sum`).
    pub(super) fn first_circle(
        &mut self,
        q1: QualifiedCirc,
        dist1: f64,
        tolerance: f64,
        enclosing_sum: bool,
    ) -> i32 {
        let r1: f64 = q1.qualified().radius();
        self.ok = false;
        if q1.is_enclosed() {
            if dist1 - r1 < tolerance {
                self.set(1, (r1 - dist1).abs());
                self.ok = true;
                return 1;
            }
        } else if q1.is_outside() {
            if r1 - dist1 < tolerance {
                self.set(1, (r1 - dist1).abs());
                self.ok = true;
                return 1;
            }
        } else if q1.is_enclosing() {
            self.ok = true;
            self.set(
                1,
                if enclosing_sum {
                    r1 + dist1
                } else {
                    (r1 - dist1).abs()
                },
            );
            return 1;
        } else if q1.is_unqualified() {
            self.ok = true;
            self.set(1, (r1 - dist1).abs());
            self.set(2, r1 + dist1);
            return 2;
        }
        0
    }

    /// Keep the radii that circle 2 also allows for a centre at `dist2`; returns
    /// nbsol2 (the radii kept are moved to the front).
    pub(super) fn second_circle(
        &mut self,
        q2: QualifiedCirc,
        dist2: f64,
        nbsol1: i32,
        (tolerance, tol): (f64, f64),
    ) -> i32 {
        let r2: f64 = q2.qualified().radius();
        let inner: f64 = (r2 - dist2).abs();
        let mut nbsol2: i32 = 0;
        if !self.ok {
            return 0;
        }
        if q2.is_enclosed() || q2.is_outside() {
            let allowed: bool = if q2.is_enclosed() {
                dist2 - r2 < tolerance
            } else {
                r2 - dist2 < tolerance
            };
            if !allowed {
                return 0;
            }
            for ii in 1..=nbsol1 {
                if (self.at(ii) - inner).abs() < tol {
                    self.set(1, inner);
                    nbsol2 = 1;
                }
            }
            return nbsol2;
        }
        if q2.is_enclosing() {
            for ii in 1..=nbsol1 {
                if (self.at(ii) - r2 - dist2).abs() < tol {
                    self.set(1, r2 + dist2);
                    nbsol2 = 1;
                }
            }
            return nbsol2;
        }
        self.second_unqualified(r2, dist2, nbsol1, tol)
    }

    fn second_unqualified(&mut self, r2: f64, dist2: f64, nbsol1: i32, tol: f64) -> i32 {
        let mut rradius: f64 = 0.0;
        let mut nbsol2: i32 = 0;
        for ii in 1..=nbsol1 {
            if (self.at(ii) - (r2 - dist2).abs()).abs() < tol {
                rradius = (r2 - dist2).abs();
                nbsol2 += 1;
            } else if (self.at(ii) - r2 - dist2).abs() < tol {
                rradius = r2 + dist2;
                nbsol2 += 1;
            }
        }
        if nbsol2 == 1 {
            self.set(1, rradius);
        } else if nbsol2 == 2 {
            self.set(1, (r2 - dist2).abs());
            self.set(2, r2 + dist2);
        }
        nbsol2
    }

    /// Keep the radius that a line at `dist2` also allows (the centre on the side
    /// `side` of it); the matching radius moves to Radius(1).
    pub(super) fn line_radius(
        &mut self,
        qualified: crate::gcc_ent::QualifiedLin,
        side: f64,
        dist2: f64,
        nbsol1: i32,
        tol: f64,
    ) {
        if !self.ok {
            return;
        }
        let allowed: bool = if qualified.is_enclosed() {
            side <= 0.0
        } else if qualified.is_outside() {
            side >= 0.0
        } else {
            qualified.is_unqualified()
        };
        if !allowed {
            return;
        }
        for ii in 1..=nbsol1 {
            if (dist2 - self.at(ii)).abs() < tol {
                self.set(1, self.at(ii));
            }
        }
    }
}
