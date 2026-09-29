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
// Rust translation of IntCurve_IntConicConic_Tool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The helpers of IntCurve_IntConicConic_Tool.hxx: the transition of a line and a
//! conic, the circle normalisation, and the `Interval` and `PeriodicInterval`
//! classes (global, unprefixed classes natively).
use crate::int_imp_par_gen::{set_crossing_transitions, set_touch_transitions};
use crate::int_res2d::{Domain, Position, Transition};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Vector2};
use std::f64::consts::PI;

/// `PIpPI`: the period of a circle, `M_PI + M_PI`.
pub const PI_P_PI: f64 = PI + PI;

// `TOLERANCE_ANGULAIRE` of IntCurve_IntConicConic_Tool.cxx.
const TOLERANCE_ANGULAIRE: f64 = 0.00000001;

/// `Determine_Transition_LC(Pos1, Tan1, Norm1, Trans1, Pos2, Tan2, Norm2, Trans2,
/// Tol)`: IN/OUT from the sign of the cross product of the tangents; for parallel
/// tangents (within 1e-8 of the product of their magnitudes) a TOUCH decided by the
/// normals projected on the unit normal of `tan1` (normalized in place;
/// gp_VectorWithNullMagnitude when it is null). The tolerance is unused;
/// `determine_transition_lc((pos1, &mut tan1, norm1, &mut t1), (pos2, &mut tan2, norm2, &mut t2), 1e-7)?`.
pub fn determine_transition_lc(
    (pos1, tan1, norm1, t1): (Position, &mut Vector2, Vector2, &mut Transition),
    (pos2, tan2, norm2, t2): (Position, &mut Vector2, Vector2, &mut Transition),
    _tol: f64,
) -> Result<(), Failure> {
    let sgn: f64 = tan1.crossed(*tan2);
    let norm: f64 = tan1.magnitude() * tan2.magnitude();
    let touch: bool = sgn.abs() <= TOLERANCE_ANGULAIRE * norm;
    if !touch {
        set_crossing_transitions((pos1, t1), (pos2, t2), sgn);
        return Ok(());
    }
    let opos: bool = tan1.dot(*tan2) < 0.0;
    // Modified by Sergey KHROMOV - Thu Nov  2 17:57:15 2000: the tangent is unit.
    tan1.normalize()?;
    let normal: Vector2 = Vector2::new(-tan1.y(), tan1.x());
    let (val1, val2) = (normal.dot(norm1), normal.dot(norm2));
    set_touch_transitions((pos1, t1), (pos2, t2), (val1, val2), opos, gp::resolution());
    Ok(())
}

/// `NormalizeOnCircleDomain(Param, Domain)`: adds, then subtracts, whole periods
/// until the parameter is not below the first bound, then not above the last one;
/// Standard_DomainError on an unbounded domain.
/// `normalize_on_circle_domain(-1.0, &domain)?`.
pub fn normalize_on_circle_domain(param: f64, domain: &Domain) -> Result<f64, Failure> {
    let mut param: f64 = param;
    while param < domain.first_parameter()? {
        param += PI_P_PI;
    }
    while param > domain.last_parameter()? {
        param -= PI_P_PI;
    }
    Ok(param)
}

/// `Interval`: a parameter interval on a line, possibly unbounded on either side,
/// or null.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve::Interval;
/// let mut half = Interval::with_bounds(0.0, true, 0.0, false);
/// let clipped = half.intersection_with_bounded(&Interval::new(-1.0, 2.0));
/// assert_eq!((clipped.binf, clipped.bsup), (0.0, 2.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    /// `Binf`: the lower bound.
    pub binf: f64,
    /// `Bsup`: the upper bound.
    pub bsup: f64,
    /// `HasFirstBound`.
    pub has_first_bound: bool,
    /// `HasLastBound`.
    pub has_last_bound: bool,
    /// `IsNull`: the interval is empty.
    pub is_null: bool,
}

impl Default for Interval {
    /// `Interval()`: the null interval. `Interval::default().is_null`.
    fn default() -> Self {
        Self {
            binf: 0.0,
            bsup: 0.0,
            has_first_bound: false,
            has_last_bound: false,
            is_null: true,
        }
    }
}

