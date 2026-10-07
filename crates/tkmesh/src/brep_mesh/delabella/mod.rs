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

// Rust translation of delabella.pxx and delabella.cpp (the copy OCCT bundles in
// src/ModelingAlgorithms/TKMesh/BRepMesh at 3d097a0328e71b826377d4814ab05ec3c3d23871).

//! DELABELLA: the Delaunay triangulation of a 2D point cloud as the lower convex hull of
//! the points lifted onto the paraboloid `z = x² + y²` (the library behind
//! `BRepMesh_DelabellaBaseMeshAlgo`).
//!
//! The native pointers become indices: a triangle's `v` holds indices into the
//! (sorted) vertex array read through [`DelaBella::vertex`], its `f` and `next` indices
//! into the triangle array read through [`DelaBella::triangle`]. The z coordinates and
//! every cross/dot product are `long double` natively; they are computed in
//! [`LongDouble`] (the x87 80-bit format of the pinned GCC x86-64 profile) so every
//! sign decision, and so the triangulation, is bit-identical.
mod arith;
mod cone;
mod insert;
mod output;
mod seed;

pub use output::dela_bella;

use arith::Norm;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math_poly::LongDouble;

/// `DelaBella_Vertex` (with the private `CDelaBella::Vert` fields): one input point.
#[derive(Clone, Copy, Debug)]
pub struct DelaBellaVertex {
    /// Index of the original point.
    pub i: i32,
    /// X coordinate (input copy).
    pub x: f64,
    /// Y coordinate (input copy).
    pub y: f64,
    /// Next silhouette vertex (an index into the vertex array).
    pub next: Option<usize>,
    z: LongDouble,
    sew: Option<usize>,
}

/// `DelaBella_Triangle` (with the private `CDelaBella::Face` normal): one hull face.
#[derive(Clone, Copy, Debug)]
pub struct DelaBellaTriangle {
    /// The 3 vertices spanning this triangle (indices into the vertex array).
    pub v: [usize; 3],
    /// The 3 adjacent faces; `f[i]` is at the edge opposite to vertex `v[i]`.
    pub f: [Option<usize>; 3],
    /// Next triangle of the Delaunay set or of the hull set.
    pub next: Option<usize>,
    n: Norm,
}

/// The `SetErrLog` callback: receives each formatted `[ERR]`/`[WRN]`/`[NFO]` message.
pub type DelaBellaErrLog = Box<dyn FnMut(&str) + Send>;

/// `IDelaBella` / `CDelaBella`: the triangulator object.
///
/// ```
/// use rustycad_tkmesh::brep_mesh::delabella::DelaBella;
/// let mut triangulator = DelaBella::create();
/// let xy = [0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 3.0, 2.0];
/// assert_eq!(triangulator.triangulate_f64(4, &xy, None, 0)?, 6);
/// assert!(triangulator.get_first_delaunay_triangle().is_some());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct DelaBella {
    verts: Vec<DelaBellaVertex>,
    faces: Vec<DelaBellaTriangle>,
    first_dela_face: Option<usize>,
    first_hull_face: Option<usize>,
    first_hull_vert: Option<usize>,
    inp_verts: i32,
    out_verts: i32,
    errlog: Option<DelaBellaErrLog>,
}

impl DelaBella {
    /// `IDelaBella::Create()`: an empty triangulator without a log;
    /// `DelaBella::create()`.
    pub fn create() -> Self {
        Self::default()
    }

    /// `SetErrLog(proc, stream)`: the callback of the diagnostics (`None` silences them);
    /// `triangulator.set_err_log(Some(Box::new(|text| eprint!("{text}"))))`.
    pub fn set_err_log(&mut self, errlog: Option<DelaBellaErrLog>) {
        self.errlog = errlog;
    }

    /// `Triangulate(points, const double* x, const double* y, advance_bytes)`: `x[k *
    /// advance]` and `y[k * advance]` (y defaults to `&x[1..]`, an advance below 2 is 2)
    /// are point k. Returns 3 × the Delaunay triangle count, or -(hull vertex count)
    /// when all points are colinear; `triangulator.triangulate_f64(3, &xy, None, 0)?`.
    ///
    /// # Errors
    /// `Standard_OutOfRange` when a slice is too short, `Standard_DomainError` for a
    /// non-finite coordinate or a point count below 1 (undefined natively), and
    /// `Standard_ProgramError` when the hull links break (a native crash).
    pub fn triangulate_f64(
        &mut self,
        points: i32,
        x: &[f64],
        y: Option<&[f64]>,
        advance: usize,
    ) -> Result<i32, Failure> {
        let coordinates = strided_coordinates(points, x, y, advance, |value| value)?;
        self.triangulate_points(&coordinates)
    }

    /// `Triangulate(points, const float* x, const float* y, advance_bytes)`: the float
    /// overload, each coordinate widened to double first;
    /// `triangulator.triangulate_f32(3, &xy, None, 0)?`.
    ///
    /// # Errors
    /// As [`DelaBella::triangulate_f64`].
    pub fn triangulate_f32(
        &mut self,
        points: i32,
        x: &[f32],
        y: Option<&[f32]>,
        advance: usize,
    ) -> Result<i32, Failure> {
        let coordinates = strided_coordinates(points, x, y, advance, f64::from)?;
        self.triangulate_points(&coordinates)
    }

