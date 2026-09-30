#[test]
fn ParityOrientationRejectsInternalExternal() {
    let mut parity = ParityOrientation::default();
    for orientation in [Orientation::Internal, Orientation::External] {
        assert_eq!(
            ParityOrientation::new(orientation)
                .unwrap_err()
                .exception_type(),
            "Standard_ProgramError"
        );
        assert_eq!(
            parity.assign(orientation).unwrap_err().exception_type(),
            "Standard_ProgramError"
        );
    }
    parity.assign(Orientation::Forward).unwrap();
    assert_eq!(Orientation::from(parity), Orientation::Forward);
    parity.assign(Orientation::Reversed).unwrap();
    assert_eq!(Orientation::from(parity), Orientation::Reversed);
}
#[test]
fn Storage_AppendAccess_UsesTypedIds() {
    let mut storage = Storage::new();
    let vertex = storage.append_vertex();
    let reference = storage.append_face_ref();
    let surface = storage.append_face_surface_rep();
    assert_eq!([vertex.index, reference.index, surface.index], [0, 0, 0]);
    assert!(!storage.is_removed(vertex));
    assert!(!storage.is_removed(reference));
    storage.change_vertex(vertex).tolerance = 1.25;
    storage.change_face_ref(reference).orientation = ParityOrientation::REVERSED;
    support::expect_double_eq(storage.vertex(vertex).tolerance, 1.25);
    inc_storage_orientation(&storage, reference, Orientation::Reversed, true);
    storage.change_face_ref(reference).orientation = ParityOrientation::FORWARD;
    inc_storage_orientation(&storage, reference, Orientation::Forward, false);
}
#[test]
fn Storage_GenericIdDispatch_UsesTypedHelpers() {
    let mut storage = Storage::new();
    let vertex = storage.append_vertex();
    let reference = storage.append_face_ref();
    storage.append_face_surface_rep();
    storage.change_face_ref(reference).orientation = ParityOrientation::REVERSED;
    assert_eq!(
        storage.face_ref(reference).orientation.orientation(),
        Orientation::Reversed
    );
    assert!(storage.mark_removed(vertex.into()));
    assert!(storage.mark_removed_ref(reference.into()));
    assert!(storage.is_removed(vertex));
    assert!(storage.is_removed(reference));
    assert_eq!(storage.nb_active_vertices(), 0);
    assert_eq!(storage.nb_active_face_refs(), 0);
}
#[test]
fn Storage_SetRemoved_TransitionsUpdateActiveCounts() {
    let mut storage = Storage::new();
    let vertex = storage.append_vertex();
    let reference = storage.append_face_ref();
    assert_eq!(storage.nb_active_vertices(), 1);
    assert_eq!(storage.nb_active_face_refs(), 1);
    for removed in [true, true, false, false] {
        storage.set_removed(vertex, removed);
        storage.set_removed(reference, removed);
        assert_eq!(storage.is_removed(vertex), removed);
        assert_eq!(storage.is_removed(reference), removed);
        assert_eq!(storage.nb_active_vertices(), u32::from(!removed));
        assert_eq!(storage.nb_active_face_refs(), u32::from(!removed));
    }
}
fn inc_storage_vertex(storage: &mut Storage, x: f64, tolerance: f64) -> VertexId {
    let id = storage.append_vertex();
    storage.change_vertex(id).point = Point3::new(x, 0.0, 0.0);
    storage.change_vertex(id).tolerance = tolerance;
    id
}
fn inc_storage_edge(storage: &mut Storage, start: VertexId, end: VertexId) -> EdgeId {
    let id = storage.append_edge();
    let first = storage.append_vertex_ref();
    let last = storage.append_vertex_ref();
    storage.change_vertex_ref(first).parent_edge_id = id;
    storage.change_vertex_ref(first).child_vertex_id = start;
    storage.change_vertex_ref(last).parent_edge_id = id;
    storage.change_vertex_ref(last).child_vertex_id = end;
    storage.change_edge(id).start_vertex_ref_id = first;
    storage.change_edge(id).end_vertex_ref_id = last;
    storage.change_edge(id).tolerance = 0.0;
    id
}
#[test]
fn Relations_RebindVertexSkipsRemovedParentEdge() {
    let mut storage = Storage::new();
    let old = storage.append_vertex();
    let end = storage.append_vertex();
    let new = storage.append_vertex();
    let edge = inc_storage_edge(&mut storage, old, end);
    let reference = storage.edge(edge).start_vertex_ref_id;
    storage.rebuild_derived_relations();
    assert_eq!(storage.vertex_relations(old).edge_ids.len(), 1);
    assert_eq!(storage.vertex_relations(new).edge_ids.len(), 0);
    storage.set_removed(edge, true);
    storage.rebind_vertex_edge(old, new, edge, VertexRefId::default());
    storage.rebind_vertex_ref(reference, old, new);
    assert_eq!(storage.vertex_ref(reference).child_vertex_id, old);
    assert_eq!(storage.vertex_relations(old).edge_ids.len(), 1);
    assert_eq!(storage.vertex_relations(new).edge_ids.len(), 0);
}
#[test]
fn Storage_RebuildDerivedRelations_RecountsRefStoreNbActive() {
    let mut storage = Storage::new();
    let first = storage.append_vertex();
    let last = storage.append_vertex();
    let edge = inc_storage_edge(&mut storage, first, last);
    let reference = storage.edge(edge).start_vertex_ref_id;
    storage.rebuild_derived_relations();
    let old = storage.nb_active_vertex_refs();
    assert!(old > 0);
    assert!(storage.mark_removed_ref(reference.into()));
    assert_eq!(storage.nb_active_vertex_refs(), old - 1);
    storage.rebuild_derived_relations();
    assert_eq!(storage.nb_active_vertex_refs(), old - 1);
}
fn inc_storage_use(
    storage: &mut Storage,
    wire: WireId,
    first: VertexId,
    last: VertexId,
) -> CoEdgeId {
    let edge = inc_storage_edge(storage, first, last);
    storage.create_co_edge_use(wire, edge, FaceId::default(), ParityOrientation::FORWARD)
}
#[test]
fn CanonicalizeWireCoEdgeOrderStatus_ReordersExactConnectedWire() {
    let mut storage = Storage::new();
    let wire = storage.append_wire();
    let a = inc_storage_vertex(&mut storage, 0.0, 1e-7);
    let b = inc_storage_vertex(&mut storage, 1.0, 1e-7);
    let c = inc_storage_vertex(&mut storage, 2.0, 1e-7);
    let bc = inc_storage_use(&mut storage, wire, b, c);
    let ab = inc_storage_use(&mut storage, wire, a, b);
    assert!(!storage.validate_wire_co_edge_order(wire));
    assert_eq!(
        storage.canonicalize_wire_co_edge_order_status(wire),
        rustycad_tkbrep::brep_graph_inc::WireCoEdgeOrderStatus::Reordered
    );
    assert_eq!(storage.wire_relations(wire).co_edge_ids, vec![ab, bc]);
    assert!(storage.validate_wire_co_edge_order(wire));
}
#[test]
fn CanonicalizeWireCoEdgeOrderStatus_UsesVertexTolerance() {
    let mut storage = Storage::new();
    let wire = storage.append_wire();
    let a = inc_storage_vertex(&mut storage, 0.0, 1e-7);
    let b = inc_storage_vertex(&mut storage, 1.0, 1e-2);
    let c = inc_storage_vertex(&mut storage, 1.005, 1e-2);
    let d = inc_storage_vertex(&mut storage, 2.0, 1e-7);
    let ab = inc_storage_use(&mut storage, wire, a, b);
    let cd = inc_storage_use(&mut storage, wire, c, d);
    assert!(!storage.validate_wire_co_edge_order(wire));
    assert_eq!(
        storage.canonicalize_wire_co_edge_order_status(wire),
        rustycad_tkbrep::brep_graph_inc::WireCoEdgeOrderStatus::ToleranceOrdered
    );
    assert_eq!(storage.wire_relations(wire).co_edge_ids, vec![ab, cd]);
}
#[test]
fn CanonicalizeWireCoEdgeOrderStatus_PartialPreservesDisconnectedRuns() {
    let mut storage = Storage::new();
    let wire = storage.append_wire();
    let a = inc_storage_vertex(&mut storage, 0.0, 1e-7);
    let b = inc_storage_vertex(&mut storage, 1.0, 1e-7);
    let c = inc_storage_vertex(&mut storage, 2.0, 1e-7);
    let x = inc_storage_vertex(&mut storage, 10.0, 1e-7);
    let y = inc_storage_vertex(&mut storage, 11.0, 1e-7);
    let bc = inc_storage_use(&mut storage, wire, b, c);
    let xy = inc_storage_use(&mut storage, wire, x, y);
    let ab = inc_storage_use(&mut storage, wire, a, b);
    assert_eq!(
        storage.canonicalize_wire_co_edge_order_status(wire),
        rustycad_tkbrep::brep_graph_inc::WireCoEdgeOrderStatus::Partial
    );
    assert_eq!(storage.wire_relations(wire).co_edge_ids, vec![ab, bc, xy]);
}

fn inc_storage_orientation(
    storage: &Storage,
    reference: FaceRefId,
    expected: Orientation,
    reversed: bool,
) {
    let orientation = storage.face_ref(reference).orientation;
    assert_eq!(orientation.orientation(), expected);
    assert_eq!(orientation.is_reversed, reversed);
}
