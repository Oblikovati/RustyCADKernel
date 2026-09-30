// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PolyhedronBVH.hxx/.cxx
// and IntPatch_PolyhedronTool.hxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_PolyhedronBVH`: the triangles of a polyhedron as a BVH primitive set, and
//! `IntPatch_PolyhedronTool`, the static access to a polyhedron.
use super::polyhedron::IntPatchPolyhedron;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundingBox3, box3_to_bvh};
use rustycad_tkmath::bvh::constants::{LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH};
use rustycad_tkmath::bvh::{
    BvhBox, BvhLinearBuilder, BvhObject, BvhObjectTransient, BvhPrimitiveSet,
    BvhPrimitiveSetAccess, BvhSet, BvhTree, BvhVec3d,
};
use rustycad_tkmath::gp::{Point3, Xyz};
use std::sync::Arc;

/// `IntPatch_PolyhedronTool`: the static members the interference reads on a
/// polyhedron. `IntPatchPolyhedronTool::nb_triangles(&polyhedron)`.
pub struct IntPatchPolyhedronTool;

impl IntPatchPolyhedronTool {
    /// `Bounding(thePolyh)`. `IntPatchPolyhedronTool::bounding(&p)`.
    pub fn bounding(polyh: &IntPatchPolyhedron) -> &BoundingBox3 {
        polyh.bounding()
    }
    /// `ComponentsBounding(thePolyh)`. `IntPatchPolyhedronTool::components_bounding(&p)`.
    pub fn components_bounding(polyh: &IntPatchPolyhedron) -> &Array1<'static, BoundingBox3> {
        polyh.components_bounding()
    }
    /// `DeflectionOverEstimation(thePolyh)`.
    /// `IntPatchPolyhedronTool::deflection_over_estimation(&p)`.
    pub fn deflection_over_estimation(polyh: &IntPatchPolyhedron) -> f64 {
        polyh.deflection_over_estimation()
    }
    /// `NbTriangles(thePolyh)`. `IntPatchPolyhedronTool::nb_triangles(&p)`.
    pub fn nb_triangles(polyh: &IntPatchPolyhedron) -> i32 {
        polyh.nb_triangles()
    }
    /// `Triangle(thePolyh, Index, P1, P2, P3)`. `IntPatchPolyhedronTool::triangle(&p, 1)`.
    pub fn triangle(polyh: &IntPatchPolyhedron, index: i32) -> [i32; 3] {
        polyh.triangle(index)
    }
    /// `Point(thePolyh, Index)`. `IntPatchPolyhedronTool::point(&p, 1)?`.
    pub fn point(polyh: &IntPatchPolyhedron, index: i32) -> Result<Point3, Failure> {
        polyh.point(index)
    }
    /// `TriConnex(thePolyh, Triang, Pivot, Pedge, TriCon, OtherP)`.
    /// `IntPatchPolyhedronTool::tri_connex(&p, 1, 1, 2)?`.
    pub fn tri_connex(
        polyh: &IntPatchPolyhedron,
        triang: i32,
        pivot: i32,
        pedge: i32,
    ) -> Result<(i32, i32, i32), Failure> {
        polyh.tri_connex(triang, pivot, pedge)
    }
}

/// `IntPatch_PolyhedronBVH`: the non-degenerate triangles of a polyhedron (those with
/// a non-void box) as a BVH primitive set built by a linear builder (5 primitives per
/// leaf, depth 32); the index map follows the reordering of the build.
///
/// ```ignore
/// let mut set = IntPatchPolyhedronBVH::with_polyhedron(&polyhedron)?;
/// assert!(set.is_initialized() && set.size() > 0);
/// ```
pub struct IntPatchPolyhedronBVH<'a> {
    set: BvhPrimitiveSet<BvhVec3d>,
    poly: Option<&'a IntPatchPolyhedron>,
    index_map: Vec<i32>,
}

