// Created on: 1996-06-06
// Created by: Philippe MANGIN
// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_CompBezierSurfacesToBSplineSurface.hxx/.cxx/.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_CompBezierSurfacesToBSplineSurface`: the B-spline surface data of a
//! grid of adjacent polynomial Bezier patches.
use super::comp_bezier_surfaces_perform::{JoinedPoles, perform};
use crate::array_access::at;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1, HArray2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierSurface, GeomCurve, GeomSurface};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The knots, multiplicities, degrees and poles of the B-spline surface joining a
/// grid of Bezier patches (row index along U, column index along V). The patches must
/// be polynomial and adjacent; the constructors elevate their degrees in place.
///
/// `let conv = CompBezierSurfacesToBSplineSurface::new(&patches)?;
/// let poles = conv.poles();`
#[derive(Clone)]
pub struct CompBezierSurfacesToBSplineSurface {
    my_u_degree: i32,
    my_v_degree: i32,
    my_v_mults: Handle<HArray1<i32>>,
    my_u_mults: Handle<HArray1<i32>>,
    my_u_knots: Handle<HArray1<f64>>,
    my_v_knots: Handle<HArray1<f64>>,
    my_poles: Handle<HArray2<Point3>>,
    my_done: bool,
}

impl CompBezierSurfacesToBSplineSurface {
    /// The C0 B-spline with knots 0, 1, 2... in both directions
    /// (`GeomConvert_CompBezierSurfacesToBSplineSurface(Beziers)`); Standard_NotImplemented
    /// for a rational patch; `CompBezierSurfacesToBSplineSurface::new(&patches)?`.
    pub fn new(beziers: &Array2<'_, Handle<GeomBezierSurface>>) -> Result<Self, Failure> {
        let u_knots: Array1<'static, f64> = index_knots(beziers.col_length() + 1)?;
        let v_knots: Array1<'static, f64> = index_knots(beziers.row_length() + 1)?;
        Self::performed(beziers, u_knots, v_knots)
    }

    /// The B-spline whose knots follow the pseudo-lengths of the patches (an iso of
    /// the middle row or column, at 0.3), with the inner multiplicities reduced as far
    /// as `tolerance` (halved at each step) allows, down to 0 when `remove_knots`,
    /// else 1 (`GeomConvert_CompBezierSurfacesToBSplineSurface(Beziers, Tolerance,
    /// RemoveKnots = true)`).
    ///
    /// Natively the V reduction loop tests the last U multiplicity (`multU`) instead
    /// of `multV`, and the port does the same: when that value exceeds the lower
    /// limit, V knots are removed until one removal fails;
    /// `CompBezierSurfacesToBSplineSurface::with_tolerance(&patches, 1e-7, true)?`.
    pub fn with_tolerance(
        beziers: &Array2<'_, Handle<GeomBezierSurface>>,
        tolerance: f64,
        remove_knots: bool,
    ) -> Result<Self, Failure> {
        let u_knots: Array1<'static, f64> = length_knots(beziers, true)?;
        let v_knots: Array1<'static, f64> = length_knots(beziers, false)?;
        let mut result: Self = Self::performed(beziers, u_knots, v_knots)?;
        let surface: GeomBSplineSurface = result.surface()?;
        let minus: i32 = if remove_knots { 0 } else { 1 };
        let mut mult_u: i32 = 0;
        for ii in (2..result.nb_u_knots()).rev() {
            let (mut ok, mut tol) = (true, tolerance / 2.0);
            mult_u = mult_of(&result.my_u_mults, ii)? - 1;
            while ok && mult_u > minus {
                ok = surface.remove_u_knot(ii, mult_u, tol)?;
                mult_u -= 1;
                tol /= 2.0;
            }
        }
        for ii in (2..result.nb_v_knots()).rev() {
            let (mut ok, mut tol) = (true, tolerance / 2.0);
            let mut mult_v: i32 = mult_of(&result.my_v_mults, ii)? - 1;
            // The native condition tests multU, not multV.
            while ok && mult_u > minus {
                ok = surface.remove_v_knot(ii, mult_v, tol)?;
                mult_v -= 1;
                tol /= 2.0;
            }
        }
        result.store_surface(&surface, true);
        Ok(result)
    }

