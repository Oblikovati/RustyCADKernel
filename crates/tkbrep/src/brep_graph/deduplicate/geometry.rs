// Rust translation of the geometry scope of `BRepGraph_Deduplicate::Perform`
// (ModelingData/TKBRep/BRepGraph/BRepGraph_Deduplicate.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry pass: faces whose surface equals an earlier face's surface (under the
//! `GeomHash` hashers) take that surface; edges likewise take the earlier curve.
use super::{DeduplicateResult, Options};
use crate::brep_graph::id_walk::full_ids;
use crate::brep_graph::layer_history::{HistoryKind, LayerHistory};
use crate::brep_graph::tool;
use crate::brep_graph::{BRepGraph, EdgeId, EdgeTag, FaceId, FaceTag, NodeId, NodeTag, TypedId};
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, HashKey, HashPolicy};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_hash::{GeomHashCurveHasher, GeomHashSurfaceHasher};

/// A duplicate-to-canonical id map (`NCollection_DataMap<Id, Id>`), iterated in the
/// native bucket order by the rewrites and merges.
pub(super) type RewriteMap<Id> = DataMap<Id, Id, DefaultHashPolicy>;

/// `NCollection_DataMap(max(1, theCount))` of the rewrite maps.
pub(super) fn rewrite_map<Id: std::fmt::Debug + HashKey + PartialEq>(count: u32) -> RewriteMap<Id> {
    DataMap::with_hasher(DefaultHashPolicy, count.max(1) as usize)
}

/// The geometry scope: counts the canonical surfaces and curves, then (unless
/// analyzing) rewrites the duplicates with a Modified history record each.
pub(super) fn canonicalize(
    graph: &mut BRepGraph,
    history: &Handle<LayerHistory>,
    options: &Options,
    result: &mut DeduplicateResult,
) -> Result<(), Failure> {
    let surface_rewrites = surface_rewrites(graph, options);
    let curve_rewrites = curve_rewrites(graph, options);
    result.nb_canonical_surfaces = graph.topo().faces().nb() - surface_rewrites.len() as u32;
    result.nb_canonical_curves = graph.topo().edges().nb() - curve_rewrites.len() as u32;
    if options.analyze_only {
        return Ok(());
    }
    rewrite_surfaces(graph, history, &surface_rewrites, result)?;
    rewrite_curves(graph, history, &curve_rewrites, result)
}

// Face -> earlier face with an equal surface, for the active faces with a surface.
fn surface_rewrites(graph: &BRepGraph, options: &Options) -> RewriteMap<FaceId> {
    let hasher = GeomHashSurfaceHasher::new(options.comp_tolerance, options.hash_tolerance);
    geometry_rewrites::<FaceTag, dyn GeomSurface, _>(graph, hasher, |graph, face| {
        tool::face::has_surface(graph, face).then(|| tool::face::surface(graph, face))
    })
}

// Edge -> earlier edge with an equal curve, for the active edges with a curve.
fn curve_rewrites(graph: &BRepGraph, options: &Options) -> RewriteMap<EdgeId> {
    let hasher = GeomHashCurveHasher::new(options.comp_tolerance, options.hash_tolerance);
    geometry_rewrites::<EdgeTag, dyn GeomCurve, _>(graph, hasher, |graph, edge| {
        tool::edge::has_curve(graph, edge).then(|| tool::edge::curve(graph, edge))
    })
}

// The shared body: the geometry of each active node (in full-iterator order) is looked
// up in a hashed map sized `max(1, 2 * Nb)`; the first node owning an equal geometry is
// its canonical node, and every later one maps to it in a map sized `max(1, Nb)`.
fn geometry_rewrites<T, G, H>(
    graph: &BRepGraph,
    hasher: H,
    geometry_of: impl Fn(&BRepGraph, TypedId<T>) -> Option<Handle<G>>,
) -> RewriteMap<TypedId<T>>
where
    T: NodeTag,
    G: ?Sized + RuntimeTyped,
    H: HashPolicy<Handle<G>>,
{
    let nb = graph.inc_storage().nb::<T>();
    let buckets = (nb as usize * 2).max(1);
    let mut canonical: DataMap<GeometryKey<G>, TypedId<T>, GeometryHasher<H>> =
        DataMap::with_hasher(GeometryHasher(hasher), buckets);
    let mut rewrites = rewrite_map(nb);
    for node in full_ids::<T>(graph) {
        if node.is_removed(graph) {
            continue;
        }
        let Some(geometry) = geometry_of(graph, node) else {
            continue;
        };
        let geometry = GeometryKey(geometry);
        match canonical.seek(&geometry).copied() {
            None => {
                canonical.bind(geometry, node);
            }
            Some(canon) if canon != node => {
                rewrites.bind(node, canon);
            }
            Some(_) => {}
        }
    }
    rewrites
}

// A geometry handle as a map key; it prints its address.
struct GeometryKey<G: ?Sized + RuntimeTyped>(Handle<G>);

impl<G: ?Sized + RuntimeTyped> std::fmt::Debug for GeometryKey<G> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "GeometryKey({:?})", self.0.object_address())
    }
}

// The `GeomHash` hasher of the geometry map, applied to the wrapped handles.
struct GeometryHasher<H>(H);

impl<G: ?Sized + RuntimeTyped, H: HashPolicy<Handle<G>>> HashPolicy<GeometryKey<G>>
    for GeometryHasher<H>
{
    fn hash(&self, key: &GeometryKey<G>) -> usize {
        self.0.hash(&key.0)
    }

    fn equal(&self, first: &GeometryKey<G>, second: &GeometryKey<G>) -> bool {
        self.0.equal(&first.0, &second.0)
    }
}

// Each duplicate face takes its canonical face's surface under a (clean) face guard.
fn rewrite_surfaces(
    graph: &mut BRepGraph,
    history: &Handle<LayerHistory>,
    rewrites: &RewriteMap<FaceId>,
    result: &mut DeduplicateResult,
) -> Result<(), Failure> {
    let label = AsciiString::new("Dedup:CanonicalizeSurface");
    for (&face, &canon) in rewrites.items() {
        let surface = tool::face::surface(graph, canon);
        let mut guard = graph.editor().faces().mutate(face)?;
        guard
            .graph_mut()
            .editor()
            .faces()
            .set_surface(face, &surface)?;
        drop(guard);
        result.nb_surface_rewrites += 1;
        result.affected_faces.push(face);
        let canon: NodeId = canon.into();
        history.record(graph, &label, face.into(), &[canon], HistoryKind::Modified);
        result.nb_history_records += 1;
    }
    Ok(())
}

// Each duplicate edge takes its canonical edge's curve over its own range; a canonical
// edge without curve is skipped.
fn rewrite_curves(
    graph: &mut BRepGraph,
    history: &Handle<LayerHistory>,
    rewrites: &RewriteMap<EdgeId>,
    result: &mut DeduplicateResult,
) -> Result<(), Failure> {
    let label = AsciiString::new("Dedup:CanonicalizeCurve");
    for (&edge, &canon) in rewrites.items() {
        let curve = tool::edge::curve(graph, canon);
        if curve.is_null() {
            continue;
        }
        let (first, last) = tool::edge::range(graph, edge);
        graph
            .editor()
            .edges()
            .set_curve(edge, &curve, first, last)?;
        result.nb_curve_rewrites += 1;
        result.affected_edges.push(edge);
        let canon: NodeId = canon.into();
        history.record(graph, &label, edge.into(), &[canon], HistoryKind::Modified);
        result.nb_history_records += 1;
    }
    Ok(())
}
