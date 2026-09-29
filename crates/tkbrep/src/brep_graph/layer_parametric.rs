// Copyright (c) 2026 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerParametric.hxx and
// BRepGraph_LayerParametric.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_LayerParametric`: the base of the graph-owned parametric generators
//! (the BRepGraphPrim box, plane or loft layers build on it).
use crate::brep_graph::layer::BRepGraphLayer;
use crate::brep_graph::{BRepGraph, NodeId};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::Failure;

/// `BRepGraph_LayerParametric::GenerationFlag`: the graph artifacts a generator
/// creates or refreshes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GenerationFlag {
    /// `Topology` (0x01): topological nodes and references.
    Topology = 0x01,
    /// `Geometry` (0x02): analytic representations (without it the topology is
    /// mesh-only).
    Geometry = 0x02,
    /// `Mesh` (0x04): triangulation representations.
    Mesh = 0x04,
}

/// `BRepGraph_LayerParametric::MeshQuality`: the shared mesh quality hint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MeshQuality {
    /// `VeryCoarse`.
    VeryCoarse,
    /// `Coarse`.
    Coarse,
    /// `Medium`.
    Medium,
    /// `Fine`.
    Fine,
    /// `VeryFine`.
    VeryFine,
}

/// `THE_INVALID_INSTANCE`: no instance created.
pub const INVALID_INSTANCE: u32 = u32::MAX;

/// `THE_DEFAULT_GENERATION_FLAGS`: topology and analytic geometry.
pub const DEFAULT_GENERATION_FLAGS: u32 =
    GenerationFlag::Topology as u32 | GenerationFlag::Geometry as u32;

/// `BRepGraph_LayerParametric::AddResult`: the instance a generator added and the root
/// of its subtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParametricAddResult {
    /// `Instance`: `INVALID_INSTANCE` when nothing was created.
    pub instance: u32,
    /// `Root`: the root topology node.
    pub root: NodeId,
}

impl Default for ParametricAddResult {
    fn default() -> Self {
        Self {
            instance: INVALID_INSTANCE,
            root: NodeId::default(),
        }
    }
}

/// `GenerationMask(theFlag)`: the flag's bit.
pub const fn generation_mask(flag: GenerationFlag) -> u32 {
    flag as u32
}

/// `HasGenerationFlag(theFlags, theFlag)`: the mask holds the flag;
/// `has_generation_flag(DEFAULT_GENERATION_FLAGS, GenerationFlag::Mesh) == false`.
pub const fn has_generation_flag(flags: u32, flag: GenerationFlag) -> bool {
    flags & generation_mask(flag) != 0
}

/// `MeshQualityValue(theQuality, ...)`: the value of the quality's rung;
/// `mesh_quality_value(MeshQuality::Fine, [1, 2, 3, 4, 5]) == 4`.
pub const fn mesh_quality_value(quality: MeshQuality, ladder: [u32; 5]) -> u32 {
    match quality {
        MeshQuality::VeryCoarse => ladder[0],
        MeshQuality::Coarse => ladder[1],
        MeshQuality::Medium => ladder[2],
        MeshQuality::Fine => ladder[3],
        MeshQuality::VeryFine => ladder[4],
    }
}

/// `BRepGraph_LayerParametric`: the abstract base of parametric generator layers,
/// with the shared instance identity, generation flags and mesh quality controls. The
/// protected `lockLayer` waits for `BRepGraph_LayerLock` (docs/topology-design.md 11.9).
pub trait LayerParametric: BRepGraphLayer {
    /// `graphForMutation()` (protected): the attached graph; Standard_ProgramError when
    /// the layer is detached (or `graph` is not its graph).
    fn graph_for_mutation<'g>(
        &self,
        graph: &'g mut BRepGraph,
    ) -> Result<&'g mut BRepGraph, Failure> {
        if self.layer_base().attached_graph(graph).is_none() {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "BRepGraph_LayerParametric: layer is detached from graph (expected the graph \
                 whose registry holds it)",
                "",
            ));
        }
        Ok(graph)
    }
}
impl_transient_interface!(
    dyn LayerParametric,
    "BRepGraph_LayerParametric",
    dyn BRepGraphLayer
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_and_quality_ladder_follow_the_native_values() {
        assert_eq!(DEFAULT_GENERATION_FLAGS, 3);
        assert!(has_generation_flag(
            DEFAULT_GENERATION_FLAGS,
            GenerationFlag::Geometry
        ));
        assert!(!has_generation_flag(
            DEFAULT_GENERATION_FLAGS,
            GenerationFlag::Mesh
        ));
        assert_eq!(generation_mask(GenerationFlag::Mesh), 4);
        let ladder = [10, 20, 30, 40, 50];
        assert_eq!(mesh_quality_value(MeshQuality::VeryCoarse, ladder), 10);
        assert_eq!(mesh_quality_value(MeshQuality::VeryFine, ladder), 50);
        assert_eq!(ParametricAddResult::default().instance, u32::MAX);
    }
}
