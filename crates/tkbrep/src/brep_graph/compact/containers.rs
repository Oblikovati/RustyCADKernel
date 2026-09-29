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
// Rust translation of the shell, solid, compound and compsolid rebuild of
// `BRepGraph_Compact::Perform` (ModelingData/TKBRep/BRepGraph/BRepGraph_Compact.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The container rebuild: each mapped container with its remapped children, the new
//! references mapped to the old ones, then the old generations and UIDs.
use super::maps::{NodeMaps, RefMaps};
use super::topology::copy_generations;
use crate::brep::errors::program_error;
use crate::brep_graph::editor::SolidOps;
use crate::brep_graph::id_walk::active_ids;
use crate::brep_graph::refs_iterator::{
    RefsChildOfCompound, RefsFaceOfShell, RefsShellOfSolid, RefsSolidOfCompSolid,
};
use crate::brep_graph::*;
use rustycad_tkernel::standard::Failure;

/// The shells in active order with their remapped faces (orientation kept).
pub(super) fn rebuild_shells(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &NodeMaps,
    refs: &mut RefMaps,
) -> Result<(), Failure> {
    for old in active_ids::<ShellTag>(graph) {
        if maps.shells.seek(&old).is_none() {
            continue;
        }
        let new = fresh.editor().shells().add();
        for reference in RefsFaceOfShell::new(graph, old) {
            let entry = graph.refs().faces().entry(reference);
            let face: FaceId = maps.remap_typed(entry.child_face_id);
            if !face.is_valid() {
                continue;
            }
            let new_ref = fresh.editor().shells().append(new, face, entry.orientation)?;
            if new_ref.is_valid() {
                refs.face_refs.bind(reference, new_ref);
            }
        }
    }
    copy_generations(graph, fresh, &maps.shells);
    Ok(())
}

/// The solids in active order with their remapped shells (orientation kept).
pub(super) fn rebuild_solids(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &NodeMaps,
    refs: &mut RefMaps,
) -> Result<(), Failure> {
    for old in active_ids::<SolidTag>(graph) {
        if maps.solids.seek(&old).is_none() {
            continue;
        }
        let new = fresh.editor().solids().add();
        for reference in RefsShellOfSolid::new(graph, old) {
            let entry = graph.refs().shells().entry(reference);
            let shell: ShellId = maps.remap_typed(entry.child_shell_id);
            if !shell.is_valid() {
                continue;
            }
            let new_ref = fresh.editor().solids().append(new, shell, entry.orientation)?;
            if new_ref.is_valid() {
                refs.shell_refs.bind(reference, new_ref);
            }
        }
    }
    copy_generations(graph, fresh, &maps.solids);
    Ok(())
}

/// The compounds: all created empty first (at their precomputed ids) so a compound
/// child needs no storage order, then each one's remapped children appended with their
/// orientation and local location.
pub(super) fn rebuild_compounds(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &NodeMaps,
    refs: &mut RefMaps,
) -> Result<(), Failure> {
    let compounds = active_ids::<CompoundTag>(graph);
    for old in &compounds {
        let Some(&expected) = maps.compounds.seek(old) else {
            continue;
        };
        let new = fresh.editor().compounds().add(&[]);
        if new != expected {
            return Err(program_error(
                "BRepGraph_Compact: unexpected compound id",
                &format!("{new:?} (expected {expected:?})"),
            ));
        }
    }
    for old in compounds {
        let Some(&new) = maps.compounds.seek(&old) else {
            continue;
        };
        append_compound_children(graph, fresh, maps, refs, old, new)?;
    }
    copy_generations(graph, fresh, &maps.compounds);
    Ok(())
}

fn append_compound_children(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &NodeMaps,
    refs: &mut RefMaps,
    old: CompoundId,
    new: CompoundId,
) -> Result<(), Failure> {
    for reference in RefsChildOfCompound::new(graph, old) {
        let entry = graph.refs().children().entry(reference);
        let child = maps.remap(entry.child_node_id);
        if !child.is_valid() {
            continue;
        }
        let new_ref = fresh.editor().compounds().append(new, child, entry.orientation)?;
        if new_ref.is_valid() {
            let location = &entry.local_location;
            fresh
                .editor()
                .generic()
                .set_child_ref_local_location(new_ref, location)?;
            refs.child_refs.bind(reference, new_ref);
        }
    }
    Ok(())
}

/// The compsolids with their remapped solids; each new solid reference takes the old
/// orientation through a reference guard and is mapped.
pub(super) fn rebuild_comp_solids(
    graph: &BRepGraph,
    fresh: &mut BRepGraph,
    maps: &NodeMaps,
    refs: &mut RefMaps,
) -> Result<(), Failure> {
    for old in active_ids::<CompSolidTag>(graph) {
        if maps.comp_solids.seek(&old).is_none() {
            continue;
        }
        let (mut solids, mut old_refs) = (Vec::new(), Vec::new());
        for reference in RefsSolidOfCompSolid::new(graph, old) {
            let solid: SolidId = maps.remap_typed(graph.refs().solids().entry(reference).child_solid_id);
            if solid.is_valid() {
                solids.push(solid);
                old_refs.push(reference);
            }
        }
        let new = fresh.editor().comp_solids().add(&solids);
        if !new.is_valid() {
            continue;
        }
        let new_refs = fresh.topo().comp_solids().relations(new).solid_ref_ids.clone();
        for (old_ref, new_ref) in old_refs.into_iter().zip(new_refs) {
            let orientation = graph.refs().solids().entry(old_ref).orientation;
            let mut guard = fresh.editor().solids().mutate_ref(new_ref)?;
            SolidOps::set_ref_orientation_guarded(&mut guard, orientation);
            drop(guard);
            refs.solid_refs.bind(old_ref, new_ref);
        }
    }
    copy_generations(graph, fresh, &maps.comp_solids);
    Ok(())
}
