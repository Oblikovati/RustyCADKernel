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
// Rust translation of ValidateRelations, ValidateWireCoEdgeOrder(s) and
// CanonicalizeWireCoEdgeOrder(Status) of ModelingData/TKBRep/BRepGraphInc/
// BRepGraphInc_Storage.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The relation consistency check and the wire coedge ordering.
use super::Storage;
use super::wire_order::{build_co_edge_order, co_edge_ids_are_unique, co_edge_oriented_vertices};
use crate::brep_graph::*;

/// `BRepGraphInc_Storage::WireCoEdgeOrderStatus`: how a wire order was reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireCoEdgeOrderStatus {
    /// `Connected`: the stored order was already connected.
    Connected,
    /// `Reordered`: reordered into an exactly connected chain.
    Reordered,
    /// `ToleranceOrdered`: ordered through tolerance-equivalent endpoints.
    ToleranceOrdered,
    /// `Partial`: grouped into best-effort connected runs.
    Partial,
    /// `InvalidInput`: the wire or its coedges are invalid.
    InvalidInput,
}

impl Storage {
    /// `ValidateRelations()`: every relation list agrees with the record endpoints.
    pub fn validate_relations(&self) -> bool {
        self.co_edges_are_consistent()
            && self.edges_are_consistent()
            && self.wires_are_consistent()
            && self.ref_lists_are_consistent()
            && self.compounds_are_consistent()
            && self.products_are_consistent()
            && self.occurrences_are_consistent()
    }

    fn co_edges_are_consistent(&self) -> bool {
        (0..self.nb_co_edges())
            .map(CoEdgeId::new)
            .all(|id| self.is_removed(id) || self.co_edge_is_consistent(id))
    }

    fn co_edge_is_consistent(&self, id: CoEdgeId) -> bool {
        let def = self.co_edge(id);
        let has_face = def.face_id.is_valid_for(self.nb_faces()) && !self.is_removed(def.face_id);
        let rep_ok = |valid: bool, in_range: bool, removed: bool, parent: CoEdgeId| {
            !valid || (in_range && (removed || (has_face && parent == id)))
        };
        let curve = def.curve2d_rep_id;
        let curve_ok = rep_ok(
            curve.is_valid(),
            curve.is_valid_for(self.nb_co_edge_curves2d()),
            self.is_removed(curve),
            self.rep_parent(
                |s| s.co_edge_curve2d_rep(curve).parent_co_edge_id,
                curve.is_valid_for(self.nb_co_edge_curves2d()),
            ),
        );
        let polygon = def.polygon2d_rep_id;
        let polygon_ok = rep_ok(
            polygon.is_valid(),
            polygon.is_valid_for(self.nb_co_edge_polygons2d()),
            self.is_removed(polygon),
            self.rep_parent(
                |s| s.co_edge_polygon2d_rep(polygon).parent_co_edge_id,
                polygon.is_valid_for(self.nb_co_edge_polygons2d()),
            ),
        );
        let on_tri = def.polygon_on_tri_rep_id;
        let on_tri_ok = rep_ok(
            on_tri.is_valid(),
            on_tri.is_valid_for(self.nb_co_edge_polygons_on_tri()),
            self.is_removed(on_tri),
            self.rep_parent(
                |s| s.co_edge_polygon_on_tri_rep(on_tri).parent_co_edge_id,
                on_tri.is_valid_for(self.nb_co_edge_polygons_on_tri()),
            ),
        );
        curve_ok && polygon_ok && on_tri_ok && self.co_edge_links_are_consistent(id)
    }

    // The owner coedge of a use record, read only when the record is in range.
    fn rep_parent(&self, parent: impl Fn(&Self) -> CoEdgeId, in_range: bool) -> CoEdgeId {
        if in_range {
            parent(self)
        } else {
            CoEdgeId::default()
        }
    }