    /// The B-spline with the given knots and continuities: each inner knot is
    /// reduced to multiplicity `Degree - k` for a continuity Ck; `is_done` is false
    /// when a removal fails within `tolerance`. Standard_ConstructionError for a
    /// continuity above C3 or when `Degree - k <= 0` with inner knots
    /// (`GeomConvert_CompBezierSurfacesToBSplineSurface(Beziers, UKnots, VKnots,
    /// UContinuity = GeomAbs_C0, VContinuity = GeomAbs_C0, Tolerance = 1e-4)`);
    /// `CompBezierSurfacesToBSplineSurface::with_knots(&patches, &uk, &vk, (Shape::C1, Shape::C1), 1e-4)?`.
    pub fn with_knots(
        beziers: &Array2<'_, Handle<GeomBezierSurface>>,
        u_knots: &Array1<'_, f64>,
        v_knots: &Array1<'_, f64>,
        (u_continuity, v_continuity): (Shape, Shape),
        tolerance: f64,
    ) -> Result<Self, Failure> {
        // myUKnots->ChangeArray1() = UKnots: the table takes the given knots and bounds.
        let mut my_u_knots: Array1<'static, f64> = one_based(beziers.col_length() + 1)?;
        my_u_knots.assign(u_knots);
        let mut my_v_knots: Array1<'static, f64> = one_based(beziers.row_length() + 1)?;
        my_v_knots.assign(v_knots);
        let mut result: Self = Self::performed(beziers, my_u_knots, my_v_knots)?;
        let decu: i32 = continuity_rank(u_continuity, "UContinuity")?;
        let decv: i32 = continuity_rank(v_continuity, "VContinuity")?;
        if decu > 0 || decv > 0 {
            result.reduce(decu, decv, tolerance)?;
        }
        Ok(result)
    }

    // Computes the poles and multiplicities for the given knot tables.
    fn performed(
        beziers: &Array2<'_, Handle<GeomBezierSurface>>,
        u_knots: Array1<'static, f64>,
        v_knots: Array1<'static, f64>,
    ) -> Result<Self, Failure> {
        let joined: JoinedPoles = perform(beziers, u_knots.length(), v_knots.length())?;
        Ok(Self {
            my_u_degree: joined.u_degree,
            my_v_degree: joined.v_degree,
            my_v_mults: Handle::new(HArray1::from_owned(joined.v_mults)),
            my_u_mults: Handle::new(HArray1::from_owned(joined.u_mults)),
            my_u_knots: Handle::new(HArray1::from_owned(u_knots)),
            my_v_knots: Handle::new(HArray1::from_owned(v_knots)),
            my_poles: Handle::new(HArray2::from_owned(joined.poles)),
            my_done: true,
        })
    }

    // The continuity reduction of the knots constructor.
    fn reduce(&mut self, decu: i32, decv: i32, tolerance: f64) -> Result<(), Failure> {
        let mult_u: i32 = self.my_u_degree - decu;
        if mult_u <= 0 && self.nb_u_knots() > 2 {
            return Err(failure(
                "Standard_ConstructionError",
                "GeomConvert_CompBezierSurfacesToBSpl:: UContinuity or Udeg error",
            ));
        }
        let mult_v: i32 = self.my_v_degree - decv;
        if mult_v <= 0 && self.nb_v_knots() > 2 {
            return Err(failure(
                "Standard_ConstructionError",
                "GeomConvert_CompBezierSurfacesToBSpl:: VContinuity or Vdeg error",
            ));
        }
        let surface: GeomBSplineSurface = self.surface()?;
        if decu > 0 {
            for ii in 2..self.nb_u_knots() {
                if !surface.remove_u_knot(ii, mult_u, tolerance)? {
                    self.my_done = false;
                }
            }
        }
        if decv > 0 {
            for ii in 2..self.nb_v_knots() {
                if !surface.remove_v_knot(ii, mult_v, tolerance)? {
                    self.my_done = false;
                }
            }
        }
        self.store_surface(&surface, false);
        Ok(())
    }

