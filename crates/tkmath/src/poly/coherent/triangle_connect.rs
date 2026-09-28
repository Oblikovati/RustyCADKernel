// Created on: 2007-11-25
// Created by: Alexander GRIGORIEV
// Copyright (c) 2007-2014 OPEN CASCADE SAS
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
// Rust translation of the SetConnection members of FoundationClasses/TKMath/Poly/Poly_CoherentTriangle.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Connections exist only between coherently oriented triangles: the shared side runs in
//! opposite directions in the two triangles.
use super::CoherentTriangle;
use rustycad_tkernel::standard::Failure;
/// One native branch: this side, the other's side and the other's node reported opposite.
type Pairing = (usize, usize, usize);
impl CoherentTriangle {
    /// Connects `other` across side `conn` of `this` when the side matches; returns whether it
    /// did; `CoherentTriangle::set_connection(&mut triangles, 0, 1, 1)?`.
    pub fn set_connection(
        triangles: &mut [Self],
        this: usize,
        conn: i32,
        other: usize,
    ) -> Result<bool, Failure> {
        const II: [usize; 5] = [2, 0, 1, 2, 0];
        let side = conn as usize;
        let (start, end) = (
            triangles[this].nodes[II[side + 2]],
            triangles[this].nodes[II[side]],
        );
        let theirs = triangles[other].nodes;
        // The first node of `other` equal to `start` decides; `end` must then precede it.
        let Some(at) = (0..3).find(|k| theirs[*k] == start) else {
            return Ok(false);
        };
        let before = (at + 2) % 3;
        if theirs[before] != end {
            return Ok(false);
        }
        let other_side = (at + 1) % 3;
        Self::link_pair(
            triangles,
            (this, side),
            (other, other_side),
            theirs[other_side],
        )?;
        Ok(true)
    }
    /// Finds the shared side of two triangles and connects them, unless they are already
    /// connected on it; `CoherentTriangle::set_connection_any(&mut triangles, 0, 1)?`.
    pub fn set_connection_any(
        triangles: &mut [Self],
        this: usize,
        other: usize,
    ) -> Result<bool, Failure> {
        let Some((own, theirs, opposite)) = Self::shared_side(triangles, this, other) else {
            return Ok(false);
        };
        let opposite_node = triangles[other].nodes[opposite];
        Self::link_pair(triangles, (this, own), (other, theirs), opposite_node)?;
        Ok(true)
    }
    // The native decision tree of SetConnection(Poly_CoherentTriangle&).
    fn shared_side(triangles: &[Self], this: usize, other: usize) -> Option<Pairing> {
        let mine = triangles[this].nodes;
        let t = triangles[other].nodes;
        let free = |side: usize| triangles[this].connected[side] != Some(other);
        let first =
            |pairs: [(bool, Pairing); 2]| pairs.into_iter().find(|(ok, _)| *ok).map(|(_, p)| p);
        if mine[0] == t[0] {
            return first([
                (mine[1] == t[2] && free(2), (2, 1, 1)),
                (mine[2] == t[1] && free(1), (1, 2, 2)),
            ]);
        }
        if mine[0] == t[1] {
            return first([
                (mine[1] == t[0] && free(2), (2, 2, 2)),
                (mine[2] == t[2] && free(1), (1, 0, 0)),
            ]);
        }
        if mine[0] == t[2] {
            return first([
                (mine[1] == t[1] && free(2), (2, 0, 0)),
                (mine[2] == t[0] && free(1), (1, 1, 1)),
            ]);
        }
        if !free(0) {
            return None;
        }
        [
            (mine[1] == t[0] && mine[2] == t[2], (0, 1, 1)),
            (mine[1] == t[2] && mine[2] == t[1], (0, 0, 0)),
            (mine[1] == t[1] && mine[2] == t[0], (0, 2, 2)),
        ]
        .into_iter()
        .find(|(ok, _)| *ok)
        .map(|(_, pairing)| pairing)
    }
}