    // The wire list, the edge list and the face-wire use of an active coedge.
    fn co_edge_links_are_consistent(&self, id: CoEdgeId) -> bool {
        let def = self.co_edge(id);
        let wire_in_range = def.parent_wire_id.is_valid_for(self.nb_wires());
        if wire_in_range
            && !self
                .wire_relations(def.parent_wire_id)
                .co_edge_ids
                .contains(&id)
        {
            return false;
        }
        if !def.child_edge_id.is_valid_for(self.nb_edges()) {
            return true;
        }
        if !self
            .edge_relations(def.child_edge_id)
            .co_edge_ids
            .contains(&id)
        {
            return false;
        }
        if !wire_in_range || !def.face_id.is_valid_for(self.nb_faces()) {
            return true;
        }
        self.face_relations(def.face_id)
            .wire_ref_ids
            .iter()
            .any(|ref_id| {
                ref_id.is_valid_for(self.nb_wire_refs()) && !self.is_removed(*ref_id) && {
                    let entry = self.wire_ref(*ref_id);
                    entry.parent_face_id == def.face_id && entry.child_wire_id == def.parent_wire_id
                }
            })
    }

    fn edges_are_consistent(&self) -> bool {
        (0..self.nb_edges())
            .map(EdgeId::new)
            .all(|edge| self.is_removed(edge) || self.edge_is_consistent(edge))
    }

    fn edge_is_consistent(&self, edge: EdgeId) -> bool {
        let co_edges_ok = self.edge_relations(edge).co_edge_ids.iter().all(|co_edge| {
            co_edge.is_valid_for(self.nb_co_edges())
                && !self.is_removed(*co_edge)
                && self.co_edge(*co_edge).child_edge_id == edge
        });
        let def = self.edge(edge);
        let vertex_ok = |ref_id: VertexRefId| {
            if !ref_id.is_valid_for(self.nb_vertex_refs()) || self.is_removed(ref_id) {
                return true;
            }
            let vertex = self.vertex_ref(ref_id).child_vertex_id;
            !vertex.is_valid_for(self.nb_vertices())
                || self.is_removed(vertex)
                || self.vertex_relations(vertex).edge_ids.contains(&edge)
        };
        co_edges_ok && vertex_ok(def.start_vertex_ref_id) && vertex_ok(def.end_vertex_ref_id)
    }

    fn wires_are_consistent(&self) -> bool {
        (0..self.nb_wires()).map(WireId::new).all(|wire| {
            self.is_removed(wire)
                || self.wire_relations(wire).co_edge_ids.iter().all(|co_edge| {
                    co_edge.is_valid_for(self.nb_co_edges())
                        && !self.is_removed(*co_edge)
                        && self.co_edge(*co_edge).parent_wire_id == wire
                })
        })
    }

    // The face, shell, solid and compsolid forward lists against the child incoming lists.
    fn ref_lists_are_consistent(&self) -> bool {
        let faces = (0..self.nb_faces()).map(FaceId::new).all(|face| {
            self.is_removed(face)
                || self.face_relations(face).wire_ref_ids.iter().all(|id| {
                    self.forward_ref_ok(*id, |s| {
                        let entry = s.wire_ref(*id);
                        let child = entry.child_wire_id;
                        entry.parent_face_id == face
                            && s.child_active(child)
                            && s.wire_relations(child).parent_wire_ref_ids.contains(id)
                    })
                })
        });
        faces && self.shells_and_solids_are_consistent() && self.comp_solids_are_consistent()
    }

    fn shells_and_solids_are_consistent(&self) -> bool {
        let shells = (0..self.nb_shells()).map(ShellId::new).all(|shell| {
            self.is_removed(shell)
                || self.shell_relations(shell).face_ref_ids.iter().all(|id| {
                    self.forward_ref_ok(*id, |s| {
                        let entry = s.face_ref(*id);
                        let child = entry.child_face_id;
                        entry.parent_shell_id == shell
                            && s.child_active(child)
                            && s.face_relations(child).parent_face_ref_ids.contains(id)
                    })
                })
        });
        let solids = (0..self.nb_solids()).map(SolidId::new).all(|solid| {
            self.is_removed(solid)
                || self.solid_relations(solid).shell_ref_ids.iter().all(|id| {
                    self.forward_ref_ok(*id, |s| {
                        let entry = s.shell_ref(*id);
                        let child = entry.child_shell_id;
                        entry.parent_solid_id == solid
                            && s.child_active(child)
                            && s.shell_relations(child).parent_shell_ref_ids.contains(id)
                    })
                })
        });
        shells && solids
    }

