// Copyright (c) 2015-2021 OPEN CASCADE SAS
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
// Rust translation of the element-pushing members of FoundationClasses/TKMath/Poly/Poly_MergeNodesTool.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::MergeNodesTool;
use crate::{
    gp::{Point3, Transform3, Xyz},
    poly::{Triangle, Triangulation},
};
use rustycad_tkernel::{
    ncollection::Vec3,
    standard::{Failure, Handle},
};
fn internal_error(member: &str, nb_nodes: i32) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!(
            "Poly_MergeNodesTool::{member}() - Internal error: {nb_nodes} element nodes; expected 3 or 4"
        ),
        "",
    )
}
impl MergeNodesTool {
    /// Adds every triangle of a mesh, transformed and optionally reversed; an empty output
    /// is first sized to the input; `tool.add_triangulation(&mesh, Transform3::default(), false)?`.
    pub fn add_triangulation(
        &mut self,
        triangulation: &Handle<Triangulation>,
        transform: Transform3,
        reverse: bool,
    ) -> Result<(), Failure> {
        let Some(mesh) = triangulation.get() else {
            return Ok(());
        };
        if let Some(output) = self.output.get().filter(|output| output.nb_nodes() == 0) {
            output.set_double_precision(mesh.is_double_precision())?;
            output.resize_nodes(mesh.nb_nodes(), false)?;
            output.resize_triangles(mesh.nb_triangles(), false)?;
        }
        for index in 1..=mesh.nb_triangles() {
            let triangle = mesh.triangle(index)?;
            let order = if reverse { [1, 3, 2] } else { [1, 2, 3] };
            for (slot, position) in order.into_iter().enumerate() {
                let node = mesh.node(triangle.value(position)?)?;
                self.places[slot] = node.transformed(transform).xyz();
            }
            self.push_last_element(3)?;
        }
        Ok(())
    }
    /// Adds a triangle, as `AddTriangle`; `tool.add_triangle(&[a, b, c])?`.
    pub fn add_triangle(&mut self, nodes: &[Xyz; 3]) -> Result<(), Failure> {
        self.add_element(nodes)
    }
    /// Adds a quad as two triangles, as `AddQuad`; `tool.add_quad(&[a, b, c, d])?`.
    pub fn add_quad(&mut self, nodes: &[Xyz; 4]) -> Result<(), Failure> {
        self.add_element(nodes)
    }
    /// Adds a triangle or quad; other sizes raise `Standard_ProgramError`; `tool.add_element(&nodes)?`.
    pub fn add_element(&mut self, nodes: &[Xyz]) -> Result<(), Failure> {
        let count = nodes.len() as i32;
        if count != 3 && count != 4 {
            return Err(internal_error("AddElement", count));
        }
        self.places[..nodes.len()].copy_from_slice(nodes);
        self.push_last_element(count)
    }
    /// Adds the triangle staged with `change_element_node`, as `PushLastTriangle`; `tool.push_last_triangle()?`.
    pub fn push_last_triangle(&mut self) -> Result<(), Failure> {
        self.push_last_element(3)
    }
    /// Adds the staged quad, as `PushLastQuad`; `tool.push_last_quad()?`.
    pub fn push_last_quad(&mut self) -> Result<(), Failure> {
        self.push_last_element(4)
    }
    /// Merges the staged nodes and appends the element unless it is degenerate or repeated;
    /// `tool.push_last_element(3)?`.
    pub fn push_last_element(&mut self, nb_nodes: i32) -> Result<(), Failure> {
        if nb_nodes != 3 && nb_nodes != 4 {
            return Err(internal_error("PushLastElement", nb_nodes));
        }
        self.node_indices[3] = -1;
        self.push_nodes(nb_nodes as usize)?;
        if self.drop_degenerate && nb_nodes == 3 && self.is_degenerate() {
            // A degenerate quad is kept natively; dropping may leave unused nodes.
            self.nb_degenerate += 1;
            return Ok(());
        }
        if self.merge_elements && !self.is_new_element(nb_nodes as usize) {
            self.nb_merged += 1;
            return Ok(());
        }
        self.nb_elements += 1;
        let shared = self.output.clone();
        let Some(output) = shared.get() else {
            // Without an output a quad counts as one element, as natively.
            return Ok(());
        };
        self.append_triangle(output, [0, 1, 2])?;
        if nb_nodes == 4 {
            self.nb_elements += 1;
            self.append_triangle(output, [0, 2, 3])?;
        }
        Ok(())
    }
    fn push_nodes(&mut self, count: usize) -> Result<(), Failure> {
        let merging = self.node_map.has_merge_angle() || self.node_map.has_merge_tolerance();
        if merging && !self.node_map.to_merge_any_angle() {
            self.triangle_normal = self.compute_tri_normal();
        }
        for slot in 0..count {
            if merging {
                self.push_node_check(slot)?;
            } else {
                self.push_node_no_merge(slot)?;
            }
        }
        Ok(())
    }
    fn push_node_check(&mut self, slot: usize) -> Result<(), Failure> {
        let place = self.places[slot];
        let position = Vec3::new(place.x() as f32, place.y() as f32, place.z() as f32);
        let binding = self
            .node_map
            .bind(self.nb_nodes, position, self.triangle_normal);
        if binding.is_new {
            self.store_node(place * self.unit_factor)?;
        }
        self.node_indices[slot] = binding.index;
        Ok(())
    }
    fn push_node_no_merge(&mut self, slot: usize) -> Result<(), Failure> {
        let index = self.nb_nodes;
        self.store_node(self.places[slot] * self.unit_factor)?;
        self.node_indices[slot] = index;
        Ok(())
    }
    // Counts the node and writes it, doubling the output when it is full.
    fn store_node(&mut self, place: Xyz) -> Result<(), Failure> {
        self.nb_nodes += 1;
        let Some(output) = self.output.get() else {
            return Ok(());
        };
        if output.nb_nodes() < self.nb_nodes {
            output.resize_nodes(self.nb_nodes * 2, true)?;
        }
        output.set_node(self.nb_nodes, Point3::new(place.x(), place.y(), place.z()))
    }
    fn is_degenerate(&self) -> bool {
        let [a, b, c, _] = self.node_indices;
        a == b || a == c || b == c
    }
    fn is_new_element(&mut self, count: usize) -> bool {
        let mut sorted = self.node_indices;
        sorted[..count].sort_unstable();
        self.elements.insert(sorted)
    }
    fn append_triangle(&self, output: &Triangulation, corners: [usize; 3]) -> Result<(), Failure> {
        if output.nb_triangles() < self.nb_elements {
            output.resize_triangles(self.nb_elements * 2, true)?;
        }
        let [a, b, c] = corners.map(|corner| self.node_indices[corner] + 1);
        output.set_triangle(self.nb_elements, Triangle::new(a, b, c))
    }
}
