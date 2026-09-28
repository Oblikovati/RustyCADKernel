// Copyright (c) 2009-2014 OPEN CASCADE SAS
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
// Rust translation of Poly_MakeLoops3D and Poly_MakeLoops2D in
// FoundationClasses/TKMath/Poly/Poly_MakeLoops.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native `chooseLeftWay` overrides: the smallest signed turn from the incoming
//! tangent, measured around the surface normal (3D) or in the plane (2D).
use super::{Link, LinkTable, LoopWay, MakeLoops, MakeLoopsHelper, get_link};
use crate::gp::{Direction2, Direction3, Xyz};
use std::f64::consts::PI;
/// `Poly_MakeLoops3D::Helper`: tangents and normals; `None` when undefined.
pub trait MakeLoops3DHelper: MakeLoopsHelper {
    /// Tangent at the first node of a link; `helper.get_first_tangent(&link)`.
    fn get_first_tangent(&self, link: &Link) -> Option<Direction3>;
    /// Tangent at the last node of a link; `helper.get_last_tangent(&link)`.
    fn get_last_tangent(&self, link: &Link) -> Option<Direction3>;
    /// Surface normal at a node; `helper.get_normal(3)`.
    fn get_normal(&self, node: i32) -> Option<Direction3>;
}
/// `Poly_MakeLoops2D::Helper`: planar tangents; `None` when undefined.
pub trait MakeLoops2DHelper: MakeLoopsHelper {
    /// Tangent at the first node of a link; `helper.get_first_tangent(&link)`.
    fn get_first_tangent(&self, link: &Link) -> Option<Direction2>;
    /// Tangent at the last node of a link; `helper.get_last_tangent(&link)`.
    fn get_last_tangent(&self, link: &Link) -> Option<Direction2>;
}
/// The `Poly_MakeLoops3D` strategy; `MakeLoops::new(LeftWay3D::new(&helper))`.
pub struct LeftWay3D<'h> {
    helper: &'h dyn MakeLoops3DHelper,
}
impl<'h> LeftWay3D<'h> {
    /// Wraps a 3D helper; `LeftWay3D::new(&helper)`.
    pub fn new(helper: &'h dyn MakeLoops3DHelper) -> Self {
        Self { helper }
    }
}
/// `Poly_MakeLoops3D`; `MakeLoops3D::new(LeftWay3D::new(&helper))`.
pub type MakeLoops3D<'h> = MakeLoops<LeftWay3D<'h>>;
/// `Poly_MakeLoops2D`; `MakeLoops2D::new(LeftWay2D::new(true, &helper))`.
pub type MakeLoops2D<'h> = MakeLoops<LeftWay2D<'h>>;
// Tangent projected to the plane orthogonal to the normal, when long enough.
fn projected(normal: Direction3, tangent: Direction3) -> Option<Direction3> {
    let normal = Xyz::from(normal);
    let projection = normal.cross_crossed(Xyz::from(tangent), normal);
    if projection.square_modulus() < 1e-14 {
        return None;
    }
    Direction3::from_xyz(projection).ok()
}
// Keeps the candidate with the smallest angle; an angle near -PI counts as PI.
fn smallest_turn(candidates: &[i32], mut angle_of: impl FnMut(i32) -> Option<f64>) -> i32 {
    let mut best = (PI * 2.0, 0);
    for &candidate in candidates {
        let Some(mut angle) = angle_of(candidate) else {
            continue;
        };
        if angle < 1e-4 - PI {
            angle = PI;
        }
        if angle < best.0 {
            best = (angle, candidate);
        }
    }
    if best.1 == 0 { candidates[0] } else { best.1 }
}
impl LoopWay for LeftWay3D<'_> {
    fn helper(&self) -> &dyn MakeLoopsHelper {
        self.helper
    }
    fn choose_left_way(
        &self,
        links: &LinkTable,
        node: i32,
        segment: i32,
        candidates: &[i32],
    ) -> i32 {
        let Some(normal) = self.helper.get_normal(node) else {
            return candidates[0];
        };
        let reference = self
            .helper
            .get_last_tangent(&get_link(links, segment))
            .and_then(|tangent| projected(normal, tangent));
        let Some(reference) = reference else {
            return candidates[0];
        };
        smallest_turn(candidates, |candidate| {
            let tangent = self.helper.get_first_tangent(&get_link(links, candidate))?;
            Some(projected(normal, tangent)?.angle_with_ref(reference, normal))
        })
    }
}
/// The `Poly_MakeLoops2D` strategy, turning left or (when `left_way` is false) right;
/// `LeftWay2D::new(true, &helper)`.
pub struct LeftWay2D<'h> {
    right_way: bool,
    helper: &'h dyn MakeLoops2DHelper,
}
impl<'h> LeftWay2D<'h> {
    /// Wraps a 2D helper; `LeftWay2D::new(true, &helper)`.
    pub fn new(left_way: bool, helper: &'h dyn MakeLoops2DHelper) -> Self {
        Self {
            right_way: !left_way,
            helper,
        }
    }
}
impl LoopWay for LeftWay2D<'_> {
    fn helper(&self) -> &dyn MakeLoopsHelper {
        self.helper
    }
    fn choose_left_way(
        &self,
        links: &LinkTable,
        _node: i32,
        segment: i32,
        candidates: &[i32],
    ) -> i32 {
        let Some(reference) = self.helper.get_last_tangent(&get_link(links, segment)) else {
            return candidates[0];
        };
        smallest_turn(candidates, |candidate| {
            let tangent = self.helper.get_first_tangent(&get_link(links, candidate))?;
            let angle = tangent.angle(reference);
            Some(if self.right_way { -angle } else { angle })
        })
    }
}
