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
// Rust translation of IntCurve_IntConicConic.hxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_IntConicConic`: the intersection of two gp conics; each Perform
//! overload lives in the module of its native source.
use super::{GpConic, IntImpConicParConic};
use crate::int_res2d::{Domain, Intersection, construction_error};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};
use std::ops::{Deref, DerefMut};

/// `IntCurve_IntConicConic`: an [`Intersection`] (reached through `Deref`) of two
/// conics. Lines and circles are intersected geometrically within the tolerances;
/// the other pairs go through [`IntImpConicParConic`] on the first conic as implicit
/// curve, the parametric domain of an unbounded conic being first reduced around
/// the analytic intersections (IntAna2d) of offset copies. A circle or ellipse
/// domain without equivalent parameters is closed on `[First, First + 2PI]` where
/// the native code does. Each constructor is named after its pair and each Perform
/// overload is the `perform_` method of that name; the exceptions of the domains
/// and of gp are returned as `Err`.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve::IntConicConic;
/// use rustycad_tkgeomalgo::int_res2d::Domain;
/// use rustycad_tkmath::gp::{Direction2, Line2, Point2};
/// let l1 = Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
/// let l2 = Line2::new(Point2::new(1.0, -1.0), Direction2::new(0.0, 1.0).unwrap());
/// let inter = IntConicConic::lines(l1, &Domain::new(), l2, &Domain::new(), 1e-9, 1e-9).unwrap();
/// assert_eq!(inter.point(1).unwrap().param_on_first(), 1.0);
/// ```
#[derive(Clone, Default)]
pub struct IntConicConic {
    pub(super) base: Intersection,
    pub(super) inter: IntImpConicParConic,
}

impl Deref for IntConicConic {
    type Target = Intersection;
    fn deref(&self) -> &Intersection {
        &self.base
    }
}

impl DerefMut for IntConicConic {
    fn deref_mut(&mut self) -> &mut Intersection {
        &mut self.base
    }
}

