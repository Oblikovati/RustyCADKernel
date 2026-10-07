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

// Rust translation of the arithmetic types of delabella.cpp (Signed14..Signed62,
// s14sqr, Norm, Vect, Vert::operator-, Face::dot/cross, Vert::u28cmp).

//! The configured arithmetic: xy coordinates and `Vect::xy` are `double`; z, `Vect::z`,
//! every `Norm` component and the dot products are `long double` (`LongDouble`).
use super::{DelaBella, DelaBellaVertex};
use rustycad_tkmath::math_poly::LongDouble;
use std::cmp::Ordering;

/// `Norm`: a cross product (`Signed45` x/y, `Signed31` z).
#[derive(Clone, Copy, Debug)]
pub(super) struct Norm {
    pub(super) x: LongDouble,
    pub(super) y: LongDouble,
    pub(super) z: LongDouble,
}

impl Default for Norm {
    fn default() -> Self {
        Self {
            x: LongDouble::ZERO,
            y: LongDouble::ZERO,
            z: LongDouble::ZERO,
        }
    }
}

/// `Vect`: a vertex difference (`Signed15` x/y in double, `Signed29` z).
#[derive(Clone, Copy, Debug)]
pub(super) struct Vect {
    x: f64,
    y: f64,
    z: LongDouble,
}

/// `s14sqr(s)`: the square of a coordinate rounded once to `long double`.
pub(super) fn s14sqr(s: f64) -> LongDouble {
    LongDouble::new(s) * LongDouble::new(s)
}

impl Vect {
    /// `Vect::cross(v)`, operation for operation.
    pub(super) fn cross(&self, v: &Vect) -> Norm {
        let (x, y) = (LongDouble::new(self.x), LongDouble::new(self.y));
        let (vx, vy) = (LongDouble::new(v.x), LongDouble::new(v.y));
        Norm {
            x: y * v.z - self.z * vy,
            y: self.z * vx - x * v.z,
            z: x * vy - y * vx,
        }
    }
}

impl Norm {
    /// `(f.n.x > 0 && o.x > 0) || (f.n.x < 0 && o.x < 0) || ...` over the 3 components:
    /// whether any component of `other` has the strict sign of this normal's.
    pub(super) fn agrees_with(&self, other: &Norm) -> bool {
        let pairs = [(self.x, other.x), (self.y, other.y), (self.z, other.z)];
        pairs.iter().any(|&(mine, theirs)| {
            let zero = LongDouble::ZERO;
            (mine > zero && theirs > zero) || (mine < zero && theirs < zero)
        })
    }
}

/// `Vert::operator-(v)`: `a - b`.
pub(super) fn vert_diff(a: &DelaBellaVertex, b: &DelaBellaVertex) -> Vect {
    Vect {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
}

/// `Vert::u28cmp`: z, then y, then x, then the original index.
pub(super) fn u28cmp(a: &DelaBellaVertex, b: &DelaBellaVertex) -> Ordering {
    // Coordinates are finite (checked on input), so every partial comparison is total.
    let by_z = a.z.partial_cmp(&b.z).unwrap_or(Ordering::Equal);
    let by_y = a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal);
    let by_x = a.x.partial_cmp(&b.x).unwrap_or(Ordering::Equal);
    by_z.then(by_y).then(by_x).then(a.i.cmp(&b.i))
}

/// `Vert::overlap(v1, v2)`: same x and y.
pub(super) fn overlap(a: &DelaBellaVertex, b: &DelaBellaVertex) -> bool {
    a.x == b.x && a.y == b.y
}

impl DelaBella {
    /// `Face::cross()` of the face spanned by `v`: `(v1 - v0).cross(v2 - v0)`.
    pub(super) fn cross_of(&self, v: [usize; 3]) -> Norm {
        let origin = &self.verts[v[0]];
        let first = vert_diff(&self.verts[v[1]], origin);
        first.cross(&vert_diff(&self.verts[v[2]], origin))
    }

    /// `Face::dot(p)` for a face with normal `n` and first vertex `origin`:
    /// `n.x * d.x + n.y * d.y + n.z * d.z` with `d = p - origin`.
    pub(super) fn plane_dot(&self, n: &Norm, origin: usize, p: usize) -> LongDouble {
        let d = vert_diff(&self.verts[p], &self.verts[origin]);
        n.x * LongDouble::new(d.x) + n.y * LongDouble::new(d.y) + n.z * d.z
    }

    /// `Face::dot(p)` of the stored face `face`.
    pub(super) fn face_dot(&self, face: usize, p: usize) -> LongDouble {
        let stored = &self.faces[face];
        self.plane_dot(&stored.n, stored.v[0], p)
    }
}
