// Rust translation of FoundationClasses/TKMath/Poly/Poly_CoherentTriangulation.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Poly_CoherentTriangulation`: an editable mesh with triangle-triangle, node-triangle
//! and optional link connectivity.
//!
//! Triangles, nodes and links live in growable arrays and are named by their index, where
//! the native API hands out references or pointers into its `NCollection_DynamicArray`s.
//! Removed triangles and links stay in place with negative nodes. The native allocator
//! is retained for incident triangle rings on every node.
mod edit;
mod export;
mod links;
use super::{CoherentLink, CoherentNode, CoherentTriangle};
use crate::{gp::Xyz, poly::Triangulation};
use rustycad_tkernel::{
    ncollection::BaseAllocator,
    standard::{Failure, Handle, StandardTransient},
};
/// A removed node and the node the mesh was reconnected to, as `TwoIntegers`;
/// `TwoIntegers::new(4, 2)`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TwoIntegers {
    /// Removed node, then the node that replaced it.
    pub value: [i32; 2],
}
impl TwoIntegers {
    /// Stores the pair; `TwoIntegers::new(4, 2)`.
    pub const fn new(first: i32, second: i32) -> Self {
        Self {
            value: [first, second],
        }
    }
}
/// `Poly_CoherentTriangulation`; `CoherentTriangulation::new()`.
#[derive(Default)]
pub struct CoherentTriangulation {
    base: StandardTransient,
    pub(super) triangles: Vec<CoherentTriangle>,
    pub(super) nodes: Vec<CoherentNode>,
    // Link slots and the used count: NCollection_DynamicArray::Clear keeps its blocks, so
    // a triangle that still refers to a cleared link reads (and writes) the old slot.
    pub(super) links: Vec<CoherentLink>,
    pub(super) link_count: usize,
    allocator: BaseAllocator,
    deflection: f64,
}
rustycad_tkernel::impl_transient!(
    CoherentTriangulation,
    "Poly_CoherentTriangulation",
    StandardTransient,
    base
);
impl CoherentTriangulation {
    /// An empty structure; `CoherentTriangulation::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Creates an empty structure whose incident rings use `allocator`;
    /// `CoherentTriangulation::new_in(arena)`.
    pub fn new_in(allocator: BaseAllocator) -> Self {
        Self {
            base: StandardTransient::default(),
            triangles: Vec::new(),
            nodes: Vec::new(),
            links: Vec::new(),
            link_count: 0,
            allocator,
            deflection: 0.0,
        }
    }
    /// Copies nodes, non-degenerate triangles, UV nodes, normals and deflection; links are
    /// not built. A null handle gives an empty structure (natively the deflection is then
    /// left uninitialized); `CoherentTriangulation::from_triangulation(&mesh)?`.
    pub fn from_triangulation(triangulation: &Handle<Triangulation>) -> Result<Self, Failure> {
        Self::from_triangulation_in(triangulation, BaseAllocator::common())
    }
    /// Copies a mesh using `allocator` for each incident triangle ring;
    /// `CoherentTriangulation::from_triangulation_in(&mesh, arena)?`.
    pub fn from_triangulation_in(
        triangulation: &Handle<Triangulation>,
        allocator: BaseAllocator,
    ) -> Result<Self, Failure> {
        let mut result = Self::new_in(allocator);
        let Some(mesh) = triangulation.get() else {
            return Ok(result);
        };
        result.copy_from_mesh(mesh)?;
        Ok(result)
    }
    fn copy_from_mesh(&mut self, mesh: &Triangulation) -> Result<(), Failure> {
        for offset in 0..mesh.nb_nodes() {
            let new_index = self.set_node(mesh.node(offset + 1)?.xyz(), offset);
            self.change_node(new_index)?.set_index(offset + 1);
        }
        for index in 1..=mesh.nb_triangles() {
            let (a, b, c) = mesh.triangle(index)?.get();
            if a != b && b != c && c != a {
                self.add_triangle(a - 1, b - 1, c - 1)?;
            }
        }
        self.copy_uv_and_normals(mesh)?;
        self.deflection = mesh.deflection();
        Ok(())
    }
    fn copy_uv_and_normals(&mut self, mesh: &Triangulation) -> Result<(), Failure> {
        let count = mesh.nb_nodes();
        if mesh.has_uv_nodes() {
            for offset in 0..count {
                let uv = mesh.uv_node(offset + 1)?;
                self.change_node(offset)?.set_uv(uv.x(), uv.y());
            }
        }
        if mesh.has_normals() {
            for offset in 0..count {
                let normal = mesh.normal_vec3(offset + 1)?;
                let widened = Xyz::new(normal.x().into(), normal.y().into(), normal.z().into());
                self.change_node(offset)?.set_normal(widened);
            }
        }
        Ok(())
    }
    /// Index of the last node; `mesh.max_node()`.
    pub fn max_node(&self) -> i32 {
        self.nodes.len() as i32 - 1
    }
    /// Index of the last triangle; `mesh.max_triangle()`.
    pub fn max_triangle(&self) -> i32 {
        self.triangles.len() as i32 - 1
    }
    /// Sets the deflection; `mesh.set_deflection(0.1)`.
    pub const fn set_deflection(&mut self, deflection: f64) {
        self.deflection = deflection;
    }
    /// The deflection, 0 unless set or copied; `mesh.deflection()`.
    pub const fn deflection(&self) -> f64 {
        self.deflection
    }
    /// Allocator passed to native `Allocator()`; `mesh.allocator() == &arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.allocator
    }
    /// Appends a node for a negative index, else replaces (growing with default nodes)
    /// the node at `index`; returns the node index; `mesh.set_node(point, -1)`.
    pub fn set_node(&mut self, point: Xyz, index: i32) -> i32 {
        let Ok(slot) = usize::try_from(index) else {
            self.nodes.push(CoherentNode::new(point));
            return self.nodes.len() as i32 - 1;
        };
        if slot >= self.nodes.len() {
            self.nodes.resize_with(slot + 1, CoherentNode::default);
        }
        self.nodes[slot] = CoherentNode::new(point);
        index
    }
    /// The node at an index; `mesh.node(0)?`.
    pub fn node(&self, index: i32) -> Result<&CoherentNode, Failure> {
        let count = self.nodes.len();
        usize::try_from(index)
            .ok()
            .and_then(|slot| self.nodes.get(slot))
            .ok_or_else(|| out_of_range("Node", index, count))
    }
    /// The node at an index, for editing; `mesh.change_node(0)?.set_uv(0.0, 1.0)`.
    pub fn change_node(&mut self, index: i32) -> Result<&mut CoherentNode, Failure> {
        let count = self.nodes.len();
        usize::try_from(index)
            .ok()
            .and_then(|slot| self.nodes.get_mut(slot))
            .ok_or_else(|| out_of_range("ChangeNode", index, count))
    }
    /// The triangle at an index; `mesh.triangle(0)?`.
    pub fn triangle(&self, index: usize) -> Result<&CoherentTriangle, Failure> {
        let count = self.triangles.len();
        self.triangles
            .get(index)
            .ok_or_else(|| out_of_range("Triangle", index as i32, count))
    }
    /// Number of link slots in use, as the native array length; `mesh.link_count()`.
    pub const fn link_count(&self) -> usize {
        self.link_count
    }
    /// The link at an index, as reached through `CoherentTriangle::get_link`; `mesh.link(0)?`.
    pub fn link(&self, index: usize) -> Result<&CoherentLink, Failure> {
        let count = self.link_count;
        self.links[..count]
            .get(index)
            .ok_or_else(|| out_of_range("Link", index as i32, count))
    }
    /// Number of nodes used by at least one triangle; `mesh.n_nodes()`.
    pub fn n_nodes(&self) -> i32 {
        self.nodes
            .iter()
            .filter(|node| !node.is_free_node())
            .count() as i32
    }
    /// Number of non-removed triangles; `mesh.n_triangles()`.
    pub fn n_triangles(&self) -> i32 {
        self.triangles.iter().filter(|t| !t.is_empty()).count() as i32
    }
    /// Number of non-removed links; `mesh.n_links()`.
    pub fn n_links(&self) -> i32 {
        self.links[..self.link_count]
            .iter()
            .filter(|link| !link.is_empty())
            .count() as i32
    }
    /// Non-removed triangles with their indices, as `IteratorOfTriangle`; `mesh.iter_triangles()`.
    pub fn iter_triangles(&self) -> impl Iterator<Item = (usize, &CoherentTriangle)> {
        self.triangles
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.is_empty())
    }
    /// Nodes used by triangles, with their indices, as `IteratorOfNode`; `mesh.iter_nodes()`.
    pub fn iter_nodes(&self) -> impl Iterator<Item = (usize, &CoherentNode)> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| !node.is_free_node())
    }
    /// Non-removed links with their indices, as `IteratorOfLink`; `mesh.iter_links()`.
    pub fn iter_links(&self) -> impl Iterator<Item = (usize, &CoherentLink)> {
        self.links[..self.link_count]
            .iter()
            .enumerate()
            .filter(|(_, link)| !link.is_empty())
    }
}
impl Drop for CoherentTriangulation {
    fn drop(&mut self) {
        for node in &mut self.nodes {
            let _ = node.clear_in(&self.allocator);
        }
    }
}
pub(super) fn out_of_range(member: &str, index: i32, count: usize) -> Failure {
    Failure::out_of_range(&format!(
        "Poly_CoherentTriangulation::{member}: index {index}; expected [0, {count})"
    ))
}
