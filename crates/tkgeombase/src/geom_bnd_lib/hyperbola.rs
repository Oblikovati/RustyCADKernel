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
// Rust translation of GeomBndLib_Hyperbola and GeomBndLib_Hyperbola2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Hyperbola` and `GeomBndLib_Hyperbola2d`: boxes of hyperbola branches.
use super::live;
use super::ranges::add_branch_range;
use super::space::{SampleBounds, SamplePoint};
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dHyperbola};
use rustycad_tkg3d::geom::{GeomCurve, GeomHyperbola};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Hyperbola2, Hyperbola3};

/// The finite-arc analysis `computeHyperbolaBox` (GeomBndLib_Hyperbola(2d).cxx) and
/// `ComputeBox` (BndLib.cxx), identical bodies: both ends, the vertex when the arc
/// crosses 0, and per coordinate the extremum `t = log(|B - A| / |A + B|) / 2` of
/// `A sinh(t) + B cosh(t)` (A = Rmin Yk, B = Rmaj Xk) when it lies in `[t1, t2]`.
pub struct HyperbolaArc<'a, P: SamplePoint> {
    /// Major radius.
    pub major: f64,
    /// Minor radius.
    pub minor: f64,
    /// X direction coordinates.
    pub x_dir: [f64; 3],
    /// Y direction coordinates.
    pub y_dir: [f64; 3],
    /// The point at a parameter (`ElCLib::Value`).
    pub eval: &'a dyn Fn(f64) -> P,
}

impl<P: SamplePoint> HyperbolaArc<'_, P> {
    /// Add the arc `[t1, t2]`; returns the native `iErr` (0 when an extremum was added,
    /// 1 otherwise); `arc.add_arc(&mut bounds, -1.0, 1.0)`.
    pub fn add_arc<B: SampleBounds<Point = P>>(&self, bounds: &mut B, t1: f64, t2: f64) -> i32 {
        bounds.add_sample((self.eval)(t1));
        bounds.add_sample((self.eval)(t2));
        if t1 * t2 < 0.0 {
            bounds.add_sample((self.eval)(0.0));
        }
        let eps: f64 = epsilon(1.0);
        let mut error: i32 = 1;
        for k in 0..P::DIM {
            let a: f64 = self.minor * self.y_dir[k];
            let b: f64 = self.major * self.x_dir[k];
            let sum: f64 = (a + b).abs();
            let difference: f64 = (b - a).abs();
            if sum < eps || difference < eps {
                continue;
            }
            let t3: f64 = 0.5 * (difference / sum).ln();
            if t3 < t1 || t3 > t2 {
                continue;
            }
            bounds.add_sample((self.eval)(t3));
            error = 0;
        }
        error
    }
}

/// `GeomBndLib_Hyperbola`: the box of a `Geom_Hyperbola` branch; `hypr_box` takes a
/// `gp_Hypr`.
pub struct Hyperbola {
    geom: Handle<GeomHyperbola>,
}

impl Hyperbola {
    /// The evaluator of `hyperbola`; `Hyperbola::new(handle)`.
    pub fn new(hyperbola: Handle<GeomHyperbola>) -> Self {
        Self { geom: hyperbola }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomHyperbola> {
        &self.geom
    }
    /// `Box(U1, U2, Tol)`; `evaluator.bounding_box_range(-1.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        Self::hypr_box(
            &live(&self.geom, "GeomBndLib_Hyperbola")?.hypr()?,
            u1,
            u2,
            tolerance,
        )
    }
    /// `Box(Tol)`: the whole branch, whose infinite parameters open every side;
    /// `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let curve = live(&self.geom, "GeomBndLib_Hyperbola")?;
        self.bounding_box_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(-1.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(gp_Hypr, U1, U2, Tol)`: an infinite end opens every min (U -> -inf) or max
    /// (U -> +inf) side; two infinite ends of the same sign raise Standard_Failure;
    /// `Hyperbola::hypr_box(&hyperbola, -1.0, 1.0, 0.0)?`.
    pub fn hypr_box(
        hyperbola: &Hyperbola3,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let value = |u: f64| elclib::value(u, hyperbola);
        let position = hyperbola.position();
        let arc = HyperbolaArc {
            major: hyperbola.major_radius(),
            minor: hyperbola.minor_radius(),
            x_dir: *position.x_direction().coordinates(),
            y_dir: *position.y_direction().coordinates(),
            eval: &value,
        };
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        let mut finite = |b: &mut BoundingBox3| {
            arc.add_arc(b, u1, u2);
        };
        let message = "GeomBndLib_Hyperbola::Box - bad parameter";
        add_branch_range(&mut bounds, (u1, u2), &value, &mut finite, message)?;
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
}

/// `GeomBndLib_Hyperbola2d`: the box of a `Geom2d_Hyperbola` branch; `hypr_box` takes
/// a `gp_Hypr2d`.
pub struct Hyperbola2d {
    geom: Handle<Geom2dHyperbola>,
}

impl Hyperbola2d {
    /// The evaluator of `hyperbola`; `Hyperbola2d::new(handle)`.
    pub fn new(hyperbola: Handle<Geom2dHyperbola>) -> Self {
        Self { geom: hyperbola }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<Geom2dHyperbola> {
        &self.geom
    }
    /// `Box(U1, U2, Tol)`; `evaluator.bounding_box_range(-1.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        Self::hypr_box(
            &live(&self.geom, "GeomBndLib_Hyperbola2d")?.hypr2d()?,
            u1,
            u2,
            tolerance,
        )
    }
    /// `Box(Tol)`: the whole branch; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        let curve = live(&self.geom, "GeomBndLib_Hyperbola2d")?;
        self.bounding_box_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: the analytic box;
    /// `evaluator.bounding_box_optimal_range(-1.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(gp_Hypr2d, U1, U2, Tol)`: as [`Hyperbola::hypr_box`] in 2D;
    /// `Hyperbola2d::hypr_box(&hyperbola, -1.0, 1.0, 0.0)?`.
    pub fn hypr_box(
        hyperbola: &Hyperbola2,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        let value = |u: f64| elclib::value(u, hyperbola);
        let frame = hyperbola.axis();
        let (x, y) = (frame.x_direction(), frame.y_direction());
        let arc = HyperbolaArc {
            major: hyperbola.major_radius(),
            minor: hyperbola.minor_radius(),
            x_dir: [x.x(), x.y(), 0.0],
            y_dir: [y.x(), y.y(), 0.0],
            eval: &value,
        };
        let mut bounds: BoundingBox2 = BoundingBox2::default();
        let mut finite = |b: &mut BoundingBox2| {
            arc.add_arc(b, u1, u2);
        };
        let message = "GeomBndLib_Hyperbola2d::Box - bad parameter";
        add_branch_range(&mut bounds, (u1, u2), &value, &mut finite, message)?;
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
}