impl Default for IntPatchPolyhedronBVH<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntPatchPolyhedronBVH<'a> {
    /// `IntPatch_PolyhedronBVH()`: an empty set. `IntPatchPolyhedronBVH::new()`.
    pub fn new() -> Self {
        let builder = BvhLinearBuilder::<BvhVec3d>::new(LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH);
        Self {
            set: BvhPrimitiveSet::new(Some(Arc::new(builder))),
            poly: None,
            index_map: Vec::new(),
        }
    }

    /// `IntPatch_PolyhedronBVH(thePoly)`: the set of the polyhedron (which it borrows).
    /// `IntPatchPolyhedronBVH::with_polyhedron(&polyhedron)?`.
    pub fn with_polyhedron(poly: &'a IntPatchPolyhedron) -> Result<Self, Failure> {
        let mut set = Self::new();
        set.init(poly)?;
        Ok(set)
    }

    /// `Init(thePoly)`: keeps the triangles whose box is not void and marks the tree
    /// dirty. `set.init(&polyhedron)?`.
    pub fn init(&mut self, poly: &'a IntPatchPolyhedron) -> Result<(), Failure> {
        self.poly = Some(poly);
        let boxes = IntPatchPolyhedronTool::components_bounding(poly);
        self.index_map.clear();
        for idx in 1..=IntPatchPolyhedronTool::nb_triangles(poly) {
            if !box_at(boxes, idx)?.is_void() {
                self.index_map.push(idx);
            }
        }
        self.mark_dirty();
        Ok(())
    }

    /// `Clear()`: forgets the polyhedron. `set.clear()`.
    pub fn clear(&mut self) {
        self.poly = None;
        self.index_map.clear();
        self.mark_dirty();
    }

    /// `OriginalIndex(theIndex)`: the 1-based triangle of the 0-based `index`, 0
    /// outside the set. `set.original_index(0)`.
    pub fn original_index(&self, index: i32) -> i32 {
        if index < 0 || index >= self.size() {
            return 0;
        }
        self.index_map[index as usize]
    }

    /// `IsInitialized()`. `set.is_initialized()`.
    pub fn is_initialized(&self) -> bool {
        self.poly.is_some()
    }

    /// `BVH()`: the tree, built first while dirty. `set.built_bvh()?.length()`.
    pub fn built_bvh(&mut self) -> Result<&BvhTree<BvhVec3d>, Failure> {
        BvhPrimitiveSetAccess::bvh(self)
    }

    /// The triangle of a valid 0-based index.
    fn triangle_at(&self, index: i32) -> Option<(&'a IntPatchPolyhedron, i32)> {
        let poly: &IntPatchPolyhedron = self.poly?;
        (index >= 0 && index < self.size()).then(|| (poly, self.index_map[index as usize]))
    }
}

fn box_at(boxes: &Array1<'_, BoundingBox3>, idx: i32) -> Result<BoundingBox3, Failure> {
    if idx < boxes.lower() || idx > boxes.upper() {
        return Err(Failure::out_of_range(&format!(
            "IntPatch_PolyhedronBVH::Init() - triangle {idx} has no box (1..={})",
            boxes.upper()
        )));
    }
    Ok(boxes[idx].clone())
}

impl BvhSet<BvhVec3d> for IntPatchPolyhedronBVH<'_> {
    /// `Size()`: 0 without a polyhedron.
    fn size(&self) -> i32 {
        if self.poly.is_none() {
            return 0;
        }
        self.index_map.len() as i32
    }
    /// `Box(theIndex)`: the component box of the triangle; an empty box outside the set.
    fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        let Some((poly, idx)) = self.triangle_at(index) else {
            return Ok(BvhBox::default());
        };
        box3_to_bvh(&box_at(
            IntPatchPolyhedronTool::components_bounding(poly),
            idx,
        )?)
    }
    /// `Center(theIndex, theAxis)`: the coordinate of the triangle centroid; 0 outside
    /// the set or for another axis.
    fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        let Some((poly, idx)) = self.triangle_at(index) else {
            return Ok(0.0);
        };
        let [p1, p2, p3] = IntPatchPolyhedronTool::triangle(poly, idx);
        let xyz =
            |p: i32| -> Result<Xyz, Failure> { Ok(IntPatchPolyhedronTool::point(poly, p)?.xyz()) };
        let (a, b, c) = (xyz(p1)?, xyz(p2)?, xyz(p3)?);
        Ok(match axis {
            0 => (a.x() + b.x() + c.x()) / 3.0,
            1 => (a.y() + b.y() + c.y()) / 3.0,
            2 => (a.z() + b.z() + c.z()) / 3.0,
            _ => 0.0,
        })
    }
    /// `Swap(theIndex1, theIndex2)`: exchanges two entries of the index map.
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        if first != second {
            self.index_map.swap(first as usize, second as usize);
        }
        Ok(())
    }
    /// `Box()`: the cached box of the whole set.
    fn bounding_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.cached_box()
    }
}

impl BvhPrimitiveSetAccess<BvhVec3d> for IntPatchPolyhedronBVH<'_> {
    fn primitive_set(&self) -> &BvhPrimitiveSet<BvhVec3d> {
        &self.set
    }
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet<BvhVec3d> {
        &mut self.set
    }
}

impl BvhObject<BvhVec3d> for IntPatchPolyhedronBVH<'_> {
    fn object_transient(&self) -> &BvhObjectTransient {
        self.set.object_transient()
    }
    fn object_transient_mut(&mut self) -> &mut BvhObjectTransient {
        self.set.object_transient_mut()
    }
    fn object_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.cached_box()
    }
}
