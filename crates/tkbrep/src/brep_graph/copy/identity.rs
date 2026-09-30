// Rust translation of `countsForIdentityCopy`, `copyTopologyDefinitionsIdentity`,
// `copyReferencesIdentity`, `copyRepresentationsIdentity`, `copyRootProductsIdentity`,
// `copyUIDAndGraphStateIdentity`, `copyFlagsAndActiveCountsIdentity` and
// `copyFullGraphIdentity` of ModelingData/TKBRep/BRepGraph/BRepGraph_Copy.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The identity fast path of a full copy into an empty graph: every store slot for
//! slot, so the result ids equal the source ids.
use super::policy::{CachePolicy, DeepCopy, GeomPolicy, MeshPolicy};
use crate::brep_graph::cache_registry::CacheRegistry;
use crate::brep_graph::copy_remap::{MappingKind, Mode};
use crate::brep_graph::layer_registry::LayerRegistry;
use crate::brep_graph::*;
use crate::brep_graph_inc::{LoadCounts, Storage};
use rustycad_tkernel::standard::{Failure, Handle};

/// `copyFullGraphIdentity(...)`: sizes the result from the source counts (the dropped
/// representation stores empty), copies every store, the roots, the UID counters, the
/// generation, the GUID, the removed flags, the derived relations and the shape
/// bindings, then the layers and (under `CopyFresh`) the caches with the identity
/// mapping.
pub(crate) fn copy_full_graph_identity(
    source: &BRepGraph,
    target: &mut BRepGraph,
    geom: GeomPolicy,
    mesh: MeshPolicy,
    cache: CachePolicy,
) -> Result<bool, Failure> {
    let src = source.inc_storage();
    let dst = target.inc_storage_mut();
    dst.prepare_for_load(&counts_for_identity_copy(src, geom, mesh));
    copy_topology_definitions(src, dst, geom, mesh);
    copy_references(src, dst);
    copy_representations(src, dst, geom, mesh)?;
    dst.change_root_product_ids().clear();
    dst.change_root_product_ids()
        .extend_from_slice(src.root_product_ids());
    copy_uid_and_graph_state(src, dst);
    // Bulk-copy all removed flags in a single pass per kind, then recount.
    dst.copy_removed_flags_from(src);
    dst.recount_active_counts();
    dst.copy_derived_relations_from(src);
    dst.mark_uid_reverse_indexes_dirty();
    dst.copy_shape_bindings_from(src);
    LayerRegistry::copy_layers_to_with_mapping(source, target, MappingKind::Identity, Mode::Copy)?;
    if cache == CachePolicy::CopyFresh {
        CacheRegistry::copy_fresh_caches_to_with_mapping(
            source,
            target,
            MappingKind::Identity,
            Mode::Copy,
        )?;
    }
    Ok(true)
}

// `countsForIdentityCopy`: the source counts with the dropped representation stores
// empty.
fn counts_for_identity_copy(src: &Storage, geom: GeomPolicy, mesh: MeshPolicy) -> LoadCounts {
    let mut counts = src.counts();
    if geom == GeomPolicy::Drop {
        counts.nb_face_surface_reps = 0;
        counts.nb_edge_curve3d_reps = 0;
        counts.nb_co_edge_curve2d_reps = 0;
    }
    if mesh == MeshPolicy::Drop {
        counts.nb_face_triangulation_reps = 0;
        counts.nb_edge_polygon3d_reps = 0;
        counts.nb_co_edge_polygon2d_reps = 0;
        counts.nb_co_edge_polygon_on_tri_reps = 0;
    }
    counts
}

// Every definition of one kind, slot for slot.
fn copy_defs<T: NodeTag>(src: &Storage, dst: &mut Storage) {
    for index in 0..src.nb::<T>() {
        let id = TypedId::<T>::new(index);
        *dst.change_def(id) = src.def(id).clone();
    }
}

// Every entry of one reference kind, slot for slot.
fn copy_entries<T: RefTag>(src: &Storage, dst: &mut Storage) {
    for index in 0..src.nb::<T>() {
        let id = TypedId::<T>::new(index);
        *dst.change_entry(id) = src.entry(id).clone();
    }
}

// `copyTopologyDefinitionsIdentity`: the definitions, the dropped representation ids
// cleared from faces, then from coedges and edges.
fn copy_topology_definitions(src: &Storage, dst: &mut Storage, geom: GeomPolicy, mesh: MeshPolicy) {
    copy_defs::<VertexTag>(src, dst);
    copy_defs::<EdgeTag>(src, dst);
    copy_defs::<CoEdgeTag>(src, dst);
    copy_defs::<WireTag>(src, dst);
    copy_faces(src, dst, geom, mesh);
    copy_defs::<ShellTag>(src, dst);
    copy_defs::<SolidTag>(src, dst);
    copy_defs::<CompoundTag>(src, dst);
    copy_defs::<CompSolidTag>(src, dst);
    copy_defs::<ProductTag>(src, dst);
    copy_defs::<OccurrenceTag>(src, dst);
    if geom == GeomPolicy::Drop || mesh == MeshPolicy::Drop {
        clear_dropped_edge_representations(src, dst, geom, mesh);
    }
}

