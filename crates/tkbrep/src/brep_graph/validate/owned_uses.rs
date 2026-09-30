// Rust translation of BRepGraph_Validate::CheckOwnedUseReferences of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The private `CheckOwnedUseReferences` of the audit: every live representation use
//! names its owner back, carries a handle, and has exactly one owner.
//!
//! The seven native owner tallies (a count and a first-owner vector per kind) and the
//! seven copies of the per-use checks are one generic [`OwnerTally`] over the
//! representation tag, whose [`OwnedUse`] impl names the record's parent and handle.
use super::{Issue, Severity, report};
use crate::brep_graph::iterator::{CoEdgeIterator, EdgeIterator, FaceIterator};
use crate::brep_graph::*;
use crate::brep_graph_inc::{
    CoEdgeCurve2DRep, CoEdgePolygon2DRep, CoEdgePolygonOnTriRep, EdgeCurve3DRep, EdgePolygon3DRep,
    FaceSurfaceRep, FaceTriangulationRep,
};
use crate::brep_graph_inc::{RepTag, Storage};

// A representation kind whose use record names its owning node and holds a handle.
trait OwnedUse: RepTag {
    // The record name of the messages (`EdgeCurve3DRep`...).
    const USE_NAME: &'static str;
    // The parent field of the mismatch message (`ParentEdgeId`...).
    const PARENT_FIELD: &'static str;
    // The handle field of the null message (`Curve`...).
    const HANDLE_FIELD: &'static str;
    fn parent(record: &Self::Record) -> NodeId;
    fn is_null(record: &Self::Record) -> bool;
}

macro_rules! owned_uses {
    ($($tag:ty, $record:ty, $name:literal, $parent_field:literal, $parent:ident,
       $handle_field:literal, $handle:ident;)*) => {$(
        impl OwnedUse for $tag {
            const USE_NAME: &'static str = $name;
            const PARENT_FIELD: &'static str = $parent_field;
            const HANDLE_FIELD: &'static str = $handle_field;
            fn parent(record: &$record) -> NodeId {
                record.$parent.into()
            }
            fn is_null(record: &$record) -> bool {
                record.$handle.is_null()
            }
        }
    )*};
}

owned_uses! {
    FaceSurfaceTag, FaceSurfaceRep, "FaceSurfaceRep", "ParentFaceId", parent_face_id,
        "Surface", surface;
    FaceTriangulationTag, FaceTriangulationRep, "FaceTriangulationRep", "ParentFaceId",
        parent_face_id, "Triangulation", triangulation;
    EdgeCurve3DTag, EdgeCurve3DRep, "EdgeCurve3DRep", "ParentEdgeId", parent_edge_id,
        "Curve", curve;
    EdgePolygon3DTag, EdgePolygon3DRep, "EdgePolygon3DRep", "ParentEdgeId", parent_edge_id,
        "Polygon", polygon;
    CoEdgeCurve2DTag, CoEdgeCurve2DRep, "CoEdgeCurve2DRep", "ParentCoEdgeId",
        parent_co_edge_id, "Curve", curve;
    CoEdgePolygon2DTag, CoEdgePolygon2DRep, "CoEdgePolygon2DRep", "ParentCoEdgeId",
        parent_co_edge_id, "Polygon", polygon;
    CoEdgePolygonOnTriTag, CoEdgePolygonOnTriRep, "CoEdgePolygonOnTriRep", "ParentCoEdgeId",
        parent_co_edge_id, "Polygon", polygon;
}

// The owner count and the first owner of each slot of one representation store.
struct OwnerTally<T: OwnedUse> {
    counts: Vec<u32>,
    first_owners: Vec<NodeId>,
    marker: std::marker::PhantomData<T>,
}

impl<T: OwnedUse> OwnerTally<T> {
    fn new(storage: &Storage) -> Self {
        let count = storage.nb::<T>() as usize;
        Self {
            counts: vec![0; count],
            first_owners: vec![NodeId::default(); count],
            marker: std::marker::PhantomData,
        }
    }

    // One owner's use: counted, and checked for its parent and handle when live.
    fn visit(&mut self, storage: &Storage, id: TypedId<T>, owner: NodeId, issues: &mut Vec<Issue>) {
        if !id.is_valid_for(storage.nb::<T>()) || storage.is_removed(id) {
            return;
        }
        let record = storage.record(id);
        self.remember(id.index as usize, owner);
        if T::parent(record) != owner {
            let text = format!("{}.{} mismatch", T::USE_NAME, T::PARENT_FIELD);
            report(issues, Severity::Error, owner, &text);
        }
        if T::is_null(record) {
            let text = format!("{}.{} is null", T::USE_NAME, T::HANDLE_FIELD);
            report(issues, Severity::Error, owner, &text);
        }
    }

