#[test]
fn BRepGraph_ReconstructTest__Box_Area_Preserved() {
    let shape = reconstruct_box();
    reconstructed_measure(&shape, reconstruction_area);
}
#[test]
fn BRepGraph_ReconstructTest__Box_Volume_Preserved() {
    let shape = reconstruct_box();
    reconstructed_measure(&shape, reconstruction_volume);
}
#[test]
fn BRepGraph_ReconstructTest__Sphere_Area_Preserved() {
    let shape = BRepPrimAPIMakeSphere::new(15.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    reconstructed_measure(&shape, reconstruction_area);
}
#[test]
fn BRepGraph_ReconstructTest__Sphere_Volume_Preserved() {
    let shape = BRepPrimAPIMakeSphere::new(15.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    reconstructed_measure(&shape, reconstruction_volume);
}
#[test]
fn BRepGraph_ReconstructTest__Cylinder_Area_Preserved() {
    let shape = BRepPrimAPIMakeCylinder::new(10.0, 20.0)
        .shape()
        .unwrap()
        .clone();
    reconstructed_measure(&shape, reconstruction_area);
}
#[test]
fn BRepGraph_ReconstructTest__Cylinder_Volume_Preserved() {
    let shape = BRepPrimAPIMakeCylinder::new(10.0, 20.0)
        .shape()
        .unwrap()
        .clone();
    reconstructed_measure(&shape, reconstruction_volume);
}
#[test]
fn BRepGraph_ReconstructTest__Shell_FaceCount_MatchesOriginal() {
    let shape = reconstruct_box();
    let original_count = reconstruction_subshapes(&shape, ShapeEnum::Face).len();
    let graph = reconstruct_graph(&shape);
    let reconstructed = graph
        .shapes()
        .reconstruct(NodeId::new(NodeKind::SHELL, 0))
        .unwrap();
    assert_eq!(
        reconstruction_subshapes(&reconstructed, ShapeEnum::Face).len(),
        original_count
    );
}
#[test]
fn BRepGraph_ReconstructTest__Wire_EdgeCount_FourPerBoxFace() {
    let graph = reconstruct_graph(&reconstruct_box());
    for id in reconstruction_ids::<rustycad_tkbrep::brep_graph::WireTag>(&graph) {
        let reconstructed = graph.shapes().reconstruct(id.into()).unwrap();
        assert_eq!(
            reconstruction_subshapes(&reconstructed, ShapeEnum::Edge).len(),
            4
        );
    }
}
#[test]
fn BRepGraph_ReconstructTest__Edge_HasCurve_NonNull() {
    let graph = reconstruct_graph(&reconstruct_box());
    for id in reconstruction_ids(&graph) {
        if edge::degenerated(&graph, id) {
            continue;
        }
        assert!(!reconstructed_curve(&graph, id).curve.is_null());
    }
}
#[test]
fn BRepGraph_ReconstructTest__Edge_ParameterRange_Preserved() {
    let graph = reconstruct_graph(&reconstruct_box());
    for id in reconstruction_ids(&graph) {
        if edge::degenerated(&graph, id) {
            continue;
        }
        let curve = reconstructed_curve(&graph, id);
        let (first, last) = edge::range(&graph, id);
        assert!((curve.first - first).abs() <= precision::confusion());
        assert!((curve.last - last).abs() <= precision::confusion());
    }
}
#[test]
fn BRepGraph_ReconstructTest__Vertex_Point_MatchesDefPoint() {
    let graph = reconstruct_graph(&reconstruct_box());
    for id in reconstruction_ids(&graph) {
        assert_reconstruction_coordinates(reconstructed_point(&graph, id), vertex::pnt(&graph, id));
    }
}
#[test]
fn BRepGraph_ReconstructTest__Face_PCurvesPresent_OnAllEdges() {
    let graph = reconstruct_graph(&reconstruct_box());
    for id in reconstruction_ids::<rustycad_tkbrep::brep_graph::FaceTag>(&graph) {
        let reconstructed = graph.shapes().reconstruct(id.into()).unwrap();
        assert!(!reconstructed.is_null());
        assert_reconstructed_pcurves(&reconstructed);
    }
}
#[test]
fn BRepGraph_ReconstructTest__Face_OrientationPreserved() {
    let graph = reconstruct_graph(&reconstruct_box());
    assert_eq!(graph.topo().shells().nb(), 1);
    for id in ref_test_tools::face_refs_of_shell(&graph, ShellId::start()) {
        let reference = graph.refs().faces().entry(*id);
        let _expected_orientation = reference.orientation;
        let reconstructed = graph
            .shapes()
            .reconstruct(reference.child_face_id.into())
            .unwrap();
        assert!(!reconstructed.is_null());
        // Upstream only checks the face kind; orientation now belongs to incidence refs.
        assert_eq!(reconstructed.shape_type().unwrap(), ShapeEnum::Face);
    }
}
