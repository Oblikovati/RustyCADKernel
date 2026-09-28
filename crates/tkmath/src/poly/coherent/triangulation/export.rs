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
// Rust translation of GetTriangulation, GetFreeNodes, Clone and Dump of
// FoundationClasses/TKMath/Poly/Poly_CoherentTriangulation.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::CoherentTriangulation;
use crate::{
    gp::{Direction3, Point2, Point3},
    poly::{Triangle, Triangulation, coherent::CoherentNode},
};
use rustycad_tkernel::{
    ncollection::List,
    precision,
    standard::{Failure, Handle},
};
use std::io::{self, Write};
impl CoherentTriangulation {
    /// Exports used nodes (renumbered from 1) and live triangles. A node gets a normal when
    /// its normal is not null and UV when `u² + v² > confusion`, so the default infinite UV
    /// is exported too, as natively; a null handle when empty; `mesh.get_triangulation()?`.
    pub fn get_triangulation(&self) -> Result<Handle<Triangulation>, Failure> {
        let (nb_nodes, nb_triangles) = (self.n_nodes(), self.n_triangles());
        if nb_nodes == 0 || nb_triangles == 0 {
            return Ok(Handle::default());
        }
        let result = Triangulation::with_sizes(nb_nodes, nb_triangles, false, false)?;
        let mut renumbered = vec![0; self.nodes.len()];
        let mut count = 0;
        for (slot, node) in self.nodes.iter().enumerate() {
            if node.is_free_node() {
                continue;
            }
            count += 1;
            renumbered[slot] = count;
            export_node(&result, count, node)?;
        }
        let live = self.triangles.iter().filter(|t| !t.is_empty());
        for (offset, triangle) in live.enumerate() {
            let [a, b, c] = triangle.nodes.map(|n| renumbered[n as usize]);
            result.set_triangle(offset as i32 + 1, Triangle::new(a, b, c))?;
        }
        result.set_deflection(self.deflection());
        Ok(Handle::new(result))
    }
    /// Collects the indices of nodes without triangles; true when there is one;
    /// `mesh.get_free_nodes(&mut list)`.
    pub fn get_free_nodes(&self, nodes: &mut List<i32>) -> bool {
        nodes.clear();
        for (slot, node) in self.nodes.iter().enumerate() {
            if node.is_free_node() {
                nodes.append(slot as i32);
            }
        }
        !nodes.is_empty()
    }
    /// Rebuilds a copy through [`CoherentTriangulation::get_triangulation`], so free nodes
    /// and removed triangles are dropped and links are not copied; `None` where OCCT returns
    /// a null handle (no triangle or node); `mesh.clone_triangulation()?`.
    pub fn clone_triangulation(&self) -> Result<Option<Self>, Failure> {
        if self.n_triangles() == 0 || self.n_nodes() == 0 {
            return Ok(None);
        }
        let exported = self.get_triangulation()?;
        let mut copy = Self::from_triangulation(&exported)?;
        copy.set_deflection(exported.deflection());
        Ok(Some(copy))
    }
    /// Prints each used node and the nodes of its triangles; `mesh.dump(&mut out)?`.
    pub fn dump(&self, output: &mut impl Write) -> io::Result<()> {
        for (slot, node) in self.iter_nodes() {
            write!(output, "Node {slot}")?;
            node.dump(output, &self.triangles)?;
        }
        Ok(())
    }
}
fn export_node(result: &Triangulation, index: i32, node: &CoherentNode) -> Result<(), Failure> {
    let normal = node.get_normal();
    if normal.square_modulus() > precision::confusion() {
        result.add_normals()?;
        result.set_normal(index, Direction3::from_xyz(normal)?)?;
    }
    result.set_node(index, Point3::new(node.x(), node.y(), node.z()))?;
    let (u, v) = (node.get_u(), node.get_v());
    if u * u + v * v > precision::confusion() {
        result.add_uv_nodes()?;
        result.set_uv_node(index, Point2::new(u, v))?;
    }
    Ok(())
}
