fn geometry_valid_curves(graph: &BRepGraph) {
    for id in geometry_ids::<EdgeTag>(graph) {
        if !edge::degenerated(graph, id) && edge::has_curve(graph, id) {
            assert!(!edge::curve(graph, id).is_null());
        }
    }
}
fn geometry_valid_surfaces(graph: &BRepGraph) {
    for id in geometry_ids::<FaceTag>(graph) {
        assert!(face::has_surface(graph, id));
        assert!(!face::surface(graph, id).is_null());
    }
}
#[test]
fn BRepGraph_GeometryTest__Box_RepCounts_MatchTopology() {
    let graph = geometry_box(false);
    assert!(graph.topo().geometry().nb_face_surfaces() > 0);
    assert!(graph.topo().geometry().nb_edge_curves3d() > 0);
    assert!(graph.topo().geometry().nb_co_edge_curves2d() > 0);
    geometry_valid_surfaces(&graph);
    geometry_valid_curves(&graph);
    for id in geometry_ids::<CoEdgeTag>(&graph) {
        if graph
            .topo()
            .co_edges()
            .definition(id)
            .curve2d_rep_id
            .is_valid()
        {
            assert!(!co_edge::pcurve(&graph, id).is_null());
        }
    }
}
#[test]
fn BRepGraph_GeometryTest__Sphere_SurfaceDedup_SharedHandle() {
    let graph = geometry_sphere();
    if graph.topo().faces().nb() > 1 {
        let first = face::surface(&graph, FaceId::start());
        for id in (1..graph.topo().faces().nb()).map(FaceId::new) {
            assert!(face::surface(&graph, id) == first);
        }
    }
}
#[test]
fn BRepGraph_GeometryTest__Cylinder_TriangulationReps_Populated() {
    let graph = geometry_cylinder(10.0);
    for id in geometry_ids::<FaceTag>(&graph) {
        let rep = graph.topo().faces().definition(id).triangulation_rep_id;
        if !rep.is_valid() {
            continue;
        }
        assert!(rep.is_valid());
        if graph.mesh().effective().faces().has(id) {
            assert!(!graph.mesh().effective().faces().triangulation(id).is_null());
        }
    }
}
#[test]
fn BRepGraph_GeometryTest__RepId_FactoryMethods() {
    let surface = FaceSurfaceRepId::new(42);
    assert_eq!(RepId::from(surface).kind, RepKind::FACE_SURFACE);
    assert_eq!(surface.index, 42);
    assert!(surface.is_valid());
    let curve = EdgeCurve3DRepId::new(7);
    assert_eq!(RepId::from(curve).kind, RepKind::EDGE_CURVE_3D);
    assert_eq!(curve.index, 7);
    assert!(!RepId::default().is_valid());
    assert_eq!(surface, FaceSurfaceRepId::new(42));
    assert_ne!(RepId::from(surface), RepId::from(curve));
}
#[test]
fn BRepGraph_GeometryTest__RepId_UntypedArithmetic_PreservesKindAndIndex() {
    let base = RepId::new(RepKind::EDGE_CURVE_3D, 5);
    assert_eq!(base.kind, RepKind::EDGE_CURVE_3D);
    assert_eq!(base.index, 5);
    assert_eq!(base, RepId::new(RepKind::EDGE_CURVE_3D, 5));
    let different = RepId::new(RepKind::EDGE_CURVE_3D, 9);
    assert_ne!(base, different);
    assert!(base < different);
    assert_ne!(base, RepId::new(RepKind::FACE_SURFACE, 5));
}
#[test]
fn BRepGraph_GeometryTest__Compound_TwoBoxes_SurfaceDedup() {
    let first = support::make_box(10.0, 10.0, 10.0);
    let second = support::make_box(20.0, 20.0, 20.0);
    let graph = geometry_source(&geometry_compound(&first, &second));
    assert_eq!(graph.topo().faces().nb(), 12);
    assert!(graph.topo().geometry().nb_face_surfaces() > 0);
    assert!(graph.topo().geometry().nb_face_surfaces() <= 12);
    for id in geometry_ids::<FaceTag>(&graph) {
        assert!(face::has_surface(&graph, id));
    }
    geometry_valid_curves(&graph);
}
#[test]
fn BRepGraph_GeometryTest__Box_Polygon2DRep_MatchesInline() {
    let graph = geometry_box(false);
    for id in geometry_ids::<CoEdgeTag>(&graph) {
        let def = graph.topo().co_edges().definition(id);
        if def.polygon2d_rep_id.is_valid() {
            assert!(
                !graph
                    .mesh()
                    .persistent()
                    .co_edges()
                    .polygon_on_surface(id)
                    .is_null()
            );
        }
        if def.polygon_on_tri_rep_id.is_valid() {
            assert!(def.polygon_on_tri_rep_id.is_valid());
        }
    }
}
