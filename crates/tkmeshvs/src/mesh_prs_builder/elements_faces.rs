// SPDX-License-Identifier: GPL-2.0-only
//! The faces of `MeshVS_MeshPrsBuilder::BuildElements`: the edges not covered by a link
//! (overlap control), else every edge, and the triangles of the solid modes.
use super::elements::{Element, ElementPass};
use super::primitives::node_xyz;
use super::solid::{MeshVSFaceSolidOptions, meshvs_mesh_prs_add_face_solid_prs};
use super::{MeshVSMeshPrsBuilder, MeshVSMeshPrsBuilderSource};
use crate::node_pairs::{MeshVSNodePair, MeshVSSymmetricPairHasher, MeshVSTwoNodes};
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::graphic3d::Graphic3dArrayOfSegments;

/// The face of an element of the second pass.
pub(super) fn add_face<B: MeshVSMeshPrsBuilderSource + ?Sized>(
    builder: &B,
    pass: &ElementPass<'_>,
    element: &Element<'_, '_>,
    segment_map: &mut Map<MeshVSNodePair, MeshVSSymmetricPairHasher>,
) -> Result<(), Failure> {
    let s = pass.settings;
    if let (true, true, Some(edges)) = (
        s.show_edges,
        s.overlap_control,
        pass.arrays.edge_segments.get(),
    ) {
        add_overlap_controlled_edges(pass, element, edges, segment_map)?;
    }
    if s.overlap_control && !s.is_shading {
        return Ok(());
    }
    if let (false, true, Some(edges)) = (
        s.overlap_control,
        s.show_edges,
        pass.arrays.edge_segments.get(),
    ) {
        MeshVSMeshPrsBuilder::add_face_wire_prs(
            element.coords,
            element.nb_nodes,
            edges,
            s.is_shrink || s.has_select,
            s.shrink_coef,
        )?;
    }
    if (s.is_shading || s.is_shrink) && !s.has_select {
        let triangles = pass.arrays.face_triangles.get().expect("face triangles");
        let options = MeshVSFaceSolidOptions {
            is_shaded: s.is_reflect,
            is_shrinked: s.is_shrink || s.has_select,
            shrink_coef: s.shrink_coef,
            is_smooth_shading: s.is_smooth,
        };
        meshvs_mesh_prs_add_face_solid_prs(
            builder,
            element.key,
            element.coords,
            element.nb_nodes,
            s.max_nodes,
            triangles,
            options,
        )?;
    }
    Ok(())
}

fn add_edge(edges: &Graphic3dArrayOfSegments, from: [f64; 3], to: [f64; 3]) -> Result<(), Failure> {
    edges.add_vertex_f64(from[0], from[1], from[2])?;
    edges.add_vertex_f64(to[0], to[1], to[2]).map(|_| ())
}

/// The native overlap-control loop: the edges before each edge that is a link are drawn
/// (the link itself is not), then the remaining edges once each across the faces.
fn add_overlap_controlled_edges(
    pass: &ElementPass<'_>,
    element: &Element<'_, '_>,
    edges: &Graphic3dArrayOfSegments,
    segment_map: &mut Map<MeshVSNodePair, MeshVSSymmetricPairHasher>,
) -> Result<(), Failure> {
    let (coords, nodes, nb_nodes) = (element.coords, element.nodes, element.nb_nodes);
    let node = |i: i32| {
        nodes
            .value(i)
            .copied()
            .map_err(|e| e.into_failure("MeshVS node ids"))
    };
    let mut last = 0;
    let mut two_nodes = MeshVSTwoNodes::new(node(1)?, 0);
    for i in 1..=nb_nodes {
        if i > 1 {
            two_nodes.first = two_nodes.second;
        }
        two_nodes.second = if i < nb_nodes { node(i + 1)? } else { node(1)? };
        if !pass.link_nodes.contains(&two_nodes) {
            continue;
        }
        for index in last + 1..i {
            add_edge(
                edges,
                node_xyz(coords, index)?,
                node_xyz(coords, index + 1)?,
            )?;
        }
        last = i;
    }
    for index in last..nb_nodes {
        let next = (index + 1) % nb_nodes;
        let segment = (node(index + 1)?, node(next + 1)?);
        if !segment_map.contains(&segment) {
            add_edge(
                edges,
                node_xyz(coords, index + 1)?,
                node_xyz(coords, next + 1)?,
            )?;
            segment_map.add(segment);
        }
    }
    Ok(())
}
