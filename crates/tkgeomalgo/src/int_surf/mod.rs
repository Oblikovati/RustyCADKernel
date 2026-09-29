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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntSurf at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntSurf: the points, lines, transitions and quadrics shared by the
//! surface intersection algorithms, and the `IntSurf` class functions
//! (`MakeTransition`, `SetPeriod`).
//!
//! `IntSurf_Allocator` (a typedef of the NCollection allocator handle) is
//! `Option<BaseAllocator>` at the one member that takes it, `IntSurfLineOn2S::new`.
mod interior_point;
mod line_on_2s;
mod path_point;
mod pnt_on_2s;
mod quadric;
mod quadric_metric;
mod quadric_tool;
mod transition;

pub use interior_point::{IntSurfInteriorPoint, IntSurfInteriorPointTool};
pub use line_on_2s::IntSurfLineOn2S;
pub use path_point::{IntSurfPathPoint, IntSurfPathPointTool};
pub use pnt_on_2s::{IntSurfCouple, IntSurfPntOn2S};
pub use quadric::IntSurfQuadric;
pub use quadric_tool::IntSurfQuadricTool;
pub use transition::{IntSurfSituation, IntSurfTransition, IntSurfTypeTrans};

use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Direction3, Vector3};

/// IntSurf::MakeTransition: sets the transitions `t_first` and `t_second` of the points
/// of two crossing lines of tangents `tg_first` and `tg_second`, from the sign of the
/// mixed product with `normal`.
///
/// A null first tangent gives two undecided tangent transitions; a null second
/// tangent or parallel tangents give TOUCH transitions of Unknown situation, opposite
/// when the tangents point against each other; otherwise the sign of
/// `(tg_second ^ tg_first) . normal / (|tg_second| |tg_first|)` beyond 1e-4 decides IN/OUT.
/// The transitions are updated in place, as the native output arguments: an IN, OUT or
/// undecided result keeps their previous situation and opposite flag.
/// ```
/// use rustycad_tkgeomalgo::int_surf::{make_transition, IntSurfTransition, IntSurfTypeTrans};
/// use rustycad_tkmath::gp::{Direction3, Vector3};
/// let normal = Direction3::new(0.0, 0.0, 1.0).unwrap();
/// let (mut first, mut second) = (IntSurfTransition::new(), IntSurfTransition::new());
/// let (tg1, tg2) = (Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
/// make_transition(tg1, tg2, normal, &mut first, &mut second);
/// assert_eq!(first.transition_type(), IntSurfTypeTrans::Out);
/// assert_eq!(second.transition_type(), IntSurfTypeTrans::In);
/// ```
pub fn make_transition(
    tg_first: Vector3,
    tg_second: Vector3,
    normal: Direction3,
    t_first: &mut IntSurfTransition,
    t_second: &mut IntSurfTransition,
) {
    let pvect: Vector3 = tg_second.crossed(tg_first);
    let n_tg_second: f64 = tg_second.magnitude();
    let n_tg_first: f64 = tg_first.magnitude();
    let n_tg_second_n_tg_first_angular: f64 = n_tg_second * n_tg_first * precision::angular();
    if n_tg_first <= precision::confusion() {
        t_first.set_type(true, IntSurfTypeTrans::Undecided);
        t_second.set_type(true, IntSurfTypeTrans::Undecided);
        return;
    }
    if n_tg_second <= precision::confusion() || pvect.magnitude() <= n_tg_second_n_tg_first_angular
    {
        t_first.set_touch(
            true,
            IntSurfSituation::Unknown,
            tg_first.dot(tg_second) < 0.0,
        );
        t_second.set_touch(
            true,
            IntSurfSituation::Unknown,
            tg_first.dot(tg_second) < 0.0,
        );
        return;
    }
    let mut yu: f64 = pvect.dot(Vector3::from(normal));
    yu /= n_tg_second * n_tg_first;
    set_crossing_transitions(yu, t_first, t_second);
}

/// The IN/OUT pair of `MakeTransition` from the normalized mixed product.
fn set_crossing_transitions(
    yu: f64,
    t_first: &mut IntSurfTransition,
    t_second: &mut IntSurfTransition,
) {
    if yu > 0.0001 {
        t_first.set_type(false, IntSurfTypeTrans::In);
        t_second.set_type(false, IntSurfTypeTrans::Out);
    } else if yu < -0.0001 {
        t_first.set_type(false, IntSurfTypeTrans::Out);
        t_second.set_type(false, IntSurfTypeTrans::In);
    } else {
        t_first.set_type(true, IntSurfTypeTrans::Undecided);
        t_second.set_type(true, IntSurfTypeTrans::Undecided);
    }
}

/// IntSurf::SetPeriod: the periods `[U1, V1, U2, V2]` of two surfaces, 0 in a
/// direction that is not periodic; fails as the adaptors' period queries do.
///
/// `let periods: [f64; 4] = set_period(&*first, &*second)?;`
pub fn set_period(
    first_surf: &dyn Adaptor3dSurface,
    second_surf: &dyn Adaptor3dSurface,
) -> Result<[f64; 4], Failure> {
    Ok([
        period_or_zero(first_surf.is_u_periodic()?, || first_surf.u_period())?,
        period_or_zero(first_surf.is_v_periodic()?, || first_surf.v_period())?,
        period_or_zero(second_surf.is_u_periodic()?, || second_surf.u_period())?,
        period_or_zero(second_surf.is_v_periodic()?, || second_surf.v_period())?,
    ])
}

fn period_or_zero(
    periodic: bool,
    period: impl FnOnce() -> Result<f64, Failure>,
) -> Result<f64, Failure> {
    if periodic { period() } else { Ok(0.0) }
}
