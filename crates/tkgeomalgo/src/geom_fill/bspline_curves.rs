// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_BSplineCurves.hxx/.cxx/.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_BSplineCurves`: the B-spline surface filled between two, three or four
//! B-spline boundary curves.
use super::bspline_distribution::{arrange, set_same_distribution};
use super::curves_filling::{
    CurvedPair, FourBoundaries, SchemeGrids, closing_poles, curved_pair, curved_two, fill_scheme,
    live, ruled_grids, scheme_grids,
};
use super::{GeomFillFilling, GeomFillFillingStyle};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomBoundedCurve, GeomCurve};
use rustycad_tkmath::gp::Point3;

/// The B-spline surface of two to four B-spline boundaries (`GeomFill_BSplineCurves`),
/// filled in the `GeomFill_FillingStyle` asked for.
///
/// `let fill = GeomFillBSplineCurves::with_three(&c1, &c2, &c3, GeomFillFillingStyle::CoonsStyle)?; fill.surface()`
#[derive(Clone, Default)]
pub struct GeomFillBSplineCurves {
    my_surface: Handle<GeomBSplineSurface>,
}

type Curve = Handle<GeomBSplineCurve>;

impl GeomFillBSplineCurves {
    /// An empty filling with a null surface (`GeomFill_BSplineCurves()`);
    /// `GeomFillBSplineCurves::new().surface().is_null()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomFill_BSplineCurves(C1, C2, C3, C4, Type)`: see `init_four`.
    /// `GeomFillBSplineCurves::with_four(&c1, &c2, &c3, &c4, GeomFillFillingStyle::StretchStyle)?`.
    pub fn with_four(
        c1: &Curve,
        c2: &Curve,
        c3: &Curve,
        c4: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<Self, Failure> {
        let mut filling = Self::new();
        filling.init_four(c1, c2, c3, c4, style)?;
        Ok(filling)
    }
    /// `GeomFill_BSplineCurves(C1, C2, C3, Type)`: see `init_three`.
    /// `GeomFillBSplineCurves::with_three(&c1, &c2, &c3, GeomFillFillingStyle::CurvedStyle)?`.
    pub fn with_three(
        c1: &Curve,
        c2: &Curve,
        c3: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<Self, Failure> {
        let mut filling = Self::new();
        filling.init_three(c1, c2, c3, style)?;
        Ok(filling)
    }
    /// `GeomFill_BSplineCurves(C1, C2, Type)`: see `init_two`.
    /// `GeomFillBSplineCurves::with_two(&c1, &c2, GeomFillFillingStyle::StretchStyle)?`.
    pub fn with_two(c1: &Curve, c2: &Curve, style: GeomFillFillingStyle) -> Result<Self, Failure> {
        let mut filling = Self::new();
        filling.init_two(c1, c2, style)?;
        Ok(filling)
    }
    /// `Init(C1, C2, C3, C4, Type)`: copies of the curves are chained end to start
    /// within Precision::Confusion (closed curves first, reversing some),
    /// Standard_ConstructionError when they do not join; opposite curves get one
    /// degree and one knot vector (Standard_ConstructionError when their knots cannot
    /// be merged, OCC27704), the Coons style needs four poles each way
    /// (Standard_ConstructionError), and the scheme of `style` fills the poles;
    /// `fill.init_four(&c1, &c2, &c3, &c4, GeomFillFillingStyle::CoonsStyle)?`.
    pub fn init_four(
        &mut self,
        c1: &Curve,
        c2: &Curve,
        c3: &Curve,
        c4: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<(), Failure> {
        let Some(mut cc) = arrange([c1, c2, c3, c4], precision::confusion())? else {
            return Err(errors::construction(
                " GeomFill_BSplineCurves: Courbes non jointives; expected four joined curves",
            ));
        };
        raise_opposite_degrees(&cc)?;
        let [a, b, c, d] = &mut cc;
        let nb_u_poles: i32 = set_same_distribution(a, c)?;
        let nb_v_poles: i32 = set_same_distribution(b, d)?;
        if style == GeomFillFillingStyle::CoonsStyle && (nb_u_poles < 4 || nb_v_poles < 4) {
            return Err(errors::construction(&format!(
                "GeomFill_BSplineCurves: invalid filling style; {nb_u_poles} x {nb_v_poles} poles, expected at least 4 x 4 for the Coons style"
            )));
        }
        let caro: GeomFillFilling = four_scheme(&cc, style)?;
        let (u, v) = (live(&cc[0])?, live(&cc[1])?);
        let knots = (knot_vector(u), knot_vector(v));
        let surface = surface_of(
            scheme_grids(&caro)?,
            &knots,
            (u.degree(), v.degree()),
            false,
        )?;
        self.my_surface = Handle::new(surface);
        Ok(())
    }
    /// `Init(C1, C2, C3, Type)`: the fourth boundary is the linear B-spline joining
    /// the ends of `c1` and `c3` that do not touch `c2`, on the knot range of `c2`;
    /// `fill.init_three(&c1, &c2, &c3, GeomFillFillingStyle::CoonsStyle)?`.
    pub fn init_three(
        &mut self,
        c1: &Curve,
        c2: &Curve,
        c3: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<(), Failure> {
        let middle: &GeomBSplineCurve = live(c2)?;
        let poles: Array1<'static, Point3> =
            closing_poles([live(c1)?, middle, live(c3)?].map(|c| c as &dyn GeomBoundedCurve))?;
        let mut knots: Array1<'static, f64> = one_based(2)?;
        put(&mut knots, 1, middle.knot(middle.first_u_knot_index()?)?)?;
        put(&mut knots, 2, middle.knot(middle.last_u_knot_index()?)?)?;
        let mut mults: Array1<'static, i32> = one_based(2)?;
        mults.init(2);
        let c4: Curve = Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 1, false)?);
        self.init_four(c1, c2, c3, &c4, style)
    }
    /// `Init(C1, C2, Type)`: the ruled surface between copies of the curves given one
    /// degree and one knot vector (V knots 0 and 1), or, in the curved style, the
    /// translation surface of the first curve along the second after joining their
    /// ends (Standard_OutOfRange when they do not touch). Quirks kept: the ruled
    /// surface is periodic in U only when rational, and when the end of `c1` meets
    /// the start of `c2` the caller's `c1` is reversed, not its copy;
    /// `fill.init_two(&c1, &c2, GeomFillFillingStyle::StretchStyle)?`.
    pub fn init_two(
        &mut self,
        c1: &Curve,
        c2: &Curve,
        style: GeomFillFillingStyle,
    ) -> Result<(), Failure> {
        let mut cc1: Curve = Handle::new(GeomBSplineCurve::copy_of(live(c1)?)?);
        let mut cc2: Curve = Handle::new(GeomBSplineCurve::copy_of(live(c2)?)?);
        let is_rat: bool = live(&cc1)?.is_rational() || live(&cc2)?.is_rational();
        let surface = if style != GeomFillFillingStyle::CurvedStyle {
            ruled(&mut cc1, &mut cc2, is_rat)?
        } else {
            curved(live(c1)?, [live(&cc1)?, live(&cc2)?], is_rat)?
        };
        self.my_surface = Handle::new(surface);
        Ok(())
    }
    /// `Surface()`: the surface of the last `Init`, null before; `fill.surface()`.
    pub fn surface(&self) -> Handle<GeomBSplineSurface> {
        self.my_surface.clone()
    }
}

/// The degree raise of the four-curve `Init`: each curve to the higher degree of its
/// pair of opposite curves (the stored degrees are read before any raise).
fn raise_opposite_degrees(cc: &[Curve; 4]) -> Result<(), Failure> {
    let deg: Vec<i32> = cc
        .iter()
        .map(|c| live(c).map(|c| c.degree()))
        .collect::<Result<_, _>>()?;
    let (deg_u, deg_v) = (deg[0].max(deg[2]), deg[1].max(deg[3]));
    for (k, target) in [deg_u, deg_v, deg_u, deg_v].into_iter().enumerate() {
        if deg[k] < target {
            live(&cc[k])?.increase_degree(target)?;
        }
    }
    Ok(())
}

/// The scheme of `style` on the poles and (when one curve is rational) the weights
/// of the unified curves.
fn four_scheme(cc: &[Curve; 4], style: GeomFillFillingStyle) -> Result<GeomFillFilling, Failure> {
    let curves: Vec<&GeomBSplineCurve> = cc.iter().map(live).collect::<Result<_, _>>()?;
    let poles: Vec<Array1<'static, Point3>> = curves.iter().map(|c| c.poles()).collect();
    let weights: Vec<Array1<'static, f64>> = curves
        .iter()
        .map(|c| c.weights_array(|w| w.clone()))
        .collect();
    let is_rat: bool = curves.iter().any(|c| c.is_rational());
    let boundaries = FourBoundaries {
        poles: [&poles[0], &poles[1], &poles[2], &poles[3]],
        weights: is_rat.then_some([&weights[0], &weights[1], &weights[2], &weights[3]]),
    };
    fill_scheme(&boundaries, style)
}

/// The knots and multiplicities of a curve.
type KnotArrays = (Array1<'static, f64>, Array1<'static, i32>);

fn knot_vector(curve: &GeomBSplineCurve) -> KnotArrays {
    (curve.knots(), curve.multiplicities())
}

/// `new Geom_BSplineSurface(Poles, [Weights,] UKnots, VKnots, UMults, VMults, UDeg,
/// VDeg, UPeriodic, false)` on a scheme's grids.
fn surface_of(
    (poles, weights): SchemeGrids,
    (u, v): &(KnotArrays, KnotArrays),
    degrees: (i32, i32),
    u_periodic: bool,
) -> Result<GeomBSplineSurface, Failure> {
    let (u, v) = ((&u.0, &u.1), (&v.0, &v.1));
    match weights {
        Some(w) => GeomBSplineSurface::with_weights(&poles, &w, u, v, degrees, (u_periodic, false)),
        None => GeomBSplineSurface::new(&poles, u, v, degrees, (u_periodic, false)),
    }
}

/// The ruled surface of `Init(C1, C2)` outside the curved style: V knots 0 and 1 of
/// multiplicity 2, periodic in U like the first curve only when rational.
fn ruled(cc1: &mut Curve, cc2: &mut Curve, is_rat: bool) -> Result<GeomBSplineSurface, Failure> {
    let deg_u: i32 = live(cc1)?.degree().max(live(cc2)?.degree());
    for curve in [&*cc1, &*cc2] {
        let curve: &GeomBSplineCurve = live(curve)?;
        if curve.degree() < deg_u {
            curve.increase_degree(deg_u)?;
        }
    }
    let nb_poles: i32 = set_same_distribution(cc1, cc2)?;
    let (a, b) = (live(cc1)?, live(cc2)?);
    let (p1, p2) = (a.poles(), b.poles());
    let weights = is_rat.then(|| [a, b].map(|c| c.weights_array(|w| w.clone())));
    let grids = ruled_grids(
        [&p1, &p2],
        weights.as_ref().map(|[w1, w2]| [w1, w2]),
        nb_poles,
    )?;
    let mut v_knots: Array1<'static, f64> = one_based(2)?;
    (v_knots[1], v_knots[2]) = (0.0, 1.0);
    let mut v_mults: Array1<'static, i32> = one_based(2)?;
    v_mults.init(2);
    let knots = (knot_vector(a), (v_knots, v_mults));
    surface_of(grids, &knots, (a.degree(), 1), is_rat && a.is_periodic())
}

/// The translation surface of `Init(C1, C2)` in the curved style, on the knots of
/// the joined copies and their original degrees.
fn curved(
    caller_first: &GeomBSplineCurve,
    [a, b]: [&GeomBSplineCurve; 2],
    is_rat: bool,
) -> Result<GeomBSplineSurface, Failure> {
    let (deg1, deg2) = (a.degree(), b.degree());
    curved_pair(&CurvedPair {
        copies: [a, b],
        caller_first,
        message: "GeomFill_BSplineCurves: Courbes non jointives; expected two touching curves",
    })?;
    let rows = [a.poles(), b.poles()];
    let knots = (knot_vector(a), knot_vector(b));
    let weights = is_rat.then(|| [a, b].map(|c| c.weights_array(|w| w.clone())));
    let caro = curved_two(&rows, weights.as_ref())?;
    surface_of(scheme_grids(&caro)?, &knots, (deg1, deg2), false)
}
