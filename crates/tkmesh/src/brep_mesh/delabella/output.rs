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

// Rust translation of the end of CDelaBella::Triangulate() of delabella.cpp (the
// Delaunay/hull face lists and the hull silhouette) and of the deprecated DelaBella().

//! The output lists of a full triangulation, and the deprecated one-call interface.
use super::{DelaBella, broken_hull};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math_poly::LongDouble;

/// Where the next face of a list is linked (`Face** prev_dela`): the list head or the
/// `next` of the previous face.
#[derive(Clone, Copy, Debug)]
enum ListTail {
    DelaHead,
    HullHead,
    Face(usize),
}

impl DelaBella {
    /// Faces with a downward normal form the Delaunay list, the others the hull list
    /// whose silhouette edges link the hull vertices; returns `3 × Delaunay faces`.
    pub(super) fn link_output_lists(&mut self, points: usize) -> Result<i32, Failure> {
        for vert in &mut self.verts[..points] {
            vert.next = None;
            vert.sew = None;
        }
        let count = self.split_face_lists()?;
        let first_hull = self
            .first_hull_face
            .ok_or_else(|| broken_hull("hull without upward face", self.faces.len()))?;
        self.first_hull_vert = Some(self.faces[first_hull].v[0]);
        Ok(3 * count)
    }

    /// Links the faces into the Delaunay and hull lists in index order; returns the
    /// Delaunay face count.
    fn split_face_lists(&mut self) -> Result<i32, Failure> {
        let (mut dela_tail, mut hull_tail) = (ListTail::DelaHead, ListTail::HullHead);
        let mut count = 0;
        for face in 0..self.faces.len() {
            if self.faces[face].n.z < LongDouble::ZERO {
                self.set_list_link(dela_tail, Some(face));
                dela_tail = ListTail::Face(face);
                count += 1;
            } else {
                self.set_list_link(hull_tail, Some(face));
                hull_tail = ListTail::Face(face);
                self.link_silhouette(face)?;
            }
        }
        self.set_list_link(dela_tail, None);
        self.set_list_link(hull_tail, None);
        Ok(count)
    }

    fn set_list_link(&mut self, tail: ListTail, face: Option<usize>) {
        match tail {
            ListTail::DelaHead => self.first_dela_face = face,
            ListTail::HullHead => self.first_hull_face = face,
            ListTail::Face(previous) => self.faces[previous].next = face,
        }
    }

    /// For each edge a hull face shares with a Delaunay face, `v[a].next = v[b]` and
    /// `v[a].sew = face` (edge slots `f[0]: v1->v2`, `f[1]: v2->v0`, `f[2]: v0->v1`).
    fn link_silhouette(&mut self, face: usize) -> Result<(), Failure> {
        const EDGES: [(usize, usize); 3] = [(1, 2), (2, 0), (0, 1)];
        for (slot, (a, b)) in EDGES.into_iter().enumerate() {
            let neighbour = self.faces[face].f[slot]
                .ok_or_else(|| broken_hull("hull face without neighbour", face))?;
            if self.faces[neighbour].n.z < LongDouble::ZERO {
                let [from, to] = [self.faces[face].v[a], self.faces[face].v[b]];
                self.verts[from].next = Some(to);
                self.verts[from].sew = Some(face);
            }
        }
        Ok(())
    }
}

/// The deprecated `DelaBella(points, xy, abc, errlog)`: triangulates interleaved `xy`
/// and, when `abc` is given, writes the original indices of each Delaunay triangle
/// (positive result) or of the colinear hull list (negative result). The log only
/// receives the deprecation warning, as natively.
///
/// ```
/// use rustycad_tkmesh::brep_mesh::delabella::dela_bella;
/// let mut abc = [0; 6];
/// let xy = [0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 3.0, 2.0];
/// assert_eq!(dela_bella(4, &xy, Some(&mut abc), None)?, 6);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
///
/// # Errors
/// As [`DelaBella::triangulate_f64`], and `Standard_OutOfRange` when `abc` is too short.
pub fn dela_bella(
    points: i32,
    xy: &[f64],
    abc: Option<&mut [i32]>,
    errlog: Option<&mut dyn FnMut(&str)>,
) -> Result<i32, Failure> {
    if let Some(errlog) = errlog {
        errlog("[WRN] Depreciated interface! errlog disabled.\n");
    }
    if xy.is_empty() || points <= 0 {
        return Ok(0);
    }
    let mut triangulator = DelaBella::create();
    let verts = triangulator.triangulate_f64(points, xy, None, 0)?;
    let Some(abc) = abc else {
        return Ok(verts);
    };
    let indices = if verts > 0 {
        triangulator.delaunay_indices(verts / 3)?
    } else {
        triangulator.hull_list_indices(-verts)?
    };
    write_indices(abc, &indices)?;
    Ok(verts)
}

/// Copies the indices to the front of `abc`, refusing a short buffer.
fn write_indices(abc: &mut [i32], indices: &[i32]) -> Result<(), Failure> {
    let capacity = abc.len();
    let target = abc.get_mut(..indices.len()).ok_or_else(|| {
        Failure::with_message(
            "Standard_OutOfRange",
            &format!(
                "DelaBella: abc[{capacity}] cannot hold {} indices",
                indices.len()
            ),
        )
    })?;
    target.copy_from_slice(indices);
    Ok(())
}

impl DelaBella {
    /// The original indices of the first `tris` Delaunay triangles.
    fn delaunay_indices(&self, tris: i32) -> Result<Vec<i32>, Failure> {
        let mut indices = Vec::with_capacity(3 * tris.max(0) as usize);
        let mut dela = self.first_dela_face;
        for _ in 0..tris {
            let face = dela.ok_or_else(|| broken_hull("short Delaunay list", indices.len()))?;
            indices.extend(self.faces[face].v.map(|vert| self.verts[vert].i));
            dela = self.faces[face].next;
        }
        Ok(indices)
    }

    /// The original indices of the first `pnts` hull-list vertices.
    fn hull_list_indices(&self, pnts: i32) -> Result<Vec<i32>, Failure> {
        let mut indices = Vec::with_capacity(pnts.max(0) as usize);
        let mut line = self.first_hull_vert;
        for _ in 0..pnts {
            let vert = line.ok_or_else(|| broken_hull("short hull list", indices.len()))?;
            indices.push(self.verts[vert].i);
            line = self.verts[vert].next;
        }
        Ok(indices)
    }
}