fn copy_faces(src: &Storage, dst: &mut Storage, geom: GeomPolicy, mesh: MeshPolicy) {
    for index in 0..src.nb_faces() {
        let face = FaceId::new(index);
        let copied = dst.change_face(face);
        *copied = src.face(face).clone();
        if geom == GeomPolicy::Drop {
            copied.surface_rep_id = FaceSurfaceRepId::invalid();
        }
        if mesh == MeshPolicy::Drop {
            copied.triangulation_rep_id = FaceTriangulationRepId::invalid();
        }
    }
}

fn clear_dropped_edge_representations(
    src: &Storage,
    dst: &mut Storage,
    geom: GeomPolicy,
    mesh: MeshPolicy,
) {
    for index in 0..src.nb_co_edges() {
        let co_edge = dst.change_co_edge(CoEdgeId::new(index));
        if geom == GeomPolicy::Drop {
            co_edge.curve2d_rep_id = CoEdgeCurve2DRepId::invalid();
        }
        if mesh == MeshPolicy::Drop {
            co_edge.polygon2d_rep_id = CoEdgePolygon2DRepId::invalid();
            co_edge.polygon_on_tri_rep_id = CoEdgePolygonOnTriRepId::invalid();
        }
    }
    for index in 0..src.nb_edges() {
        let edge = dst.change_edge(EdgeId::new(index));
        if geom == GeomPolicy::Drop {
            edge.curve3d_rep_id = EdgeCurve3DRepId::invalid();
        }
        if mesh == MeshPolicy::Drop {
            edge.polygon3d_rep_id = EdgePolygon3DRepId::invalid();
        }
    }
}

// `copyReferencesIdentity`.
fn copy_references(src: &Storage, dst: &mut Storage) {
    copy_entries::<ShellRefTag>(src, dst);
    copy_entries::<FaceRefTag>(src, dst);
    copy_entries::<WireRefTag>(src, dst);
    copy_entries::<VertexRefTag>(src, dst);
    copy_entries::<SolidRefTag>(src, dst);
    copy_entries::<ChildRefTag>(src, dst);
    copy_entries::<OccurrenceRefTag>(src, dst);
}

// Every record of one representation kind, its handle deep-copied under `Copy`.
fn copy_records<T: RepTag, H: ?Sized + DeepCopy>(
    src: &Storage,
    dst: &mut Storage,
    handle_of: impl Fn(&mut T::Record) -> &mut Handle<H>,
    deep_copy: bool,
) -> Result<(), Failure> {
    for index in 0..src.nb::<T>() {
        let id = TypedId::<T>::new(index);
        let record = dst.change_record(id);
        *record = src.record(id).clone();
        let handle = handle_of(record);
        let copied = match handle.get() {
            Some(stored) if deep_copy => Some(stored.deep_copy()?),
            _ => None,
        };
        if let Some(copied) = copied {
            *handle = copied;
        }
    }
    Ok(())
}

// `copyRepresentationsIdentity`: the kept representation stores, their handles
// deep-copied under the `Copy` policies.
fn copy_representations(
    src: &Storage,
    dst: &mut Storage,
    geom: GeomPolicy,
    mesh: MeshPolicy,
) -> Result<(), Failure> {
    if geom != GeomPolicy::Drop {
        let deep = geom == GeomPolicy::Copy;
        copy_records::<FaceSurfaceTag, _>(src, dst, |r| &mut r.surface, deep)?;
        copy_records::<EdgeCurve3DTag, _>(src, dst, |r| &mut r.curve, deep)?;
        copy_records::<CoEdgeCurve2DTag, _>(src, dst, |r| &mut r.curve, deep)?;
    }
    if mesh != MeshPolicy::Drop {
        let deep = mesh == MeshPolicy::Copy;
        copy_records::<FaceTriangulationTag, _>(src, dst, |r| &mut r.triangulation, deep)?;
        copy_records::<EdgePolygon3DTag, _>(src, dst, |r| &mut r.polygon, deep)?;
        copy_records::<CoEdgePolygon2DTag, _>(src, dst, |r| &mut r.polygon, deep)?;
        copy_records::<CoEdgePolygonOnTriTag, _>(src, dst, |r| &mut r.polygon, deep)?;
    }
    Ok(())
}

// `copyUIDAndGraphStateIdentity`: the per-kind UID counters, the generation and GUID.
fn copy_uid_and_graph_state(src: &Storage, dst: &mut Storage) {
    const NODE_KINDS: [NodeKind; 11] = [
        NodeKind::VERTEX,
        NodeKind::EDGE,
        NodeKind::CO_EDGE,
        NodeKind::WIRE,
        NodeKind::FACE,
        NodeKind::SHELL,
        NodeKind::SOLID,
        NodeKind::COMPOUND,
        NodeKind::COMP_SOLID,
        NodeKind::PRODUCT,
        NodeKind::OCCURRENCE,
    ];
    for kind in NODE_KINDS {
        dst.set_next_node_uid_counter(kind, src.next_node_uid_counter(kind));
    }
    const REF_KINDS: [RefKind; 7] = [
        RefKind::SHELL,
        RefKind::FACE,
        RefKind::WIRE,
        RefKind::VERTEX,
        RefKind::SOLID,
        RefKind::CHILD,
        RefKind::OCCURRENCE,
    ];
    for kind in REF_KINDS {
        dst.set_next_ref_uid_counter(kind, src.next_ref_uid_counter(kind));
    }
    dst.set_generation(src.generation());
    dst.set_graph_guid(*src.graph_guid());
}