impl Interval {
    /// `Interval(a, b)`: the bounded interval between `a` and `b` in either order.
    /// `Interval::new(2.0, 1.0).binf == 1.0`.
    pub fn new(a: f64, b: f64) -> Self {
        let (binf, bsup) = if a < b { (a, b) } else { (b, a) };
        Self::with_bounds(binf, true, bsup, true)
    }
    /// `Interval(Domain)`: the domain widened by its tolerances.
    /// `Interval::from_domain(&domain)?`.
    pub fn from_domain(domain: &Domain) -> Result<Self, Failure> {
        let mut interval = Self::with_bounds(0.0, false, 0.0, false);
        if domain.has_first_point() {
            interval.has_first_bound = true;
            interval.binf = domain.first_parameter()? - domain.first_tolerance()?;
        }
        if domain.has_last_point() {
            interval.has_last_bound = true;
            interval.bsup = domain.last_parameter()? + domain.last_tolerance()?;
        }
        Ok(interval)
    }
    /// `Interval(a, hf, b, hl)`: the bounds as given, with their existence flags.
    /// `Interval::with_bounds(0.0, true, 0.0, false)`.
    pub fn with_bounds(a: f64, hf: bool, b: f64, hl: bool) -> Self {
        Self {
            binf: a,
            bsup: b,
            has_first_bound: hf,
            has_last_bound: hl,
            is_null: false,
        }
    }
    /// `Length()`: `|Bsup - Binf|`, -1 for the null interval. `interval.length()`.
    pub fn length(&mut self) -> f64 {
        if self.is_null {
            -1.0
        } else {
            (self.bsup - self.binf).abs()
        }
    }
    /// `IntersectionWithBounded(Inter)`: the common part with a bounded interval
    /// (null when disjoint; `inter` itself when this one is unbounded).
    /// `interval.intersection_with_bounded(&other)`.
    pub fn intersection_with_bounded(&mut self, inter: &Interval) -> Interval {
        if self.is_null || inter.is_null {
            return Interval::default();
        }
        if !(self.has_first_bound || self.has_last_bound) {
            return Interval::new(inter.binf, inter.bsup);
        }
        let a: f64 = if self.has_first_bound {
            if inter.bsup < self.binf {
                return Interval::default();
            }
            if inter.binf < self.binf {
                self.binf
            } else {
                inter.binf
            }
        } else {
            inter.binf
        };
        let b: f64 = if self.has_last_bound {
            if inter.binf > self.bsup {
                return Interval::default();
            }
            if inter.bsup > self.bsup {
                self.bsup
            } else {
                inter.bsup
            }
        } else {
            inter.bsup
        };
        Interval::new(a, b)
    }
}

/// `PeriodicInterval`: an interval of a circle's parameter, normalized so that
/// `Binf` is in `[0, 2PI]` and `Bsup` in `[Binf, Binf + 2PI)` unless it spans a
/// whole period, or null.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve::PeriodicInterval;
/// let interval = PeriodicInterval::new(-1.0, 1.0);
/// assert!((interval.binf - (2.0 * std::f64::consts::PI - 1.0)).abs() < 1e-15);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PeriodicInterval {
    /// `Binf`: the lower bound.
    pub binf: f64,
    /// `Bsup`: the upper bound.
    pub bsup: f64,
    /// `isnull`: the interval is empty.
    pub isnull: bool,
}

impl Default for PeriodicInterval {
    /// `PeriodicInterval()`: the null interval. `PeriodicInterval::default().is_null()`.
    fn default() -> Self {
        Self {
            binf: 0.0,
            bsup: 0.0,
            isnull: true,
        }
    }
}