    fn comp_solids_are_consistent(&self) -> bool {
        (0..self.nb_comp_solids())
            .map(CompSolidId::new)
            .all(|comp_solid| {
                self.is_removed(comp_solid)
                    || self
                        .comp_solid_relations(comp_solid)
                        .solid_ref_ids
                        .iter()
                        .all(|id| {
                            self.forward_ref_ok(*id, |s| {
                                let entry = s.solid_ref(*id);
                                let child = entry.child_solid_id;
                                entry.parent_comp_solid_id == comp_solid
                                    && s.child_active(child)
                                    && s.solid_relations(child).parent_solid_ref_ids.contains(id)
                            })
                        })
            })
    }

    // A forward reference is active and passes its endpoint check.
    fn forward_ref_ok<R: super::RefTag>(
        &self,
        id: TypedId<R>,
        check: impl Fn(&Self) -> bool,
    ) -> bool {
        id.is_valid_for(self.nb::<R>()) && !self.is_removed(id) && check(self)
    }

    fn child_active<N: super::NodeTag>(&self, id: TypedId<N>) -> bool {
        id.is_valid_for(self.nb::<N>()) && !self.is_removed(id)
    }

    fn compounds_are_consistent(&self) -> bool {
        (0..self.nb_compounds())
            .map(CompoundId::new)
            .all(|compound| {
                self.is_removed(compound)
                    || self
                        .compound_relations(compound)
                        .child_ref_ids
                        .iter()
                        .all(|id| {
                            self.forward_ref_ok(*id, |s| {
                                let entry = s.child_ref(*id);
                                let child = entry.child_node_id;
                                entry.parent_compound_id == compound
                                    && (!child.is_valid()
                                        || s.is_node_removed(child)
                                        || s.compound_refs_of_node(child).contains(id))
                            })
                        })
            })
    }

    fn products_are_consistent(&self) -> bool {
        (0..self.nb_products()).map(ProductId::new).all(|product| {
            self.is_removed(product)
                || self
                    .product_relations(product)
                    .occurrence_ref_ids
                    .iter()
                    .all(|id| {
                        self.forward_ref_ok(*id, |s| {
                            s.occurrence_ref(*id).parent_product_id == product
                                && s.occurrence_use_is_consistent(*id)
                        })
                    })
        })
    }

    // The occurrence incoming list and the child's occurrence map hold the reference.
    fn occurrence_use_is_consistent(&self, id: OccurrenceRefId) -> bool {
        let occurrence = self.occurrence_ref(id).child_occurrence_id;
        if !occurrence.is_valid_for(self.nb_occurrences()) || self.is_removed(occurrence) {
            return true;
        }
        if !self
            .occurrence_relations(occurrence)
            .parent_occurrence_ref_ids
            .contains(&id)
        {
            return false;
        }
        let child = self.occurrence(occurrence).child_node_id;
        !child.is_valid()
            || self.is_node_removed(child)
            || self.occurrence_refs_of_node(child).contains(&id)
    }

    fn occurrences_are_consistent(&self) -> bool {
        (0..self.nb_occurrences())
            .map(OccurrenceId::new)
            .all(|occurrence| {
                self.is_removed(occurrence)
                    || self
                        .occurrence_relations(occurrence)
                        .parent_occurrence_ref_ids
                        .iter()
                        .all(|id| {
                            self.forward_ref_ok(*id, |s| {
                                let entry = s.occurrence_ref(*id);
                                let product = entry.parent_product_id;
                                entry.child_occurrence_id == occurrence
                                    && s.child_active(product)
                                    && s.product_relations(product).occurrence_ref_ids.contains(id)
                            })
                        })
            })
    }

