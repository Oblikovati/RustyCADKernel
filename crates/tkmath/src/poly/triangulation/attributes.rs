// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of the storage members of FoundationClasses/TKMath/Poly/Poly_Triangulation.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{
    ArrayOfNodes, ArrayOfUvNodes, Triangle, Triangulation, TriangulationFields, array_failure,
    in_member, node_count,
};
use rustycad_tkernel::{
    ncollection::{Array1, Vec3},
    standard::Failure,
};
impl Triangulation {
    /// Releases nodes, triangles, UV nodes and normals, keeping the node precision; `triangulation.clear()`.
    pub fn clear(&self) {
        self.fields.write().clear();
    }
    /// True for double-precision nodes (the default); `triangulation.is_double_precision()`.
    pub fn is_double_precision(&self) -> bool {
        self.fields.read().nodes.is_double_precision()
    }
    /// Selects node precision before allocation, else `Standard_ProgramError`;
    /// `triangulation.set_double_precision(false)?`.
    pub fn set_double_precision(&self, double: bool) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        fields.nodes.set_double_precision(double)?;
        fields.uv_nodes.set_double_precision(double)
    }
    /// Resizes nodes together with allocated UV nodes and normals; `triangulation.resize_nodes(8, true)?`.
    pub fn resize_nodes(&self, nb_nodes: i32, copy: bool) -> Result<(), Failure> {
        self.fields.write().resize_nodes(nb_nodes, copy)
    }
    /// Resizes the triangle array to `1..=nb_triangles`; `triangulation.resize_triangles(4, true)?`.
    pub fn resize_triangles(&self, nb_triangles: i32, copy: bool) -> Result<(), Failure> {
        self.fields
            .write()
            .triangles
            .resize_bounds(1, nb_triangles, copy)
            .map_err(|e| array_failure("Poly_Triangulation::ResizeTriangles", e))
    }
    /// Allocates UV nodes when absent or of another size; zero nodes raise
    /// `Standard_RangeError` natively; `triangulation.add_uv_nodes()?`.
    pub fn add_uv_nodes(&self) -> Result<(), Failure> {
        self.fields.write().add_uv_nodes()
    }
    /// Releases UV nodes, keeping their precision; `triangulation.remove_uv_nodes()`.
    pub fn remove_uv_nodes(&self) {
        self.fields.write().remove_uv_nodes();
    }
    /// Allocates normals when absent or of another size; `triangulation.add_normals()?`.
    pub fn add_normals(&self) -> Result<(), Failure> {
        self.fields.write().add_normals()
    }
    /// Releases normals; `triangulation.remove_normals()`.
    pub fn remove_normals(&self) {
        self.fields.write().normals = Array1::default();
    }
    /// Averages unnormalized single-precision triangle normals per node; a node without
    /// area gets (0, 0, 1); `triangulation.compute_normals()?`.
    pub fn compute_normals(&self) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        fields.add_normals()?;
        fields.normals.init(Vec3::from_value(0.0));
        for index in fields.triangles.lower()..=fields.triangles.upper() {
            let triangle = *fields
                .triangles
                .value(index)
                .map_err(|e| array_failure("ComputeNormals", e))?;
            fields.accumulate_normal(triangle)?;
        }
        for normal in fields.normals.iter_mut() {
            let modulus = normal.modulus();
            *normal = if modulus == 0.0 {
                Vec3::new(0.0, 0.0, 1.0)
            } else {
                *normal / modulus
            };
        }
        Ok(())
    }
}
impl TriangulationFields {
    pub(super) fn clear(&mut self) {
        if !self.nodes.is_empty() {
            let mut empty = ArrayOfNodes::default();
            // An empty array accepts any precision.
            let _ = empty.set_double_precision(self.nodes.is_double_precision());
            self.nodes = empty;
        }
        self.triangles = Array1::default();
        self.remove_uv_nodes();
        self.normals = Array1::default();
    }
    pub(super) fn remove_uv_nodes(&mut self) {
        if !self.uv_nodes.is_empty() {
            let mut empty = ArrayOfUvNodes::default();
            let _ = empty.set_double_precision(self.uv_nodes.is_double_precision());
            self.uv_nodes = empty;
        }
    }
    pub(crate) fn resize_nodes(&mut self, nb_nodes: i32, copy: bool) -> Result<(), Failure> {
        const MEMBER: &str = "Poly_Triangulation::ResizeNodes";
        self.nodes
            .resize(node_count(nb_nodes), copy)
            .map_err(|e| in_member(MEMBER, e))?;
        if !self.uv_nodes.is_empty() {
            self.uv_nodes
                .resize(node_count(nb_nodes), copy)
                .map_err(|e| in_member(MEMBER, e))?;
        }
        if !self.normals.is_empty() {
            self.normals
                .resize_bounds(0, nb_nodes.saturating_sub(1), copy)
                .map_err(|e| array_failure(MEMBER, e))?;
        }
        Ok(())
    }
    pub(crate) fn add_uv_nodes(&mut self) -> Result<(), Failure> {
        if self.uv_nodes.is_empty() || self.uv_nodes.len() != self.nodes.len() {
            self.uv_nodes
                .resize(self.nodes.len(), false)
                .map_err(|e| in_member("Poly_Triangulation::AddUVNodes", e))?;
        }
        Ok(())
    }
    pub(crate) fn add_normals(&mut self) -> Result<(), Failure> {
        if self.normals.is_empty() || self.normals.len() != self.nodes.len() {
            let upper = self.nodes.len() as i32 - 1;
            self.normals
                .resize_bounds(0, upper, false)
                .map_err(|e| array_failure("Poly_Triangulation::AddNormals", e))?;
        }
        Ok(())
    }
    fn accumulate_normal(&mut self, triangle: Triangle) -> Result<(), Failure> {
        let (a, b, c) = triangle.get();
        let corner = |index: i32| {
            self.nodes
                .value(crate::poly::errors::zero_based(index))
                .map(|point| point.xyz())
                .map_err(|e| in_member("Poly_Triangulation::ComputeNormals", e))
        };
        let (p0, p1, p2) = (corner(a)?, corner(b)?, corner(c)?);
        let normal = (p1 - p0).crossed(p2 - p0);
        let narrowed = Vec3::new(normal.x() as f32, normal.y() as f32, normal.z() as f32);
        for node in [a, b, c] {
            let slot = self
                .normals
                .change_value(node.saturating_sub(1))
                .map_err(|e| array_failure("Poly_Triangulation::ComputeNormals", e))?;
            *slot += narrowed;
        }
        Ok(())
    }
}
