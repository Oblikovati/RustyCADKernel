#[test]
fn BRepGraph_GeometryTest__Sphere_AllFaces_SameSurface() {
    let graph = geometry_sphere();
    assert!(graph.topo().faces().nb() >= 1);
    assert!(face::has_surface(&graph, FaceId::start()));
    let first = face::surface(&graph, FaceId::start());
    assert!(!first.is_null());
    for id in (1..graph.topo().faces().nb()).map(FaceId::new) {
        assert!(face::has_surface(&graph, id));
        assert!(face::surface(&graph, id) == first);
    }
}
#[test]
fn BRepGraph_GeometryTest__Sphere_AllFacesShareSurface() {
    let graph = geometry_sphere();
    assert!(face::has_surface(&graph, FaceId::start()));
    let first = face::surface(&graph, FaceId::start());
    assert!(!first.is_null());
    let count = geometry_ids::<FaceTag>(&graph)
        .into_iter()
        .filter(|id| face::has_surface(&graph, *id) && face::surface(&graph, *id) == first)
        .count();
    assert_eq!(count, graph.topo().faces().nb() as usize);
}
#[test]
fn BRepGraph_GeometryTest__Box_Curve3d_ValidForAll12Edges() {
    let graph = geometry_box(false);
    assert_eq!(graph.topo().edges().nb(), 12);
    for id in geometry_ids::<EdgeTag>(&graph) {
        assert!(edge::has_curve(&graph, id), "edge {id:?}, expected curve");
    }
}
#[test]
fn BRepGraph_GeometryTest__Box_AllEdgesHaveCurve3d() {
    let graph = geometry_box(false);
    for id in geometry_ids::<EdgeTag>(&graph) {
        assert!(edge::has_curve(&graph, id), "edge {id:?}, expected curve");
    }
}
#[test]
fn BRepGraph_GeometryTest__EdgeDef_Curve3d_IsNotNull() {
    let graph = geometry_box(false);
    for id in geometry_ids::<EdgeTag>(&graph) {
        assert!(edge::has_curve(&graph, id), "edge {id:?}, expected curve");
    }
}
#[test]
fn BRepGraph_GeometryTest__FaceDef_Surface_IsNotNull() {
    let graph = geometry_box(false);
    for id in geometry_ids::<FaceTag>(&graph) {
        assert!(
            face::has_surface(&graph, id),
            "face {id:?}, expected surface"
        );
    }
}
#[test]
fn BRepGraph_GeometryTest__SameDomainFaces_SimpleBox_Empty() {
    let graph = geometry_box(false);
    for id in geometry_ids::<FaceTag>(&graph) {
        assert!(geometry_same_domain(&graph, id).is_empty());
    }
}
#[test]
fn BRepGraph_GeometryTest__SameDomainFaces_SharedSurfaceAcrossOwnedUses() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_pln(Plane3::default())).into_cast();
    let first = geometry_shared_face(&mut graph, &surface);
    let second = geometry_shared_face(&mut graph, &surface);
    assert!(first.is_valid());
    assert!(second.is_valid());
    assert_ne!(
        graph.topo().faces().definition(first).surface_rep_id,
        graph.topo().faces().definition(second).surface_rep_id
    );
    assert_eq!(geometry_same_domain(&graph, first), vec![second]);
    assert_eq!(geometry_same_domain(&graph, second), vec![first]);
}
#[test]
fn BRepGraph_GeometryTest__CompoundWithMovedChild_SharedSolidDef() {
    let shape = support::make_box(10.0, 10.0, 10.0);
    let location = Location::new(Transform3::translation_vector(Vector3::new(
        100.0, 0.0, 0.0,
    )));
    let moved = shape.moved(&location, true).unwrap();
    let compound = geometry_compound(&shape, &moved);
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let result = graph.shapes_mut().add(&compound).unwrap();
    assert!(result.is_ok());
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().solids().nb(), 1);
    let id = CompoundId::new(result.topology_root.index);
    geometry_child_placements(&graph, id);
}
#[test]
fn BRepGraph_GeometryTest__FaceDef_Triangulation_NullForAnalyticNoCrash() {
    let graph = geometry_box(false);
    for id in geometry_ids::<FaceTag>(&graph) {
        assert!(!graph.mesh().effective().faces().has(id));
    }
}

fn geometry_child_placements(graph: &BRepGraph, id: CompoundId) {
    let refs = &graph.topo().compounds().relations(id).child_ref_ids;
    assert_eq!(refs.len(), 2);
    let queries = graph.refs().generic();
    assert!(queries.local_location(refs[0].into()).is_identity());
    assert!(!queries.local_location(refs[1].into()).is_identity());
}
fn geometry_shared_face(graph: &mut BRepGraph, surface: &Handle<dyn GeomSurface>) -> FaceId {
    let editor = graph.editor();
    let mut faces = editor.faces();
    faces.add(surface, WireId::default(), &[], 1e-7)
}