    /// `ValidateWireCoEdgeOrder(theWireId)`: an active wire whose unique, owned, active
    /// coedges chain end to start.
    pub fn validate_wire_co_edge_order(&self, wire: WireId) -> bool {
        if !wire.is_valid_for(self.nb_wires()) || self.is_removed(wire) {
            return false;
        }
        let ids = &self.wire_relations(wire).co_edge_ids;
        if !co_edge_ids_are_unique(ids) {
            return false;
        }
        let mut previous_end: Option<VertexId> = None;
        for id in ids {
            if !id.is_valid_for(self.nb_co_edges())
                || self.is_removed(*id)
                || self.co_edge(*id).parent_wire_id != wire
            {
                return false;
            }
            let Some((start, end)) = co_edge_oriented_vertices(self, *id) else {
                return false;
            };
            if previous_end.is_some_and(|previous| previous != start) {
                return false;
            }
            previous_end = Some(end);
        }
        true
    }

    /// `ValidateWireCoEdgeOrders()`: every active wire.
    pub fn validate_wire_co_edge_orders(&self) -> bool {
        (0..self.nb_wires())
            .map(WireId::new)
            .all(|wire| self.is_removed(wire) || self.validate_wire_co_edge_order(wire))
    }

    /// `CanonicalizeWireCoEdgeOrderStatus(theWireId)`: orders the wire's coedges and stores
    /// a changed order.
    pub fn canonicalize_wire_co_edge_order_status(
        &mut self,
        wire: WireId,
    ) -> WireCoEdgeOrderStatus {
        if !wire.is_valid_for(self.nb_wires()) || self.is_removed(wire) {
            return WireCoEdgeOrderStatus::InvalidInput;
        }
        let current = self.wire_relations(wire).co_edge_ids.clone();
        let (status, ordered) = build_co_edge_order(self, wire, &current);
        if matches!(
            status,
            WireCoEdgeOrderStatus::Reordered
                | WireCoEdgeOrderStatus::ToleranceOrdered
                | WireCoEdgeOrderStatus::Partial
        ) {
            self.set_wire_co_edges(wire, &ordered);
        }
        status
    }

    /// `CanonicalizeWireCoEdgeOrder(theWireId)`: false only for invalid input.
    pub fn canonicalize_wire_co_edge_order(&mut self, wire: WireId) -> bool {
        self.canonicalize_wire_co_edge_order_status(wire) != WireCoEdgeOrderStatus::InvalidInput
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph_inc::ParityOrientation;

    fn edge_between(storage: &mut Storage, a: VertexId, b: VertexId) -> EdgeId {
        let edge = storage.append_edge();
        for (vertex, start) in [(a, true), (b, false)] {
            let ref_id = storage.append_vertex_ref();
            storage.change_vertex_ref(ref_id).child_vertex_id = vertex;
            if start {
                storage.change_edge(edge).start_vertex_ref_id = ref_id;
            } else {
                storage.change_edge(edge).end_vertex_ref_id = ref_id;
            }
        }
        edge
    }

    #[test]
    fn canonicalize_reorders_a_shuffled_triangle() {
        let mut storage = Storage::new();
        let vertices: Vec<VertexId> = (0..3).map(|_| storage.append_vertex()).collect();
        let (face, wire) = (storage.append_face(), storage.append_wire());
        let pairs = [(1, 2), (0, 1), (2, 0)];
        for (a, b) in pairs {
            let edge = edge_between(&mut storage, vertices[a], vertices[b]);
            storage.create_co_edge_use(wire, edge, face, ParityOrientation::FORWARD);
        }
        assert!(!storage.validate_wire_co_edge_order(wire));
        let status = storage.canonicalize_wire_co_edge_order_status(wire);
        assert_eq!(status, WireCoEdgeOrderStatus::Reordered);
        assert!(storage.validate_wire_co_edge_orders());
        assert!(storage.canonicalize_wire_co_edge_order(wire));
        assert!(!storage.canonicalize_wire_co_edge_order(WireId::new(5)));
        storage.attach_wire_to_face(face, wire, ParityOrientation::FORWARD);
        storage.rebuild_derived_relations();
        assert!(storage.validate_relations());
    }
}
