#[test]
fn ConnectWires_InternalOnlyWireAfterManifoldWire() {
    let f = boundary_fixture();
    check_boundary_wires(vec![f.loop_wire, f.internal], (2, 4, 1, 0, true));
}
#[test]
fn ConnectWires_InternalOnlyWireBeforeManifoldWire() {
    let f = boundary_fixture();
    check_boundary_wires(vec![f.internal, f.loop_wire], (2, 4, 1, 0, true));
}
#[test]
fn ConnectWires_OnlyNonManifoldWires() {
    let f = boundary_fixture();
    check_boundary_wires(vec![f.internal, f.external], (2, 2, 1, 1, false));
}
#[test]
fn ConnectWires_EmptyWireBeforeValidWires() {
    let f = boundary_fixture();
    check_boundary_wires(vec![f.empty, f.internal, f.loop_wire], (2, 4, 1, 0, true));
}
#[test]
fn ConnectWires_OnlyEmptyWire() {
    let f = boundary_fixture();
    check_boundary_wires(vec![f.empty], (0, 0, 0, 0, false));
}
#[test]
fn ConnectWires_MixedManifoldAndInternalEdges() {
    let f = boundary_fixture();
    check_boundary_wires(vec![boundary_wire(&f.edges[..4])], (1, 4, 1, 0, false));
}
