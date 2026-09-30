#[test]
fn ConnectEdges_InternalEdgeAfterManifoldEdges() {
    let f = boundary_fixture();
    check_boundary_edges(vec![
        f.edges[0].clone(),
        f.edges[1].clone(),
        f.edges[2].clone(),
        f.edges[3].clone(),
    ]);
}
#[test]
fn ConnectEdges_InternalEdgeBeforeManifoldEdges() {
    let f = boundary_fixture();
    check_boundary_edges(vec![
        f.edges[3].clone(),
        f.edges[0].clone(),
        f.edges[1].clone(),
        f.edges[2].clone(),
    ]);
}
#[test]
fn ConnectEdges_ExternalEdgeIsSkipped() {
    let f = boundary_fixture();
    check_boundary_edges(vec![
        f.edges[0].clone(),
        f.edges[1].clone(),
        f.edges[2].clone(),
        f.edges[4].clone(),
    ]);
}
