// Created on: 2007-11-24
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
// Rust translation of FoundationClasses/TKMath/Poly/Poly_CoherentNode.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{CoherentTriPtr, CoherentTriangle};
use crate::gp::Xyz;
use rustycad_tkernel::precision;
use std::io::{self, Write};
/// `Poly_CoherentNode`: a point (it derefs to `gp_XYZ`), UV, normal, original index and the
/// ring of incident triangles; `CoherentNode::new(Xyz::new(0.0, 0.0, 0.0))`.
#[derive(Clone, Debug)]
pub struct CoherentNode {
    point: Xyz,
    uv: [f64; 2],
    triangles: Option<CoherentTriPtr>,
    index: i32,
    normal: [f32; 3],
}
impl Default for CoherentNode {
    // The native default constructor leaves the normal uninitialized; it is zero here,
    // as in the point constructor.
    fn default() -> Self {
        Self::new(Xyz::new(0., 0., 0.))
    }
}
impl std::ops::Deref for CoherentNode {
    type Target = Xyz;
    fn deref(&self) -> &Xyz {
        &self.point
    }
}
impl std::ops::DerefMut for CoherentNode {
    fn deref_mut(&mut self) -> &mut Xyz {
        &mut self.point
    }
}
impl CoherentNode {
    /// A free node at a point, with infinite UV and a null normal; `CoherentNode::new(point)`.
    pub fn new(point: Xyz) -> Self {
        Self {
            point,
            uv: [precision::infinite(); 2],
            triangles: None,
            index: -1,
            normal: [0.0; 3],
        }
    }
    /// Sets the UV coordinates; `node.set_uv(0.5, 0.25)`.
    pub const fn set_uv(&mut self, u: f64, v: f64) {
        self.uv = [u, v];
    }
    /// U coordinate; `node.get_u()`.
    pub const fn get_u(&self) -> f64 {
        self.uv[0]
    }
    /// V coordinate; `node.get_v()`.
    pub const fn get_v(&self) -> f64 {
        self.uv[1]
    }
    /// Stores the normal narrowed to single precision; `node.set_normal(Xyz::new(0.0, 0.0, 1.0))`.
    pub fn set_normal(&mut self, vector: Xyz) {
        self.normal = [vector.x() as f32, vector.y() as f32, vector.z() as f32];
    }
    /// True when the single-precision squared length exceeds the confusion; `node.has_normal()`.
    pub fn has_normal(&self) -> bool {
        let [x, y, z] = self.normal;
        f64::from(x * x + y * y + z * z) > precision::confusion()
    }
    /// The stored normal widened to double; `node.get_normal()`.
    pub fn get_normal(&self) -> Xyz {
        let [x, y, z] = self.normal.map(f64::from);
        Xyz::new(x, y, z)
    }
    /// Sets the original index; `node.set_index(3)`.
    pub const fn set_index(&mut self, index: i32) {
        self.index = index;
    }
    /// The original index, -1 by default; `node.get_index()`.
    pub const fn get_index(&self) -> i32 {
        self.index
    }
    /// True without incident triangles; `node.is_free_node()`.
    pub const fn is_free_node(&self) -> bool {
        self.triangles.is_none()
    }
    /// Drops the triangles and resets point, UV and normal, keeping the index; `node.clear()`.
    pub fn clear(&mut self) {
        self.triangles = None;
        self.uv = [precision::infinite(); 2];
        self.normal = [0.0; 3];
        self.point = Xyz::new(0., 0., 0.);
    }
    /// Adds a triangle last in the ring, as `AddTriangle`; `node.add_triangle(4)`.
    pub fn add_triangle(&mut self, triangle: usize) {
        match &mut self.triangles {
            None => self.triangles = Some(CoherentTriPtr::new(triangle)),
            Some(ring) => ring.prepend(0, triangle),
        }
    }
    /// Removes the head or the first later occurrence of a triangle; a free node reports
    /// false (natively a null dereference); `node.remove_triangle(4)`.
    pub fn remove_triangle(&mut self, triangle: usize) -> bool {
        let Some(ring) = &mut self.triangles else {
            return false;
        };
        let found = if ring.get_triangle() == triangle {
            Some(0)
        } else {
            ring.iter()
                .skip(1)
                .position(|t| t == triangle)
                .map(|p| p + 1)
        };
        let Some(member) = found else {
            return false;
        };
        ring.remove(member);
        if ring.is_empty() {
            self.triangles = None;
        }
        true
    }
    /// The ring of incident triangles, `None` for a free node, as `TriangleIterator`;
    /// `node.triangle_iterator()`.
    pub const fn triangle_iterator(&self) -> Option<&CoherentTriPtr> {
        self.triangles.as_ref()
    }
    /// Head-first triangle indices, empty for a free node; `node.triangles().collect::<Vec<_>>()`.
    pub fn triangles(&self) -> impl Iterator<Item = usize> + '_ {
        self.triangles.iter().flat_map(CoherentTriPtr::iter)
    }
    /// Prints the point and the nodes of each incident triangle; `node.dump(&mut out, &triangles)?`.
    pub fn dump(&self, output: &mut impl Write, triangles: &[CoherentTriangle]) -> io::Result<()> {
        let (x, y, z) = (self.x(), self.y(), self.z());
        writeln!(
            output,
            "  X ={}; Y ={}; Z ={}",
            fixed4(x),
            fixed4(y),
            fixed4(z)
        )?;
        for triangle in self.triangles().filter_map(|t| triangles.get(t)) {
            let [a, b, c] = [0, 1, 2].map(|i| triangle.node(i));
            writeln!(output, "      {a:>5} {b:>5} {c:>5}")?;
        }
        Ok(())
    }
}
/// `printf("%9.4f")` in the C locale; `fixed4(1.0) == "   1.0000"`.
fn fixed4(value: f64) -> String {
    let text = if value.is_nan() {
        if value.is_sign_negative() {
            "-nan"
        } else {
            "nan"
        }
        .to_owned()
    } else {
        format!("{value:.4}")
    };
    format!("{text:>9}")
}
