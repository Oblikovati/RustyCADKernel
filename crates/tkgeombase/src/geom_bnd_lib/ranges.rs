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
// Rust translation of the infinite-parameter branches of GeomBndLib_Line(2d).hxx,
// GeomBndLib_Parabola(2d).cxx, GeomBndLib_Hyperbola(2d).cxx and BndLib.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The infinite-parameter case analysis that the line, parabola and hyperbola boxes of
//! GeomBndLib and BndLib repeat with identical bodies: one function per shape of the
//! analysis, each keeping the native order of point additions.
use super::infinite::InfiniteDirection;
use super::space::SampleBounds;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};

/// A box whose every min (or max) side can be opened at once (the parabola and
/// hyperbola branches open X, Y and Z together).
pub trait OpenSides: SampleBounds {
    /// `OpenXmin(); OpenYmin(); OpenZmin();`; `bounds.open_all_min()`.
    fn open_all_min(&mut self);
    /// `OpenXmax(); OpenYmax(); OpenZmax();`; `bounds.open_all_max()`.
    fn open_all_max(&mut self);
}

impl OpenSides for BoundingBox3 {
    fn open_all_min(&mut self) {
        self.open_xmin();
        self.open_ymin();
        self.open_zmin();
    }
    fn open_all_max(&mut self) {
        self.open_xmax();
        self.open_ymax();
        self.open_zmax();
    }
}

impl OpenSides for BoundingBox2 {
    fn open_all_min(&mut self) {
        self.open_xmin();
        self.open_ymin();
    }
    fn open_all_max(&mut self) {
        self.open_xmax();
        self.open_ymax();
    }
}

// The class of one parameter against Precision::Infinite().
#[derive(Clone, Copy, PartialEq)]
enum End {
    NegativeInfinite,
    PositiveInfinite,
    Finite,
}

fn end(u: f64) -> End {
    if precision::is_negative_infinite(u) {
        End::NegativeInfinite
    } else if precision::is_positive_infinite(u) {
        End::PositiveInfinite
    } else {
        End::Finite
    }
}

// The native `throw Standard_Failure(message)` of two equal infinite parameters.
fn bad_parameter(message: &str, u1: f64, u2: f64) -> Failure {
    Failure::new(&format!(
        "{message} (U1 = {u1:e}, U2 = {u2:e}; expected at most one infinite end per side)"
    ))
}

/// The line analysis (`GeomBndLib_Line::Box`, `BndLib::Add(gp_Lin)` and their 2D twins):
/// an infinite end opens the sides the direction reaches, a doubly infinite range adds
/// the point at 0; `add_line_range(&mut bounds, dir, (0.0, 1.0), &value, "...")?`.
pub fn add_line_range<D, B>(
    bounds: &mut B,
    direction: D,
    (u1, u2): (f64, f64),
    value: &dyn Fn(f64) -> B::Point,
    message: &str,
) -> Result<(), Failure>
where
    D: InfiniteDirection<Bounds = B>,
    B: SampleBounds,
{
    match (end(u1), end(u2)) {
        (End::NegativeInfinite, End::NegativeInfinite)
        | (End::PositiveInfinite, End::PositiveInfinite) => {
            return Err(bad_parameter(message, u1, u2));
        }
        (End::Finite, second) => {
            bounds.add_sample(value(u1));
            match second {
                End::NegativeInfinite => direction.open_min(bounds),
                End::PositiveInfinite => direction.open_max(bounds),
                End::Finite => bounds.add_sample(value(u2)),
            }
        }
        (_, End::Finite) => {
            // An infinite U1 alone: its side opens, then U2's point is added.
            if end(u1) == End::NegativeInfinite {
                direction.open_min(bounds);
            } else {
                direction.open_max(bounds);
            }
            bounds.add_sample(value(u2));
        }
        _ => {
            direction.open_min_max(bounds);
            bounds.add_sample(value(0.0));
        }
    }
    Ok(())
}

/// The open-branch analysis of the parabola and hyperbola boxes: an infinite end opens
/// every min (U -> -inf) or max (U -> +inf) side; a finite range adds U1 and runs
/// `finite`; `add_branch_range(&mut bounds, (0.0, 1.0), &value, &mut finite, "...")?`.
pub fn add_branch_range<B: OpenSides>(
    bounds: &mut B,
    (u1, u2): (f64, f64),
    value: &dyn Fn(f64) -> B::Point,
    finite: &mut dyn FnMut(&mut B),
    message: &str,
) -> Result<(), Failure> {
    let (first, second) = (end(u1), end(u2));
    if first != End::Finite && first == second {
        return Err(bad_parameter(message, u1, u2));
    }
    if first == End::Finite {
        bounds.add_sample(value(u1));
    } else if second == End::Finite {
        bounds.add_sample(value(u2));
    }
    match (first, second) {
        (End::Finite, End::Finite) => finite(bounds),
        (End::NegativeInfinite, _) | (_, End::NegativeInfinite) => bounds.open_all_min(),
        _ => {}
    }
    if first == End::PositiveInfinite || second == End::PositiveInfinite {
        bounds.open_all_max();
    }
    Ok(())
}

/// A patch builder of [`add_v_range`]: add the finite V band `[v1, v2]` to the box.
pub type VBand<'f> = dyn FnMut(&mut BoundingBox3, f64, f64) -> Result<(), Failure> + 'f;

/// The V analysis of the cylinder and cone boxes (GeomBndLib_Cylinder/Cone::Box and
/// BndLib::Add(gp_Cylinder/gp_Cone)): an infinite V end replaces its band limit by 0
/// and opens the axis direction on its side; a doubly infinite range runs `whole` then
/// opens both sides; two infinite ends of the same sign raise Standard_Failure;
/// `add_v_range(&mut bounds, axis, (0.0, 1.0), &mut band, &mut whole, "...")?`.
pub fn add_v_range(
    bounds: &mut BoundingBox3,
    direction: rustycad_tkmath::gp::Direction3,
    (v_min, v_max): (f64, f64),
    band: &mut VBand<'_>,
    whole: &mut dyn FnMut(&mut BoundingBox3) -> Result<(), Failure>,
    message: &str,
) -> Result<(), Failure> {
    match (end(v_min), end(v_max)) {
        (End::NegativeInfinite, End::NegativeInfinite)
        | (End::PositiveInfinite, End::PositiveInfinite) => {
            Err(bad_parameter(message, v_min, v_max))
        }
        (End::NegativeInfinite, End::PositiveInfinite)
        | (End::PositiveInfinite, End::NegativeInfinite) => {
            whole(bounds)?;
            direction.open_min_max(bounds);
            Ok(())
        }
        (first, End::Finite) if first != End::Finite => {
            band(bounds, 0.0, v_max)?;
            open_towards(bounds, direction, first);
            Ok(())
        }
        (End::Finite, End::Finite) => band(bounds, v_min, v_max),
        (_, second) => {
            band(bounds, v_min, 0.0)?;
            open_towards(bounds, direction, second);
            Ok(())
        }
    }
}

// `OpenMin` for a negative infinite end, `OpenMax` for a positive one.
fn open_towards(bounds: &mut BoundingBox3, direction: rustycad_tkmath::gp::Direction3, side: End) {
    if side == End::NegativeInfinite {
        direction.open_min(bounds);
    } else {
        direction.open_max(bounds);
    }
}
