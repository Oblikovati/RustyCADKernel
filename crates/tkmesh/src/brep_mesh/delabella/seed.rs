/*

MIT License

DELABELLA - Delaunay triangulation library
Copyright (C) 2018 GUMIX - Marcin Sokalski

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

*/

// Rust translation of CDelaBella::Triangulate() of delabella.cpp: sorting, duplicate
// removal, the degenerate outputs and the coplanar 2D contour that seeds the hull.

//! The first stage of `CDelaBella::Triangulate()` up to the cone hull.
use super::arith::{Norm, overlap, u28cmp, vert_diff};
use super::{DelaBella, DelaBellaTriangle, broken_hull};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math_poly::LongDouble;

/// The silhouette contour while the input is coplanar: `head -> tail -> last -> head`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Contour {
    pub(super) head: usize,
    pub(super) tail: usize,
    pub(super) last: usize,
}

/// The temporary face `f` of the native code: its normal is computed once from the
/// first 3 points and kept while its vertices move.
#[derive(Clone, Copy, Debug)]
pub(super) struct SeedFace {
    pub(super) v: [usize; 3],
    pub(super) n: Norm,
}

impl DelaBella {
    /// `CDelaBella::Triangulate()` over the lifted vertices.
    pub(super) fn triangulate(&mut self) -> Result<i32, Failure> {
        self.verts.sort_by(u28cmp);
        let points = self.remove_dups(self.verts.len());
        if points < 3 {
            return Ok(self.short_output(points));
        }
        self.reset_faces(2 * points - 4);
        let mut seed = SeedFace {
            v: [0, 1, 2],
            n: self.cross_of([0, 1, 2]),
        };
        let colinear = seed.n.z == LongDouble::ZERO;
        let (mut contour, mut i) = self.grow_contour(&mut seed, points);
        if i == points {
            if colinear {
                return Ok(self.colinear_output(contour, points));
            }
            if points == 3 {
                return self.trivial_output(&seed, contour);
            }
            i = self.retract_cocircular(&mut contour, i)?;
        }
        self.finish_hull(&seed, contour.last, i, points)
    }

    /// All of more than 3 points are cocircular: the last point is retracted to
    /// become the cone's top (the contour keeps linking it, as natively).
    fn retract_cocircular(&mut self, contour: &mut Contour, i: usize) -> Result<usize, Failure> {
        self.log("[NFO] all input points are cocircular.\n");
        contour.last = contour.head;
        contour.head = self.next_vert(contour.head)?;
        Ok(i - 1)
    }

    /// The "remove dups" block: compacts the sorted vertices, keeping the first of
    /// each run with equal x and y; returns the remaining count.
    fn remove_dups(&mut self, points: usize) -> usize {
        let (mut w, mut r) = (0, 1);
        while r < points && !overlap(&self.verts[r], &self.verts[w]) {
            w += 1;
            r += 1;
        }
        w += 1;
        while r < points {
            r += 1;
            while r < points && overlap(&self.verts[r], &self.verts[r - 1]) {
                r += 1;
            }
            while r < points && !overlap(&self.verts[r], &self.verts[r - 1]) {
                self.verts[w] = self.verts[r];
                w += 1;
                r += 1;
            }
        }
        if points != w {
            self.log(&format!(
                "[WRN] detected {} dups in xy array!\n",
                points - w
            ));
        }
        w
    }

    /// Fewer than 3 distinct points: a single segment or a single point, `-points`.
    fn short_output(&mut self, points: usize) -> i32 {
        self.first_hull_vert = Some(0);
        if points == 2 {
            self.log("[WRN] all input points are colinear, returning single segment!\n");
            self.verts[0].next = Some(1);
            self.verts[1].next = None;
        } else {
            self.log("[WRN] all input points are identical, returning single point!\n");
            self.verts[0].next = None;
        }
        -(points as i32)
    }

    /// The face pool and its free list in index order (`face_alloc[i - 1].next =
    /// face_alloc + i`); the faces keep no state of an earlier call that is read.
    fn reset_faces(&mut self, hull_faces: usize) {
        let blank = DelaBellaTriangle {
            v: [0; 3],
            f: [None; 3],
            next: None,
            n: Norm::default(),
        };
        self.faces = vec![blank; hull_faces];
        for i in 1..hull_faces {
            self.faces[i - 1].next = Some(i);
        }
    }

