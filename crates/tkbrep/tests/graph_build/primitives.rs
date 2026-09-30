#[test]
fn BRepGraph_BuildTest__Sphere_IsNotEmpty() {
    let mut maker = BRepPrimAPIMakeSphere::new(10.0).unwrap();
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let _graph = build_graph(&shape);
}
#[test]
fn BRepGraph_BuildTest__Sphere_DefCounts_MatchTopExp() {
    let mut maker = BRepPrimAPIMakeSphere::new(10.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert_build_counts(
        &graph,
        &shape,
        &[
            ShapeEnum::Solid,
            ShapeEnum::Shell,
            ShapeEnum::Face,
            ShapeEnum::Wire,
            ShapeEnum::Edge,
            ShapeEnum::Vertex,
        ],
    );
}
#[test]
fn BRepGraph_BuildTest__Sphere_SurfaceType() {
    let mut maker = BRepPrimAPIMakeSphere::new(5.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert!(graph.topo().faces().nb() >= 1);
    assert!(has_surface_type(&graph, "Geom_SphericalSurface"));
}
#[test]
fn BRepGraph_BuildTest__Sphere_HasDegenerateEdges() {
    let mut maker = BRepPrimAPIMakeSphere::new(8.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert!(build_degenerate_count(&graph) >= 1);
}
#[test]
fn BRepGraph_BuildTest__Cylinder_IsNotEmpty() {
    let mut maker = BRepPrimAPIMakeCylinder::new(5.0, 20.0);
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let _graph = build_graph(&shape);
}
#[test]
fn BRepGraph_BuildTest__Cylinder_DefCounts_MatchTopExp() {
    let mut maker = BRepPrimAPIMakeCylinder::new(5.0, 20.0);
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert_build_counts(
        &graph,
        &shape,
        &[ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex],
    );
}
#[test]
fn BRepGraph_BuildTest__Cylinder_SurfaceType() {
    let mut maker = BRepPrimAPIMakeCylinder::new(5.0, 20.0);
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert!(has_surface_type(&graph, "Geom_CylindricalSurface"));
}
#[test]
fn BRepGraph_BuildTest__Cone_IsNotEmpty() {
    let mut maker = BRepPrimAPIMakeCone::new(10.0, 0.0, 15.0).unwrap();
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let _graph = build_graph(&shape);
}
#[test]
fn BRepGraph_BuildTest__Cone_DefCounts_MatchTopExp() {
    let mut maker = BRepPrimAPIMakeCone::new(10.0, 0.0, 15.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert_build_counts(
        &graph,
        &shape,
        &[ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex],
    );
}
#[test]
fn BRepGraph_BuildTest__Cone_SurfaceType() {
    let mut maker = BRepPrimAPIMakeCone::new(10.0, 5.0, 15.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert!(has_surface_type(&graph, "Geom_ConicalSurface"));
}
#[test]
fn BRepGraph_BuildTest__Cone_HasDegenerateEdge() {
    let mut maker = BRepPrimAPIMakeCone::new(10.0, 0.0, 15.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert!(build_degenerate_count(&graph) >= 1);
}
#[test]
fn BRepGraph_BuildTest__Torus_IsNotEmpty() {
    let mut maker = BRepPrimAPIMakeTorus::new(20.0, 5.0).unwrap();
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let _graph = build_graph(&shape);
}
#[test]
fn BRepGraph_BuildTest__Torus_DefCounts_MatchTopExp() {
    let mut maker = BRepPrimAPIMakeTorus::new(20.0, 5.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert_build_counts(
        &graph,
        &shape,
        &[ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex],
    );
}
#[test]
fn BRepGraph_BuildTest__Torus_SurfaceType() {
    let mut maker = BRepPrimAPIMakeTorus::new(20.0, 5.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert!(has_surface_type(&graph, "Geom_ToroidalSurface"));
}
#[test]
fn BRepGraph_BuildTest__Wedge_IsNotEmpty() {
    let mut maker = BRepPrimAPIMakeWedge::with_ltx(10.0, 10.0, 10.0, 5.0).unwrap();
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let _graph = build_graph(&shape);
}
#[test]
fn BRepGraph_BuildTest__Wedge_DefCounts_MatchTopExp() {
    let mut maker = BRepPrimAPIMakeWedge::with_ltx(10.0, 10.0, 10.0, 5.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert_build_counts(
        &graph,
        &shape,
        &[
            ShapeEnum::Solid,
            ShapeEnum::Shell,
            ShapeEnum::Face,
            ShapeEnum::Wire,
            ShapeEnum::Edge,
            ShapeEnum::Vertex,
        ],
    );
}
#[test]
fn BRepGraph_BuildTest__Wedge_AllPlanarSurfaces() {
    let mut maker = BRepPrimAPIMakeWedge::with_ltx(10.0, 10.0, 10.0, 5.0).unwrap();
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert_planar_surfaces(&graph);
}
