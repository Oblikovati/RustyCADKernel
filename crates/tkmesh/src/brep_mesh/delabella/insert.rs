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

// Rust translation of CDelaBella::Triangulate() of delabella.cpp: "ACTUAL ALGORITHM",
// the incremental insertion of each lifted point into the convex hull, and
// Face::Next.

//! Each new point deletes the hull faces it sees and is joined to their silhouette by
//! a cone of new faces.
use super::seed::SeedFace;
use super::{DelaBella, broken_hull};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math_poly::LongDouble;

/// `ab`: for the neighbour across `f[e]`, the vertex slots of the shared edge.
const SHARED_EDGE: [[usize; 2]; 3] = [[1, 2], [2, 0], [0, 1]];

/// The insertion state shared by the steps of one point.
struct Insertion {
    /// The face free list (`cache`).
    cache: Option<usize>,
    /// The new point `_q`.
    q: usize,
}

impl DelaBella {
    /// The cone hull, every remaining point inserted, then the output lists.
    pub(super) fn finish_hull(
        &mut self,
        seed: &SeedFace,
        last: usize,
        first: usize,
        points: usize,
    ) -> Result<i32, Failure> {
        let mut cache = Some(0);
        let mut hull = self.build_cone(&mut cache, seed, last, first)?;
        for i in first + 1..points {
            let mut step = Insertion { cache, q: i };
            let visible = self.first_visible_face(hull, i)?;
            self.replace_visible_faces(&mut step, visible)?;
            hull = 2 * i - 3;
            self.sew_cone(hull)?;
            cache = step.cache;
        }
        self.link_output_lists(points)
    }

    /// "FIND FIRST VISIBLE FACE": around the previous point, then (the emergency
    /// fallback) downwards from the last allocated face.
    fn first_visible_face(&self, hull: usize, i: usize) -> Result<usize, Failure> {
        let mut face = hull;
        while self.face_dot(face, i) <= LongDouble::ZERO {
            face = self.next_around(face, i - 1)?;
            if face == hull {
                return self.scan_for_visible_face(2 * i - 5, i);
            }
        }
        Ok(face)
    }

    /// The fallback scan; running below the first face is undefined natively
    /// (`assert(_f != face_alloc)` is compiled out).
    fn scan_for_visible_face(&self, start: usize, q: usize) -> Result<usize, Failure> {
        let mut face = start;
        while self.face_dot(face, q) <= LongDouble::ZERO {
            if face == 0 {
                return Err(broken_hull("no hull face visible from point", q));
            }
            face -= 1;
        }
        Ok(face)
    }

    /// `Face::Next(p)`: the next face around `p` in the face winding.
    fn next_around(&self, face: usize, p: usize) -> Result<usize, Failure> {
        let stored = &self.faces[face];
        let slot = match stored.v.iter().position(|&vert| vert == p) {
            Some(0) => 1,
            Some(1) => 2,
            Some(_) => 0,
            None => return Err(broken_hull("face without the previous point", face)),
        };
        stored.f[slot].ok_or_else(|| broken_hull("face without neighbour", face))
    }

    /// "DELETE VISIBLE FACES & ADD NEW ONES": a depth-first walk over the visible
    /// faces whose `next` pointer marks them as on the stack (the last item points to
    /// itself).
    fn replace_visible_faces(&mut self, step: &mut Insertion, first: usize) -> Result<(), Failure> {
        let mut stack = Some(first);
        self.faces[first].next = Some(first);
        while let Some(face) = stack {
            stack = self.faces[face].next.filter(|&below| below != face);
            self.faces[face].next = None;
            let (fv, ff) = (self.faces[face].v, self.faces[face].f);
            self.free_face(face, &mut step.cache);
            for (e, neighbour) in ff.into_iter().enumerate() {
                let Some(neighbour) = neighbour else { continue };
                if self.faces[neighbour].next.is_some() {
                    continue;
                }
                stack = self.visit_neighbour(step, face, fv, e, neighbour, stack)?;
            }
        }
        Ok(())
    }

    /// One neighbour of a deleted face: a silhouette edge gets a cone side, a visible
    /// neighbour is detached and pushed. Returns the new stack top.
    fn visit_neighbour(
        &mut self,
        step: &mut Insertion,
        face: usize,
        fv: [usize; 3],
        e: usize,
        neighbour: usize,
        stack: Option<usize>,
    ) -> Result<Option<usize>, Failure> {
        if self.face_dot(neighbour, step.q) <= LongDouble::ZERO {
            let [a, b] = SHARED_EDGE[e].map(|slot| fv[slot]);
            let side = self.new_face(&mut step.cache, [a, b, step.q])?;
            self.faces[side].f[2] = Some(neighbour);
            self.replace_neighbour(neighbour, face, Some(side));
            // The silhouette needed for sewing the sides in the second pass.
            self.verts[a].sew = Some(side);
            self.verts[a].next = Some(b);
            return Ok(stack);
        }
        // Disjoin visible faces so they are not processed more than once.
        self.replace_neighbour(neighbour, face, None);
        self.faces[neighbour].next = Some(stack.unwrap_or(neighbour));
        Ok(Some(neighbour))
    }

    /// The first slot of `face` that links to `old` gets `new` (none matching is
    /// `assert(0)` natively, a no-op in the NDEBUG build).
    fn replace_neighbour(&mut self, face: usize, old: usize, new: Option<usize>) {
        let links = &mut self.faces[face].f;
        if let Some(slot) = links.iter().position(|&link| link == Some(old)) {
            links[slot] = new;
        }
    }

    /// "SEW SIDES OF CONE BUILT ON SILHOUETTE SEGMENTS", starting at the silhouette
    /// edge of the last added face.
    fn sew_cone(&mut self, last_face: usize) -> Result<(), Failure> {
        let entry = self.faces[last_face].v[0];
        let mut pr = entry;
        loop {
            let nx = self.next_vert(pr)?;
            let pr_sew = self.sew_of(pr)?;
            let nx_sew = self.sew_of(nx)?;
            self.faces[pr_sew].f[0] = Some(nx_sew);
            self.faces[nx_sew].f[1] = Some(pr_sew);
            pr = nx;
            if pr == entry {
                return Ok(());
            }
        }
    }

    /// The cone side a silhouette vertex starts.
    fn sew_of(&self, vert: usize) -> Result<usize, Failure> {
        self.verts[vert]
            .sew
            .ok_or_else(|| broken_hull("silhouette vertex without cone side", vert))
    }
}