    // `aRememberOwner`.
    fn remember(&mut self, index: usize, owner: NodeId) {
        self.counts[index] += 1;
        if !self.first_owners[index].is_valid() {
            self.first_owners[index] = owner;
        }
    }

    // `aCheckStore`: every live slot has exactly one owner.
    fn check_store(&self, storage: &Storage, issues: &mut Vec<Issue>) {
        for index in 0..storage.nb::<T>() {
            if storage.is_removed(TypedId::<T>::new(index)) {
                continue;
            }
            let count = self.counts[index as usize];
            if count == 0 {
                let text = format!("Active {} has no owner", T::USE_NAME);
                report(issues, Severity::Error, NodeId::default(), &text);
            } else if count > 1 {
                let text = format!("Active {} has multiple owners", T::USE_NAME);
                report(
                    issues,
                    Severity::Error,
                    self.first_owners[index as usize],
                    &text,
                );
            }
        }
    }
}

// The seven tallies, filled in the native traversal order.
struct Tallies {
    surfaces: OwnerTally<FaceSurfaceTag>,
    triangulations: OwnerTally<FaceTriangulationTag>,
    curves3d: OwnerTally<EdgeCurve3DTag>,
    polygons3d: OwnerTally<EdgePolygon3DTag>,
    curves2d: OwnerTally<CoEdgeCurve2DTag>,
    polygons2d: OwnerTally<CoEdgePolygon2DTag>,
    polygons_on_tri: OwnerTally<CoEdgePolygonOnTriTag>,
}

/// `CheckOwnedUseReferences(theGraph, theIssues)` (private natively).
pub(super) fn check_owned_use_references(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let storage = graph.inc_storage();
    let mut tallies = Tallies {
        surfaces: OwnerTally::new(storage),
        triangulations: OwnerTally::new(storage),
        curves3d: OwnerTally::new(storage),
        polygons3d: OwnerTally::new(storage),
        curves2d: OwnerTally::new(storage),
        polygons2d: OwnerTally::new(storage),
        polygons_on_tri: OwnerTally::new(storage),
    };
    visit_edge_uses(graph, &mut tallies, issues);
    visit_co_edge_uses(graph, &mut tallies, issues);
    visit_face_uses(graph, &mut tallies, issues);
    tallies.surfaces.check_store(storage, issues);
    tallies.triangulations.check_store(storage, issues);
    tallies.curves3d.check_store(storage, issues);
    tallies.polygons3d.check_store(storage, issues);
    tallies.curves2d.check_store(storage, issues);
    tallies.polygons2d.check_store(storage, issues);
    tallies.polygons_on_tri.check_store(storage, issues);
}

fn visit_edge_uses(graph: &BRepGraph, tallies: &mut Tallies, issues: &mut Vec<Issue>) {
    let storage = graph.inc_storage();
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        let (owner, edge) = (NodeId::from(edges.current_id()), edges.current());
        tallies
            .curves3d
            .visit(storage, edge.curve3d_rep_id, owner, issues);
        tallies
            .polygons3d
            .visit(storage, edge.polygon3d_rep_id, owner, issues);
        edges.next();
    }
}

fn visit_co_edge_uses(graph: &BRepGraph, tallies: &mut Tallies, issues: &mut Vec<Issue>) {
    let storage = graph.inc_storage();
    let mut co_edges = CoEdgeIterator::new(graph);
    while co_edges.more() {
        let (owner, co_edge) = (NodeId::from(co_edges.current_id()), co_edges.current());
        tallies
            .curves2d
            .visit(storage, co_edge.curve2d_rep_id, owner, issues);
        tallies
            .polygons2d
            .visit(storage, co_edge.polygon2d_rep_id, owner, issues);
        let on_tri = co_edge.polygon_on_tri_rep_id;
        tallies
            .polygons_on_tri
            .visit(storage, on_tri, owner, issues);
        co_edges.next();
    }
}

fn visit_face_uses(graph: &BRepGraph, tallies: &mut Tallies, issues: &mut Vec<Issue>) {
    let storage = graph.inc_storage();
    let mut faces = FaceIterator::new(graph);
    while faces.more() {
        let (owner, face) = (NodeId::from(faces.current_id()), faces.current());
        tallies
            .surfaces
            .visit(storage, face.surface_rep_id, owner, issues);
        let triangulation = face.triangulation_rep_id;
        tallies
            .triangulations
            .visit(storage, triangulation, owner, issues);
        faces.next();
    }
}
