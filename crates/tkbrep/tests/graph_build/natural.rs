use rustycad_tkbrep::brep_graph::{
    iterator::WireIterator, refs_iterator::CoEdgesOfWire, tool::co_edge, validate,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3};

fn natural_frame() -> Axis3 {
    Axis3::from_axis2(
        Axis2::new(Point3::default(), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap(),
    )
}
fn no_wire_natural_face(surface: &Handle<dyn GeomSurface>) -> topods::Face {
    let builder = BRepBuilder::default();
    let mut face = topods::Face::default();
    builder
        .make_face_on_surface(&mut face, surface, precision::confusion())
        .unwrap();
    builder.natural_restriction(&face, true).unwrap();
    face
}
fn natural_trim(
    surface: Handle<dyn GeomSurface>,
    u: (f64, f64),
    v: (f64, f64),
    senses: (bool, bool),
) -> Handle<dyn GeomSurface> {
    Handle::new(GeomRectangularTrimmedSurface::new(&surface, u, v, senses).unwrap()).into_cast()
}
fn assert_natural_counts(graph: &BRepGraph) {
    assert!(graph.topo().wires().nb() > 0);
    assert!(graph.topo().edges().nb() > 0);
    assert!(graph.topo().vertices().nb() > 0);
}
fn assert_finite_natural_topology(surface: Handle<dyn GeomSurface>) {
    let face = no_wire_natural_face(&surface);
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let result = graph.shapes_mut().add(&face).unwrap();
    assert!(result.is_ok());
    assert_eq!(graph.topo().faces().nb(), 1);
    assert!(face::nb_wires(&graph, FaceId::start()) > 0);
    assert_natural_counts(&graph);
}
fn assert_face_wires(graph: &BRepGraph) {
    assert!(graph.topo().faces().nb() >= 1);
    let mut faces = FaceIterator::new(graph);
    while faces.more() {
        assert!(face::nb_wires(graph, faces.current_id()) > 0);
        faces.next();
    }
}
#[test]
fn BRepGraph_BuildTest__NaturalBound_FiniteNoWireNaturalFaces_SynthesizeTopology() {
    let frame = natural_frame();
    assert_finite_natural_topology(
        Handle::new(GeomSphericalSurface::new(frame, 10.0).unwrap()).into_cast(),
    );
    assert_finite_natural_topology(
        Handle::new(GeomToroidalSurface::new(frame, 10.0, 3.0).unwrap()).into_cast(),
    );
    let cylinder = Handle::new(GeomCylindricalSurface::new(frame, 5.0).unwrap()).into_cast();
    assert_finite_natural_topology(natural_periodic_trim(cylinder));
    let cone = Handle::new(GeomConicalSurface::new(frame, 0.2, 5.0).unwrap()).into_cast();
    assert_finite_natural_topology(natural_periodic_trim(cone));
    let plane = Handle::new(GeomPlane::new(frame)).into_cast();
    assert_finite_natural_topology(natural_trim(plane, (0.0, 3.0), (0.0, 4.0), (true, true)));
}
#[test]
fn BRepGraph_BuildTest__NaturalBound_Sphere_HasWiresAndEdges() {
    let mut maker = BRepPrimAPIMakeSphere::new(10.0).unwrap();
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let graph = build_graph(&shape);
    assert_face_wires(&graph);
    assert_natural_counts(&graph);
}
#[test]
fn BRepGraph_BuildTest__NaturalBound_Cylinder_HasWiresAndEdges() {
    let mut maker = BRepPrimAPIMakeCylinder::new(5.0, 10.0);
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let graph = build_graph(&shape);
    assert_face_wires(&graph);
}
#[test]
fn BRepGraph_BuildTest__NaturalBound_Cone_HasWiresAndEdges() {
    let mut maker = BRepPrimAPIMakeCone::new(5.0, 2.0, 10.0).unwrap();
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let graph = build_graph(&shape);
    assert_face_wires(&graph);
}
#[test]
fn BRepGraph_BuildTest__NaturalBound_Torus_HasWiresAndEdges() {
    let mut maker = BRepPrimAPIMakeTorus::new(10.0, 3.0).unwrap();
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let graph = build_graph(&shape);
    assert_face_wires(&graph);
}
#[test]
fn BRepGraph_BuildTest__NaturalBound_InfinitePlane_ProducesWarnings() {
    // Infinite UV bounds preserve the unbounded face with the original warning.
    let surface = Handle::new(GeomPlane::new(natural_frame())).into_cast();
    let face = no_wire_natural_face(&surface);
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let result = graph.shapes_mut().add(&face).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.status, AddStatus::SuccessWithWarnings);
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb_active(), 1);
    assert_eq!(face::nb_wires(&graph, FaceId::start()), 0);
    let audit = validate::perform_with_options(&graph, &validate::Options::audit());
    assert!(audit.is_valid());
    assert_eq!(audit.nb_issues(validate::Severity::Error), 0);
    assert_eq!(audit.nb_issues(validate::Severity::Warning), 1);
    assert!(
        !graph
            .shapes()
            .reconstruct(FaceId::start().into())
            .unwrap()
            .is_null()
    );
}
#[test]
fn BRepGraph_BuildTest__NaturalBound_PreservesExplicitWires() {
    // MakeSphere already supplies explicit seam edges and wires.
    let mut maker = BRepPrimAPIMakeSphere::new(10.0).unwrap();
    let shape = build_shape(&mut maker);
    assert!(maker.is_done());
    let graph = build_graph(&shape);
    assert_natural_counts(&graph);
}
#[test]
fn BRepGraph_BuildTest__NaturalBound_PlaneSynthesis_SkipsPCurves() {
    let plane = Handle::new(GeomPlane::new(natural_frame())).into_cast();
    let trimmed = natural_trim(plane, (-10.0, 10.0), (-10.0, 10.0), (false, false));
    let face = no_wire_natural_face(&trimmed);
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let result = graph.shapes_mut().add(&face).unwrap();
    assert!(result.is_ok());
    assert!(graph.topo().wires().nb() > 0);
    assert!(graph.topo().edges().nb() > 0);
    assert_generated_plane_pcurves(&graph);
}
fn assert_generated_plane_pcurves(graph: &BRepGraph) {
    let mut wires = WireIterator::new(graph);
    while wires.more() {
        for coedge in CoEdgesOfWire::new(graph, wires.current_id()) {
            assert!(!co_edge::has_pcurve(graph, coedge));
            assert!(
                co_edge::pcurve_adaptor(graph, coedge)
                    .unwrap()
                    .is_initialized()
            );
        }
        wires.next();
    }
}

fn natural_periodic_trim(surface: Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    natural_trim(
        surface,
        (0.0, 2.0 * std::f64::consts::PI),
        (0.0, 10.0),
        (true, true),
    )
}