    // The B-spline of the current poles, knots and multiplicities.
    fn surface(&self) -> Result<GeomBSplineSurface, Failure> {
        GeomBSplineSurface::new(
            &self.my_poles.array2(),
            (&self.my_u_knots.array1(), &self.my_u_mults.array1()),
            (&self.my_v_knots.array1(), &self.my_v_mults.array1()),
            (self.my_u_degree, self.my_v_degree),
            (false, false),
        )
    }

    // "The new fields have been computed": the poles and multiplicities of the
    // reduced surface, and its knots too after the tolerance constructor.
    fn store_surface(&mut self, surface: &GeomBSplineSurface, with_knots: bool) {
        self.my_poles = Handle::new(HArray2::from_owned(surface.poles()));
        self.my_u_mults = Handle::new(HArray1::from_owned(surface.u_multiplicities()));
        self.my_v_mults = Handle::new(HArray1::from_owned(surface.v_multiplicities()));
        if with_knots {
            self.my_u_knots = Handle::new(HArray1::from_owned(surface.u_knots()));
            self.my_v_knots = Handle::new(HArray1::from_owned(surface.v_knots()));
        }
    }

    /// The number of U knots (`NbUKnots()`); `conv.nb_u_knots()`.
    pub fn nb_u_knots(&self) -> i32 {
        self.my_u_knots.array1().length()
    }

    /// The number of U poles, the row count of the pole table (`NbUPoles()`);
    /// `conv.nb_u_poles()`.
    pub fn nb_u_poles(&self) -> i32 {
        self.my_poles.array2().col_length()
    }

    /// The number of V knots (`NbVKnots()`); `conv.nb_v_knots()`.
    pub fn nb_v_knots(&self) -> i32 {
        self.my_v_knots.array1().length()
    }

    /// The number of V poles, the column count of the pole table (`NbVPoles()`);
    /// `conv.nb_v_poles()`.
    pub fn nb_v_poles(&self) -> i32 {
        self.my_poles.array2().row_length()
    }

    /// The shared pole table (`Poles()`); `conv.poles()`.
    pub fn poles(&self) -> Handle<HArray2<Point3>> {
        self.my_poles.clone()
    }

    /// The shared U knot table (`UKnots()`); `conv.u_knots()`.
    pub fn u_knots(&self) -> Handle<HArray1<f64>> {
        self.my_u_knots.clone()
    }

    /// The U degree (`UDegree()`); `conv.u_degree()`.
    pub fn u_degree(&self) -> i32 {
        self.my_u_degree
    }

    /// The shared V knot table (`VKnots()`); `conv.v_knots()`.
    pub fn v_knots(&self) -> Handle<HArray1<f64>> {
        self.my_v_knots.clone()
    }

    /// The V degree (`VDegree()`); `conv.v_degree()`.
    pub fn v_degree(&self) -> i32 {
        self.my_v_degree
    }

    /// The shared U multiplicity table (`UMultiplicities()`); `conv.u_multiplicities()`.
    pub fn u_multiplicities(&self) -> Handle<HArray1<i32>> {
        self.my_u_mults.clone()
    }

    /// The shared V multiplicity table (`VMultiplicities()`); `conv.v_multiplicities()`.
    pub fn v_multiplicities(&self) -> Handle<HArray1<i32>> {
        self.my_v_mults.clone()
    }

    /// Whether the continuity reduction held within the tolerance (`IsDone()`);
    /// `conv.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }
}

// Knots 0, 1, 2, ...
fn index_knots(count: i32) -> Result<Array1<'static, f64>, Failure> {
    let mut knots: Array1<'static, f64> = one_based(count)?;
    for ii in 0..count {
        knots[ii + 1] = f64::from(ii);
    }
    Ok(knots)
}