    /// `GetNumInputPoints()`: the points of the last call; `triangulator.get_num_input_points()`.
    pub fn get_num_input_points(&self) -> i32 {
        self.inp_verts
    }

    /// `GetNumOutputVerts()`: the result of the last call; `triangulator.get_num_output_verts()`.
    pub fn get_num_output_verts(&self) -> i32 {
        self.out_verts
    }

    /// `GetFirstDelaunayTriangle()` (valid when the last result is positive);
    /// `triangulator.get_first_delaunay_triangle()`.
    pub fn get_first_delaunay_triangle(&self) -> Option<usize> {
        self.first_dela_face
    }

    /// `GetFirstHullTriangle()` (valid when the last result is positive);
    /// `triangulator.get_first_hull_triangle()`.
    pub fn get_first_hull_triangle(&self) -> Option<usize> {
        self.first_hull_face
    }

    /// `GetFirstHullVertex()`: a list when the last result is negative, else a closed
    /// contour; `triangulator.get_first_hull_vertex()`.
    pub fn get_first_hull_vertex(&self) -> Option<usize> {
        self.first_hull_vert
    }

    /// The vertex behind an index of `v`, `next` or a `GetFirst*` call;
    /// `triangulator.vertex(0).map(|vertex| vertex.i)`.
    pub fn vertex(&self, index: usize) -> Option<&DelaBellaVertex> {
        self.verts.get(index)
    }

    /// The triangle behind an index of `f`, `next` or a `GetFirst*` call;
    /// `triangulator.triangle(0).map(|triangle| triangle.v)`.
    pub fn triangle(&self, index: usize) -> Option<&DelaBellaTriangle> {
        self.faces.get(index)
    }
}

impl DelaBella {
    /// The shared tail of both overloads: `ReallocVerts`, the lifted copy of each
    /// point, then the triangulation.
    fn triangulate_points(&mut self, coordinates: &[(f64, f64)]) -> Result<i32, Failure> {
        self.inp_verts = coordinates.len() as i32;
        self.out_verts = 0;
        self.first_dela_face = None;
        self.first_hull_face = None;
        self.first_hull_vert = None;
        self.verts = coordinates
            .iter()
            .enumerate()
            .map(|(i, &(x, y))| lifted_vertex(i as i32, x, y))
            .collect();
        self.out_verts = self.triangulate()?;
        Ok(self.out_verts)
    }

    /// Sends one diagnostic to the log callback, when there is one.
    fn log(&mut self, text: &str) {
        if let Some(errlog) = self.errlog.as_mut() {
            errlog(text);
        }
    }
}

/// A vertex with `z = s14sqr(x) + s14sqr(y)`, no silhouette links.
fn lifted_vertex(i: i32, x: f64, y: f64) -> DelaBellaVertex {
    DelaBellaVertex {
        i,
        x,
        y,
        next: None,
        z: arith::s14sqr(x) + arith::s14sqr(y),
        sew: None,
    }
}

/// The `(x, y)` pairs read with the native pointer/advance conventions.
fn strided_coordinates<T: Copy>(
    points: i32,
    x: &[T],
    y: Option<&[T]>,
    advance: usize,
    widen: impl Fn(T) -> f64,
) -> Result<Vec<(f64, f64)>, Failure> {
    if points < 1 {
        return Err(domain_error(&format!(
            "DelaBella::Triangulate: {points} points, expected at least 1"
        )));
    }
    let y = y.unwrap_or_else(|| x.get(1..).unwrap_or(&[]));
    let advance = advance.max(2);
    (0..points as usize)
        .map(|k| strided_point(x, y, k * advance, &widen))
        .collect()
}

/// Point `k` at element `offset`, refusing short slices and non-finite values.
fn strided_point<T: Copy>(
    x: &[T],
    y: &[T],
    offset: usize,
    widen: &impl Fn(T) -> f64,
) -> Result<(f64, f64), Failure> {
    let (Some(&px), Some(&py)) = (x.get(offset), y.get(offset)) else {
        return Err(Failure::with_message(
            "Standard_OutOfRange",
            &format!(
                "DelaBella::Triangulate: element {offset} outside x[{}] or y[{}]",
                x.len(),
                y.len()
            ),
        ));
    };
    let (px, py) = (widen(px), widen(py));
    if !px.is_finite() || !py.is_finite() {
        return Err(domain_error(&format!(
            "DelaBella::Triangulate: point ({px}, {py}) at element {offset}, expected finite coordinates"
        )));
    }
    Ok((px, py))
}

fn domain_error(message: &str) -> Failure {
    Failure::with_message("Standard_DomainError", message)
}

/// A broken hull link: the native code dereferences a null or stray pointer here.
fn broken_hull(what: &str, index: usize) -> Failure {
    Failure::with_message(
        "Standard_ProgramError",
        &format!("DelaBella::Triangulate: {what} at index {index}, expected a linked hull"),
    )
}

#[cfg(test)]
mod tests;