// One named constructor per native constructor: a new intersection, then the
// Perform overload of the same pair.
macro_rules! conic_pair_constructor {
    ($(#[$doc:meta])* $name:ident, $perform:ident, $first:ty, $second:ty) => {
        $(#[$doc])*
        pub fn $name(
            c1: $first,
            d1: &Domain,
            c2: $second,
            d2: &Domain,
            tol_conf: f64,
            tol: f64,
        ) -> Result<Self, Failure> {
            let mut inter: Self = Self::new();
            inter.$perform(c1, d1, c2, d2, tol_conf, tol)?;
            Ok(inter)
        }
    };
}

impl IntConicConic {
    /// `IntCurve_IntConicConic()`: not done. `IntConicConic::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Lin2d, D1, gp_Lin2d, D2, TolConf, Tol)`;
        /// `IntConicConic::lines(l1, &d1, l2, &d2, 1e-9, 1e-9)?`.
        lines, perform_lines, Line2, Line2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Lin2d, DL, gp_Circ2d, DC, TolConf, Tol)`;
        /// `IntConicConic::line_circle(l, &dl, c, &dc, 1e-9, 1e-9)?`.
        line_circle, perform_line_circle, Line2, Circle2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Lin2d, DL, gp_Elips2d, DE, TolConf, Tol)`;
        /// `IntConicConic::line_ellipse(l, &dl, e, &de, 1e-9, 1e-9)?`.
        line_ellipse, perform_line_ellipse, Line2, Ellipse2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Lin2d, DL, gp_Parab2d, DP, TolConf, Tol)`;
        /// `IntConicConic::line_parabola(l, &dl, p, &dp, 1e-9, 1e-9)?`.
        line_parabola, perform_line_parabola, Line2, Parabola2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Lin2d, DL, gp_Hypr2d, DH, TolConf, Tol)`;
        /// `IntConicConic::line_hyperbola(l, &dl, h, &dh, 1e-9, 1e-9)?`.
        line_hyperbola, perform_line_hyperbola, Line2, Hyperbola2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Circ2d, D1, gp_Circ2d, D2, TolConf, Tol)`;
        /// `IntConicConic::circles(c1, &d1, c2, &d2, 1e-9, 1e-9)?`.
        circles, perform_circles, Circle2, Circle2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Circ2d, DC, gp_Elips2d, DE, TolConf, Tol)`;
        /// `IntConicConic::circle_ellipse(c, &dc, e, &de, 1e-9, 1e-9)?`.
        circle_ellipse, perform_circle_ellipse, Circle2, Ellipse2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Circ2d, DC, gp_Parab2d, DP, TolConf, Tol)`;
        /// `IntConicConic::circle_parabola(c, &dc, p, &dp, 1e-9, 1e-9)?`.
        circle_parabola, perform_circle_parabola, Circle2, Parabola2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Circ2d, DC, gp_Hypr2d, DH, TolConf, Tol)`;
        /// `IntConicConic::circle_hyperbola(c, &dc, h, &dh, 1e-9, 1e-9)?`.
        circle_hyperbola, perform_circle_hyperbola, Circle2, Hyperbola2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Elips2d, D1, gp_Elips2d, D2, TolConf, Tol)`;
        /// `IntConicConic::ellipses(e1, &d1, e2, &d2, 1e-9, 1e-9)?`.
        ellipses, perform_ellipses, Ellipse2, Ellipse2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Elips2d, DE, gp_Parab2d, DP, TolConf, Tol)`;
        /// `IntConicConic::ellipse_parabola(e, &de, p, &dp, 1e-9, 1e-9)?`.
        ellipse_parabola, perform_ellipse_parabola, Ellipse2, Parabola2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Elips2d, DE, gp_Hypr2d, DH, TolConf, Tol)`;
        /// `IntConicConic::ellipse_hyperbola(e, &de, h, &dh, 1e-9, 1e-9)?`.
        ellipse_hyperbola, perform_ellipse_hyperbola, Ellipse2, Hyperbola2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Parab2d, D1, gp_Parab2d, D2, TolConf, Tol)`;
        /// `IntConicConic::parabolas(p1, &d1, p2, &d2, 1e-9, 1e-9)?`.
        parabolas, perform_parabolas, Parabola2, Parabola2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Parab2d, DP, gp_Hypr2d, DH, TolConf, Tol)`;
        /// `IntConicConic::parabola_hyperbola(p, &dp, h, &dh, 1e-9, 1e-9)?`.
        parabola_hyperbola, perform_parabola_hyperbola, Parabola2, Hyperbola2
    );
    conic_pair_constructor!(
        /// `IntCurve_IntConicConic(gp_Hypr2d, D1, gp_Hypr2d, D2, TolConf, Tol)`;
        /// `IntConicConic::hyperbolas(h1, &d1, h2, &d2, 1e-9, 1e-9)?`.
        hyperbolas, perform_hyperbolas, Hyperbola2, Hyperbola2
    );

    /// The Perform overload of an ordered pair of conics (the rank of `first` at
    /// most that of `second`); Standard_ConstructionError for a pair out of order,
    /// which has no native overload.
    /// `inter.perform_ordered(&GpConic::Line(l), &d1, &GpConic::Circle(c), &d2, 1e-9, 1e-9)?`.
    pub fn perform_ordered(
        &mut self,
        first: &GpConic,
        d1: &Domain,
        second: &GpConic,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        use GpConic::{Circle, Ellipse, Hyperbola, Line, Parabola};
        let (tc, t) = (tol_conf, tol);
        match (*first, *second) {
            (Line(a), Line(b)) => self.perform_lines(a, d1, b, d2, tc, t),
            (Line(a), Circle(b)) => self.perform_line_circle(a, d1, b, d2, tc, t),
            (Line(a), Ellipse(b)) => self.perform_line_ellipse(a, d1, b, d2, tc, t),
            (Line(a), Parabola(b)) => self.perform_line_parabola(a, d1, b, d2, tc, t),
            (Line(a), Hyperbola(b)) => self.perform_line_hyperbola(a, d1, b, d2, tc, t),
            (Circle(a), Circle(b)) => self.perform_circles(a, d1, b, d2, tc, t),
            (Circle(a), Ellipse(b)) => self.perform_circle_ellipse(a, d1, b, d2, tc, t),
            (Circle(a), Parabola(b)) => self.perform_circle_parabola(a, d1, b, d2, tc, t),
            (Circle(a), Hyperbola(b)) => self.perform_circle_hyperbola(a, d1, b, d2, tc, t),
            (Ellipse(a), Ellipse(b)) => self.perform_ellipses(a, d1, b, d2, tc, t),
            (Ellipse(a), Parabola(b)) => self.perform_ellipse_parabola(a, d1, b, d2, tc, t),
            (Ellipse(a), Hyperbola(b)) => self.perform_ellipse_hyperbola(a, d1, b, d2, tc, t),
            (Parabola(a), Parabola(b)) => self.perform_parabolas(a, d1, b, d2, tc, t),
            (Parabola(a), Hyperbola(b)) => self.perform_parabola_hyperbola(a, d1, b, d2, tc, t),
            (Hyperbola(a), Hyperbola(b)) => self.perform_hyperbolas(a, d1, b, d2, tc, t),
            (a, b) => Err(construction_error(&format!(
                "IntCurve_IntConicConic::Perform() - conic ranks {} then {}; expected a non-decreasing pair",
                a.rank(),
                b.rank()
            ))),
        }
    }
}