// GeomAbs_C0 .. C3 as 0 .. 3; Standard_ConstructionError otherwise.
fn continuity_rank(shape: Shape, which: &str) -> Result<i32, Failure> {
    match shape {
        Shape::C0 => Ok(0),
        Shape::C1 => Ok(1),
        Shape::C2 => Ok(2),
        Shape::C3 => Ok(3),
        _ => Err(failure(
            "Standard_ConstructionError",
            &format!("GeomConvert_CompBezierSurfacesToBSpl:: {which} error"),
        )),
    }
}

fn mult_of(mults: &Handle<HArray1<i32>>, index: i32) -> Result<i32, Failure> {
    mults
        .array1()
        .value(index)
        .copied()
        .map_err(|e| crate::native_failure::array_failure(e, "Mults"))
}

// The knots of one direction from the pseudo-lengths of the isos at 0.3 across the
// middle row (U) or column (V): each new knot extends the previous span by the
// ratio of the speeds at the junction, the pseudo-length replacing a speed that
// differs from it by more than a factor 1000.
fn length_knots(
    beziers: &Array2<'_, Handle<GeomBezierSurface>>,
    in_u: bool,
) -> Result<Array1<'static, f64>, Failure> {
    let (count, across): (i32, i32) = if in_u {
        (beziers.col_length() + 1, (beziers.row_length() + 1) / 2)
    } else {
        (beziers.row_length() + 1, (beziers.col_length() + 1) / 2)
    };
    let iso = |index: i32| -> Result<Handle<dyn GeomCurve>, Failure> {
        let (row, col) = if in_u {
            (index, across)
        } else {
            (across, index)
        };
        let patch: Handle<GeomBezierSurface> = at(beziers, row, col, "Beziers")?;
        let patch: &GeomBezierSurface = patch
            .get()
            .ok_or_else(|| failure("Standard_NullObject", "null patch"))?;
        if in_u {
            patch.v_iso(0.3)
        } else {
            patch.u_iso(0.3)
        }
    };
    let mut knots: Array1<'static, f64> = one_based(count)?;
    knots[1] = 0.0;
    let first = IsoSpeeds::of(&iso(1)?, true)?;
    knots[2] = first.length;
    let mut v1: f64 = clamped_speed(first.end_speed, first.length);
    for ii in 2..count {
        let second = IsoSpeeds::of(&iso(ii)?, false)?;
        let v2: f64 = clamped_speed(second.start_speed, second.length);
        let v3: f64 = clamped_speed(second.end_speed, second.length);
        let mut ratio: f64 = 1.0;
        if v1 > precision::confusion() && v2 > precision::confusion() {
            ratio = v2 / v1;
        }
        if ratio < precision::confusion() || ratio > 1.0 / precision::confusion() {
            ratio = 1.0;
        }
        // Deduce a new knot.
        let val: f64 = knots[ii];
        knots[ii + 1] = val + ratio * (val - knots[ii - 1]);
        v1 = v3;
    }
    Ok(knots)
}

// If the parameterization is too strange we keep the pseudo-length.
fn clamped_speed(speed: f64, length: f64) -> f64 {
    if speed > 1000.0 * length || speed < length * 1.0e-3 {
        length
    } else {
        speed
    }
}

// The pseudo-length P(0) P(0.5) P(1) of an iso and its speeds at both ends.
struct IsoSpeeds {
    length: f64,
    start_speed: f64,
    end_speed: f64,
}

impl IsoSpeeds {
    // The first iso of a direction reads its start point with D0, the next ones with
    // D1 (whose speed they use), as natively.
    fn of(curve: &Handle<dyn GeomCurve>, first: bool) -> Result<Self, Failure> {
        let curve: &dyn GeomCurve = curve
            .get()
            .ok_or_else(|| failure("Standard_NullObject", "null iso"))?;
        let (p1, start): (Point3, Vector3) = if first {
            (curve.d0(0.0)?, Vector3::default())
        } else {
            curve.d1(0.0)?
        };
        let p2: Point3 = curve.d0(0.5)?;
        let (p3, end): (Point3, Vector3) = curve.d1(1.0)?;
        Ok(Self {
            length: p1.distance(&p2) + p2.distance(&p3),
            start_speed: start.magnitude(),
            end_speed: end.magnitude(),
        })
    }
}