impl PeriodicInterval {
    /// `PeriodicInterval(a, b)`: normalized when shorter than a period.
    /// `PeriodicInterval::new(0.0, 1.0)`.
    pub fn new(a: f64, b: f64) -> Self {
        let mut interval: Self = Self::default();
        interval.set_values(a, b);
        interval
    }
    /// `PeriodicInterval(Domain)`: the domain bounds, -1 and 20 for missing ones,
    /// not normalized. `PeriodicInterval::from_domain(&domain)?`.
    pub fn from_domain(domain: &Domain) -> Result<Self, Failure> {
        let binf: f64 = if domain.has_first_point() {
            domain.first_parameter()?
        } else {
            -1.0
        };
        let bsup: f64 = if domain.has_last_point() {
            domain.last_parameter()?
        } else {
            20.0
        };
        Ok(Self {
            binf,
            bsup,
            isnull: false,
        })
    }
    /// `SetNull()`. `interval.set_null()`.
    pub fn set_null(&mut self) {
        self.isnull = true;
    }
    /// `IsNull()`. `interval.is_null()`.
    pub fn is_null(&mut self) -> bool {
        self.isnull
    }
    /// `Complement()`: the rest of the period, `[Bsup, Binf + 2PI]` moved below
    /// 2PI. `interval.complement()`.
    pub fn complement(&mut self) {
        if !self.isnull {
            let t: f64 = self.binf;
            self.binf = self.bsup;
            self.bsup = t + PI_P_PI;
            if self.binf > PI_P_PI {
                self.binf -= PI_P_PI;
                self.bsup -= PI_P_PI;
            }
        }
    }
    /// `Length()`: `|Bsup - Binf|`, -100 for the null interval. `interval.length()`.
    pub fn length(&mut self) -> f64 {
        if self.isnull {
            -100.0
        } else {
            (self.bsup - self.binf).abs()
        }
    }
    /// `SetValues(a, b)`: normalized when shorter than a period.
    /// `interval.set_values(0.0, 1.0)`.
    pub fn set_values(&mut self, a: f64, b: f64) {
        self.isnull = false;
        self.binf = a;
        self.bsup = b;
        if (b - a) < PI_P_PI {
            self.normalize();
        }
    }
    /// `Normalize()`: `Binf` into `[0, 2PI]`, then `Bsup` into `[Binf, Binf + 2PI)`.
    /// `interval.normalize()`.
    pub fn normalize(&mut self) {
        if self.isnull {
            return;
        }
        while self.binf > PI_P_PI {
            self.binf -= PI_P_PI;
        }
        while self.binf < 0.0 {
            self.binf += PI_P_PI;
        }
        while self.bsup < self.binf {
            self.bsup += PI_P_PI;
        }
        while self.bsup >= (self.binf + PI_P_PI) {
            self.bsup -= PI_P_PI;
        }
    }
}

impl PeriodicInterval {
    /// `FirstIntersection(PInter)`: the common part with `pinter`, which is first
    /// shifted by whole periods next to this interval (the shift is kept, as through
    /// the native reference); an interval of a whole period gives the other one.
    /// `domain.first_intersection(&mut interval)`.
    pub fn first_intersection(&mut self, pinter: &mut PeriodicInterval) -> PeriodicInterval {
        if pinter.isnull || self.isnull {
            return PeriodicInterval::default();
        }
        if self.length() >= PI_P_PI {
            return PeriodicInterval::new(pinter.binf, pinter.bsup);
        }
        if pinter.length() >= PI_P_PI {
            return PeriodicInterval::new(self.binf, self.bsup);
        }
        if pinter.bsup <= self.binf {
            while pinter.binf <= self.binf && pinter.bsup <= self.binf {
                pinter.binf += PI_P_PI;
                pinter.bsup += PI_P_PI;
            }
        }
        if pinter.binf >= self.bsup {
            while pinter.binf >= self.bsup && pinter.bsup >= self.bsup {
                pinter.binf -= PI_P_PI;
                pinter.bsup -= PI_P_PI;
            }
        }
        if (pinter.bsup < self.binf) || (pinter.binf > self.bsup) {
            return PeriodicInterval::default();
        }
        let a: f64 = if pinter.binf > self.binf {
            pinter.binf
        } else {
            self.binf
        };
        let b: f64 = if pinter.bsup < self.bsup {
            pinter.bsup
        } else {
            self.bsup
        };
        PeriodicInterval::new(a, b)
    }
    /// `SecondIntersection(PInter)`: the common part with `pinter` shifted by one
    /// period up (down when that starts above this interval); null when either
    /// spans a whole period. `domain.second_intersection(&mut interval)`.
    pub fn second_intersection(&mut self, pinter: &mut PeriodicInterval) -> PeriodicInterval {
        if pinter.isnull || self.isnull || self.length() >= PI_P_PI || pinter.length() >= PI_P_PI {
            return PeriodicInterval::default();
        }
        let mut pinter_inf: f64 = pinter.binf + PI_P_PI;
        let mut pinter_sup: f64 = pinter.bsup + PI_P_PI;
        if pinter_inf > self.bsup {
            pinter_inf = pinter.binf - PI_P_PI;
            pinter_sup = pinter.bsup - PI_P_PI;
        }
        if (pinter_sup < self.binf) || (pinter_inf > self.bsup) {
            return PeriodicInterval::default();
        }
        let a: f64 = if pinter_inf > self.binf {
            pinter_inf
        } else {
            self.binf
        };
        let b: f64 = if pinter_sup < self.bsup {
            pinter_sup
        } else {
            self.bsup
        };
        PeriodicInterval::new(a, b)
    }
}
