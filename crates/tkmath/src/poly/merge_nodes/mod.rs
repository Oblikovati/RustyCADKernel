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
// Rust translation of FoundationClasses/TKMath/Poly/Poly_MergeNodesTool.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Poly_MergeNodesTool`: merges coincident mesh nodes, keeping sharp edges split.
mod map;
mod push;
use super::Triangulation;
use crate::gp::{Transform3, Xyz};
use map::MergedNodesMap;
use rustycad_tkernel::{
    ncollection::Vec3,
    standard::{Failure, Handle, StandardTransient},
};
use std::collections::HashSet;
/// Node merging tool writing into an output triangulation; `MergeNodesTool::new(0.5, 0.0, -1)`.
pub struct MergeNodesTool {
    base: StandardTransient,
    output: Handle<Triangulation>,
    node_map: MergedNodesMap,
    // Only membership matters for NCollection_Map::Add, so its bucket order is not kept.
    elements: HashSet<[i32; 4]>,
    node_indices: [i32; 4],
    triangle_normal: Vec3<f32>,
    places: [Xyz; 4],
    unit_factor: f64,
    nb_nodes: i32,
    nb_elements: i32,
    nb_degenerate: i32,
    nb_merged: i32,
    drop_degenerate: bool,
    merge_elements: bool,
}
rustycad_tkernel::impl_transient!(
    MergeNodesTool,
    "Poly_MergeNodesTool",
    StandardTransient,
    base
);
impl MergeNodesTool {
    /// Merges a whole mesh; a null handle for an input under 3 nodes or without triangles,
    /// or, unless `force`, when nothing merged; `MergeNodesTool::merge_nodes(&mesh, t, false, 0.5, 0.0, true)?`.
    pub fn merge_nodes(
        triangulation: &Handle<Triangulation>,
        transform: Transform3,
        reverse: bool,
        smooth_angle: f64,
        merge_tolerance: f64,
        force: bool,
    ) -> Result<Handle<Triangulation>, Failure> {
        let Some(mesh) = triangulation.get() else {
            return Ok(Handle::default());
        };
        if mesh.nb_nodes() < 3 || mesh.nb_triangles() < 1 {
            return Ok(Handle::default());
        }
        let mut tool = Self::new(smooth_angle, merge_tolerance, mesh.nb_triangles());
        tool.add_triangulation(triangulation, transform, reverse)?;
        let unchanged = tool.nb_nodes == mesh.nb_nodes() && tool.nb_elements == mesh.nb_triangles();
        if !force && unchanged {
            return Ok(Handle::default());
        }
        tool.result()
    }
    /// A tool with an angle (radians, 0 disables) and a distance tolerance; the facet
    /// estimate sizes the node map; `MergeNodesTool::new(0.5, 0.0, -1)`.
    pub fn new(smooth_angle: f64, merge_tolerance: f64, nb_facets: i32) -> Self {
        let merging = smooth_angle > 0.0 || merge_tolerance > 0.0;
        let buckets = if !merging {
            1
        } else if nb_facets > 0 {
            // A ratio of one triangle to two merged nodes is expected.
            nb_facets.wrapping_mul(2) as usize
        } else {
            995329
        };
        let mut tool = Self {
            base: StandardTransient::default(),
            output: Handle::new(Triangulation::new()),
            node_map: MergedNodesMap::new(buckets),
            elements: HashSet::new(),
            node_indices: [0, 0, 0, -1],
            triangle_normal: Vec3::new(0.0, 0.0, 1.0),
            places: [Xyz::new(0., 0., 0.); 4],
            unit_factor: 1.0,
            nb_nodes: 0,
            nb_elements: 0,
            nb_degenerate: 0,
            nb_merged: 0,
            drop_degenerate: true,
            merge_elements: false,
        };
        tool.set_merge_angle(smooth_angle);
        tool.set_merge_tolerance(merge_tolerance);
        tool
    }
    /// Merge distance; 0 merges only equal points; `tool.merge_tolerance()`.
    pub fn merge_tolerance(&self) -> f64 {
        self.node_map.merge_tolerance()
    }
    /// Sets the merge distance, stored in single precision; `tool.set_merge_tolerance(1e-6)`.
    pub fn set_merge_tolerance(&mut self, tolerance: f64) {
        self.node_map.set_merge_tolerance(tolerance);
    }
    /// Merge angle in radians, stored in single precision; `tool.merge_angle()`.
    pub fn merge_angle(&self) -> f64 {
        self.node_map.merge_angle()
    }
    /// Sets the merge angle; `tool.set_merge_angle(0.5)`.
    pub fn set_merge_angle(&mut self, angle: f64) {
        self.node_map.set_merge_angle(angle);
    }
    /// True when nodes with opposite normals merge; `tool.to_merge_opposite()`.
    pub const fn to_merge_opposite(&self) -> bool {
        self.node_map.to_merge_opposite()
    }
    /// Lets nodes with opposite normals merge; `tool.set_merge_opposite(true)`.
    pub const fn set_merge_opposite(&mut self, merge: bool) {
        self.node_map.set_merge_opposite(merge);
    }
    /// Scale applied to output node coordinates; `tool.set_unit_factor(0.001)`.
    pub const fn set_unit_factor(&mut self, factor: f64) {
        self.unit_factor = factor;
    }
    /// True when degenerate triangles are dropped (default); `tool.to_drop_degenerative()`.
    pub const fn to_drop_degenerative(&self) -> bool {
        self.drop_degenerate
    }
    /// Chooses whether degenerate triangles are dropped; `tool.set_drop_degenerative(false)`.
    pub const fn set_drop_degenerative(&mut self, drop: bool) {
        self.drop_degenerate = drop;
    }
    /// True when repeated elements are filtered; `tool.to_merge_elems()`.
    pub const fn to_merge_elems(&self) -> bool {
        self.merge_elements
    }
    /// Chooses whether repeated elements are filtered; `tool.set_merge_elems(true)`.
    pub const fn set_merge_elems(&mut self, merge: bool) {
        self.merge_elements = merge;
    }
    /// Unit normal of the first three element nodes in single precision, as `computeTriNormal`;
    /// `tool.compute_tri_normal()`.
    pub fn compute_tri_normal(&self) -> Vec3<f32> {
        let cross = (self.places[1] - self.places[0]).crossed(self.places[2] - self.places[0]);
        Vec3::new(cross.x() as f32, cross.y() as f32, cross.z() as f32).normalized()
    }
    /// Element node slot 0..3 for `push_last_element`, as `ChangeElementNode`; `*tool.change_element_node(0) = p`.
    pub fn change_element_node(&mut self, index: usize) -> &mut Xyz {
        &mut self.places[index]
    }
    /// Output node index (0-based) of an element node, as `ElementNodeIndex`; `tool.element_node_index(0)`.
    pub const fn element_node_index(&self, index: usize) -> i32 {
        self.node_indices[index]
    }
    /// Number of output nodes; `tool.nb_nodes()`.
    pub const fn nb_nodes(&self) -> i32 {
        self.nb_nodes
    }
    /// Number of output elements (a quad counts twice); `tool.nb_elements()`.
    pub const fn nb_elements(&self) -> i32 {
        self.nb_elements
    }
    /// Number of dropped degenerate elements; `tool.nb_degenerative_elems()`.
    pub const fn nb_degenerative_elems(&self) -> i32 {
        self.nb_degenerate
    }
    /// Number of filtered repeated elements; `tool.nb_merged_elems()`.
    pub const fn nb_merged_elems(&self) -> i32 {
        self.nb_merged
    }
    /// The output triangulation; a null handle turns the tool into a pure merge map, as
    /// `ChangeOutput`; `*tool.change_output() = Handle::default()`.
    pub fn change_output(&mut self) -> &mut Handle<Triangulation> {
        &mut self.output
    }
    /// Shrinks the output to the merged size and returns it, as `Result`; `tool.result()?`.
    pub fn result(&mut self) -> Result<Handle<Triangulation>, Failure> {
        let Some(output) = self.output.get() else {
            return Ok(Handle::default());
        };
        output.resize_nodes(self.nb_nodes, true)?;
        output.resize_triangles(self.nb_elements, true)?;
        Ok(self.output.clone())
    }
}