    /// The silhouette successor of a vertex.
    pub(super) fn next_vert(&self, vert: usize) -> Result<usize, Failure> {
        self.verts[vert]
            .next
            .ok_or_else(|| broken_hull("silhouette vertex without successor", vert))
    }
}

impl DelaBella {
    /// "UNTIL INPUT IS COPLANAR, GROW IT IN FORM OF A 2D CONTOUR": the contour and the
    /// index of the first point off the seed plane.
    fn grow_contour(&mut self, seed: &mut SeedFace, points: usize) -> (Contour, usize) {
        let mut contour = Contour {
            head: 0,
            tail: 1,
            last: 2,
        };
        self.verts[0].next = Some(1);
        self.verts[1].next = Some(2);
        self.verts[2].next = Some(0);
        let mut i = 3;
        while i < points && self.plane_dot(&seed.n, seed.v[0], i) == LongDouble::ZERO {
            self.attach_to_contour(seed, &mut contour, i);
            contour.last = i;
            i += 1;
        }
        (contour, i)
    }

    /// Inserts the coplanar point `v` on top of the contour edge it sees.
    fn attach_to_contour(&mut self, seed: &mut SeedFace, contour: &mut Contour, v: usize) {
        let Contour { head, tail, last } = *contour;
        let lvh_norm = vert_diff(&self.verts[v], &self.verts[last])
            .cross(&vert_diff(&self.verts[head], &self.verts[last]));
        let lvh = seed.n.agrees_with(&lvh_norm);
        let tvl_norm = vert_diff(&self.verts[v], &self.verts[tail])
            .cross(&vert_diff(&self.verts[last], &self.verts[tail]));
        let tvl = seed.n.agrees_with(&tvl_norm);
        if lvh && !tvl {
            // On top of e(2,0) = (last, head).
            seed.v[1] = last;
            seed.v[2] = v;
            self.verts[last].next = Some(v);
            self.verts[v].next = Some(head);
            contour.tail = last;
        } else if tvl && !lvh {
            // On top of e(1,2) = (tail, last).
            seed.v[0] = last;
            seed.v[2] = v;
            self.verts[tail].next = Some(v);
            self.verts[v].next = Some(last);
            contour.head = last;
        }
        // Neither: `assert(0)` natively, a no-op in the NDEBUG build; the point is not
        // linked, yet the caller still makes it `last`, exactly as there.
    }

    /// All points colinear: the contour broken into a list, `-points`.
    fn colinear_output(&mut self, contour: Contour, points: usize) -> i32 {
        self.log("[WRN] all input points are colinear, returning segment list!\n");
        self.first_hull_vert = Some(contour.head);
        self.verts[contour.last].next = None;
        -(points as i32)
    }

    /// "trivial case of 3 points": one Delaunay and one hull face, glued.
    fn trivial_output(&mut self, seed: &SeedFace, contour: Contour) -> Result<i32, Failure> {
        self.log("[NFO] trivial case of 3 points, thank you.\n");
        let mut cache = Some(0);
        let dela = self.alloc_face(&mut cache)?;
        let hull = self.alloc_face(&mut cache)?;
        self.faces[dela].f = [Some(hull); 3];
        self.faces[hull].f = [Some(dela); 3];
        let Contour { head, tail, last } = contour;
        for vert in [head, tail, last] {
            self.verts[vert].sew = Some(hull);
        }
        self.wind_trivial_faces(seed, contour, [dela, hull]);
        self.faces[dela].n = self.cross_of(self.faces[dela].v);
        self.faces[hull].n = self.cross_of(self.faces[hull].v);
        self.first_dela_face = Some(dela);
        self.first_hull_face = Some(hull);
        Ok(3)
    }

    /// The vertex order of the 2 trivial faces: the Delaunay face clockwise, and the
    /// silhouette reversed when the seed normal points down.
    fn wind_trivial_faces(&mut self, seed: &SeedFace, contour: Contour, [dela, hull]: [usize; 2]) {
        let Contour { head, tail, last } = contour;
        if seed.n.z < LongDouble::ZERO {
            self.faces[dela].v = [head, tail, last];
            self.faces[hull].v = [last, tail, head];
            self.verts[head].next = Some(last);
            self.verts[last].next = Some(tail);
            self.verts[tail].next = Some(head);
            self.first_hull_vert = Some(last);
        } else {
            self.faces[dela].v = [last, tail, head];
            self.faces[hull].v = [head, tail, last];
            self.first_hull_vert = Some(head);
        }
    }
}
