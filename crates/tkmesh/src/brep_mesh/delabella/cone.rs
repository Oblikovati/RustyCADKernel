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

// Rust translation of CDelaBella::Triangulate() of delabella.cpp: "CREATE CONE HULL
// WITH TOP AT cloud[i] AND BASE MADE OF CONTOUR LIST", and Face::Alloc/Free.

//! The cone hull over the coplanar contour, built in one of 2 windings depending on
//! the side of the contour plane the top vertex lies on.
use super::seed::SeedFace;
use super::{DelaBella, broken_hull};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math_poly::LongDouble;

/// The winding of the cone: which adjacency slots link consecutive sides and bases.
#[derive(Clone, Copy, Debug)]
struct ConeWinding {
    /// The top lies above the contour plane (`f.dot(*q) > 0`).
    upward: bool,
}

impl ConeWinding {
    /// The edge `(p, n)` in the face order of this winding.
    fn order(self, p: usize, n: usize) -> [usize; 2] {
        if self.upward { [p, n] } else { [n, p] }
    }
    /// The slot of a side that points back to the previous side.
    fn side_back(self) -> usize {
        usize::from(self.upward)
    }
    /// The slot of a side that points on to the next side.
    fn side_on(self) -> usize {
        1 - self.side_back()
    }
}

/// The cone under construction: its first/previous side and base faces.
struct ConeRun {
    first_side: usize,
    prev_side: usize,
    first_base: Option<usize>,
    prev_base: Option<usize>,
}

impl DelaBella {
    /// `Face::Alloc(&cache)`: pops the free list.
    pub(super) fn alloc_face(&mut self, cache: &mut Option<usize>) -> Result<usize, Failure> {
        let face = cache.ok_or_else(|| broken_hull("empty face pool", self.faces.len()))?;
        *cache = self.faces[face].next;
        self.faces[face].next = None;
        Ok(face)
    }

    /// `Face::Free(&cache)`: pushes the free list.
    pub(super) fn free_face(&mut self, face: usize, cache: &mut Option<usize>) {
        self.faces[face].next = *cache;
        *cache = Some(face);
    }

    /// An allocated face spanning `v` with its normal (`s->v[..] = ..; s->n = s->cross()`).
    pub(super) fn new_face(
        &mut self,
        cache: &mut Option<usize>,
        v: [usize; 3],
    ) -> Result<usize, Failure> {
        let face = self.alloc_face(cache)?;
        self.faces[face].v = v;
        self.faces[face].n = self.cross_of(v);
        Ok(face)
    }

    /// The cone with top `q` over the contour starting after `last`; returns the first
    /// side face (the initial `hull`).
    pub(super) fn build_cone(
        &mut self,
        cache: &mut Option<usize>,
        seed: &SeedFace,
        last: usize,
        q: usize,
    ) -> Result<usize, Failure> {
        let winding = ConeWinding {
            upward: self.plane_dot(&seed.n, seed.v[0], q) > LongDouble::ZERO,
        };
        let (mut run, mut p) = self.start_cone(cache, winding, last, q)?;
        let mut n = self.next_vert(p)?;
        loop {
            self.add_cone_segment(cache, winding, &mut run, [p, n, last, q])?;
            p = n;
            n = self.next_vert(n)?;
            if n == last {
                break;
            }
        }
        self.close_cone(cache, winding, &run, [p, n, q])?;
        Ok(run.first_side)
    }

    /// The first side face, over the contour edge leaving `last`; returns the run and
    /// the vertex after `last`.
    fn start_cone(
        &mut self,
        cache: &mut Option<usize>,
        winding: ConeWinding,
        last: usize,
        q: usize,
    ) -> Result<(ConeRun, usize), Failure> {
        let n = self.next_vert(last)?;
        let [a, b] = winding.order(last, n);
        let first_side = self.new_face(cache, [a, b, q])?;
        let run = ConeRun {
            first_side,
            prev_side: first_side,
            first_base: None,
            prev_base: None,
        };
        Ok((run, n))
    }

    /// One loop turn: the base face `(p, n, last)` and the side face `(p, n, q)`.
    fn add_cone_segment(
        &mut self,
        cache: &mut Option<usize>,
        winding: ConeWinding,
        run: &mut ConeRun,
        [p, n, last, q]: [usize; 4],
    ) -> Result<(), Failure> {
        let [a, b] = winding.order(n, p);
        let base = self.new_face(cache, [a, b, last])?;
        let [a, b] = winding.order(p, n);
        let side = self.new_face(cache, [a, b, q])?;
        self.faces[side].f[2] = Some(base);
        self.faces[base].f[2] = Some(side);
        self.faces[side].f[winding.side_back()] = Some(run.prev_side);
        self.faces[run.prev_side].f[winding.side_on()] = Some(side);
        // Bases chain through the opposite slots of the sides.
        self.faces[base].f[winding.side_on()] = run.prev_base;
        match run.prev_base {
            Some(prev_base) => self.faces[prev_base].f[winding.side_back()] = Some(base),
            None => run.first_base = Some(base),
        }
        run.prev_base = Some(base);
        run.prev_side = side;
        Ok(())
    }

    /// The last side face and the links that close the cone.
    fn close_cone(
        &mut self,
        cache: &mut Option<usize>,
        winding: ConeWinding,
        run: &ConeRun,
        [p, n, q]: [usize; 3],
    ) -> Result<(), Failure> {
        let (first_base, prev_base) = match (run.first_base, run.prev_base) {
            (Some(first), Some(prev)) => (first, prev),
            _ => return Err(broken_hull("cone without base face", run.first_side)),
        };
        let [a, b] = winding.order(p, n);
        let last_side = self.new_face(cache, [a, b, q])?;
        let (back, on) = (winding.side_back(), winding.side_on());
        self.faces[last_side].f[back] = Some(run.prev_side);
        self.faces[run.prev_side].f[on] = Some(last_side);
        self.faces[last_side].f[on] = Some(run.first_side);
        self.faces[run.first_side].f[back] = Some(last_side);
        self.faces[first_base].f[on] = Some(run.first_side);
        self.faces[run.first_side].f[2] = Some(first_base);
        self.faces[last_side].f[2] = Some(prev_base);
        self.faces[prev_base].f[back] = Some(last_side);
        Ok(())
    }
}
