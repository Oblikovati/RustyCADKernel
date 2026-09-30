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
// Rust translation of IntPatch_PrmPrmIntersection::RemplitLin, RemplitTri and Remplit
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The filling of the grid cells crossed by a segment or a triangle of grid points.
use super::IntPatchPrmPrmIntersection;
use crate::int_patch::IntPatchPrmPrmIntersectionT3Bits;

impl IntPatchPrmPrmIntersection {
    /// Sets the cell `(x, y, z)` of `map` when it is in the grid.
    fn add_cell(&self, [x, y, z]: [i32; 3], map: &mut IntPatchPrmPrmIntersectionT3Bits) {
        if self.dans_grille(x) != 0 && self.dans_grille(y) != 0 && self.dans_grille(z) != 0 {
            map.add(self.grille_integer(x, y, z));
        }
    }

    /// `RemplitLin(x1, y1, z1, x2, y2, z2, Map)`: sets the cells of the segment from the
    /// cell 1 to the cell 2 (by halving it until its ends are neighbours).
    /// `inter.remplit_lin([0, 0, 0], [5, 3, 1], &mut map)`.
    pub fn remplit_lin(
        &self,
        p1: [i32; 3],
        p2: [i32; 3],
        map: &mut IntPatchPrmPrmIntersectionT3Bits,
    ) {
        let xg: i32 = p1[0].wrapping_sub(p2[0]).wrapping_abs();
        let yg: i32 = p1[1].wrapping_sub(p2[1]).wrapping_abs();
        let zg: i32 = p1[2].wrapping_sub(p2[2]).wrapping_abs();
        self.add_cell(p1, map);
        if xg <= 1 && yg <= 1 && zg <= 1 {
            return;
        }
        let g: [i32; 3] = [0, 1, 2].map(|k| p1[k].wrapping_add(p2[k]) >> 1);
        self.remplit_lin(p1, g, map);
        self.remplit_lin(p2, g, map);
    }

    /// `RemplitTri(x1, y1, z1, x2, y2, z2, x3, y3, z3, Map)`: sets the cells of the
    /// triangle of the cells 1, 2 and 3 (by splitting it at its centroid cell).
    /// `inter.remplit_tri([0, 0, 0], [5, 0, 0], [0, 5, 0], &mut map)`.
    pub fn remplit_tri(
        &self,
        p1: [i32; 3],
        p2: [i32; 3],
        p3: [i32; 3],
        map: &mut IntPatchPrmPrmIntersectionT3Bits,
    ) {
        if p1 == p2 && p1 == p3 {
            self.add_cell(p1, map);
            return;
        }
        let g: [i32; 3] = [0, 1, 2].map(|k| p1[k].wrapping_add(p2[k]).wrapping_add(p3[k]) / 3);
        for (a, b, c) in [(p1, p2, p3), (p2, p1, p3), (p3, p2, p1)] {
            if g == a {
                self.remplit_lin(a, b, map);
                self.remplit_lin(a, c, map);
                return;
            }
        }
        self.add_cell(g, map);
        if g != p3 {
            self.remplit_tri(p1, p2, g, map);
        }
        if g != p1 {
            self.remplit_tri(g, p2, p3, map);
        }
        if g != p2 {
            self.remplit_tri(p1, g, p3, map);
        }
    }

    /// `Remplit(a, b, c, Map)`: sets the cells `a`, `b` and `c` (-1 for none) and, when
    /// all three are given, the cells of their triangle.
    /// `inter.remplit(a, b, c, &mut map)`.
    pub fn remplit(&self, a: i32, b: i32, c: i32, map: &mut IntPatchPrmPrmIntersectionT3Bits) {
        for code in [a, b, c] {
            if code != -1 {
                map.add(code);
            }
        }
        if a != -1 && b != -1 && c != -1 {
            let [pa, pb, pc] = [a, b, c].map(|code| {
                let (ix, iy, iz) = self.integer_grille(code);
                [ix, iy, iz]
            });
            self.remplit_tri(pa, pb, pc, map);
        }
    }
}
