//! Native topology cases for same-domain processing when an adjacent face lacks a p-curve.
mod common;
use common::{face_count, rectangle_wire};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Shape, Shell, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis3, Direction2, Plane3, Point2, Vector2};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPISewing,
};

#[test]
fn missing_source_pcurve_does_not_abort_transformation() {
    let fixture = missing_pcurve_transformation_fixture();
    assert!(!fixture.shell.is_null());
    assert!(
        !BRepTool::curve_on_face(&fixture.missing_edge, &fixture.reference_face)
            .unwrap()
            .is_stored
    );
    assert_eq!(face_count(&fixture.shell), 2);

    let unifier = ShapeUpgradeUnifySameDomain::new(&fixture.shell, false, true, false);
    unifier.allow_internal_edges(true);
    unifier.build().unwrap();

    let result = unifier.shape();
    assert!(!result.is_null());
    assert_eq!(face_count(&result), 2);
}

#[test]
fn missing_pcurve_does_not_abort_unrelated_planar_merge() {
    let fixture = missing_pcurve_shell_fixture();
    assert!(!fixture.shell.is_null());
    assert!(
        !BRepTool::curve_on_face(&fixture.missing_edge, &fixture.reference_face)
            .unwrap()
            .is_stored
    );
    assert!(
        BRepTool::curve_on_face(&fixture.valid_pcurve_edge, &fixture.reference_face)
            .unwrap()
            .is_stored
    );
    add_planar_pair_to_shell(&fixture.shell)
        .expect("planar coplanar pair should be added to cylinder shell");
}

struct MissingPcurveFixture {
    shell: Shape,
    reference_face: rustycad_tkbrep::topods::Face,
    missing_edge: rustycad_tkbrep::topods::Edge,
    valid_pcurve_edge: rustycad_tkbrep::topods::Edge,
}

fn missing_pcurve_shell_fixture() -> MissingPcurveFixture {
    let (surface, vertices) =
        cylinder_vertices(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.5, -1.0), (0.5, 1.0)]);
    let builder = BRepBuilder::default();
    let common = cylinder_edge(
        &builder,
        &surface,
        &vertices[0],
        &vertices[1],
        (0.0, 0.0),
        (1.0, 0.0),
        true,
    );
    let missing = cylinder_edge(
        &builder,
        &surface,
        &vertices[1],
        &vertices[2],
        (1.0, 0.0),
        (1.0, 1.0),
        false,
    );
    let reverse_bc = cylinder_edge(
        &builder,
        &surface,
        &vertices[2],
        &vertices[1],
        (1.0, 1.0),
        (1.0, 0.0),
        true,
    );
    let bd = cylinder_edge(
        &builder,
        &surface,
        &vertices[1],
        &vertices[3],
        (1.0, 0.0),
        (0.5, -1.0),
        true,
    );
    let da = cylinder_edge(
        &builder,
        &surface,
        &vertices[3],
        &vertices[0],
        (0.5, -1.0),
        (0.0, 0.0),
        true,
    );
    let ae = cylinder_edge(
        &builder,
        &surface,
        &vertices[0],
        &vertices[4],
        (0.0, 0.0),
        (0.5, 1.0),
        true,
    );
    let eb = cylinder_edge(
        &builder,
        &surface,
        &vertices[4],
        &vertices[1],
        (0.5, 1.0),
        (1.0, 0.0),
        true,
    );
    let first_wire = cylinder_wire(
        &builder,
        &[common.clone(), missing.clone(), reverse_bc, bd.clone(), da],
    );
    let mut reversed_common = common.clone();
    reversed_common.reverse();
    let second_wire = cylinder_wire(&builder, &[reversed_common, ae, eb]);
    let first_face = cylinder_face(&builder, &surface, &first_wire);
    let second_face = cylinder_face(&builder, &surface, &second_wire);
    let shell = cylinder_shell(&builder, &[&first_face, &second_face]);
    MissingPcurveFixture {
        shell,
        reference_face: first_face,
        missing_edge: missing,
        valid_pcurve_edge: bd,
    }
}

fn missing_pcurve_transformation_fixture() -> MissingPcurveFixture {
    let (surface, vertices) = cylinder_vertices(&[
        (0.0, 0.0),
        (0.5, 0.0),
        (0.5, 1.0),
        (0.0, 1.0),
        (1.0, 0.0),
        (1.0, 1.0),
    ]);
    let builder = BRepBuilder::default();
    let ab = cylinder_edge(
        &builder,
        &surface,
        &vertices[0],
        &vertices[1],
        (0.0, 0.0),
        (0.5, 0.0),
        true,
    );
    let bc = cylinder_edge(
        &builder,
        &surface,
        &vertices[1],
        &vertices[2],
        (0.5, 0.0),
        (0.5, 1.0),
        true,
    );
    let cd = cylinder_edge(
        &builder,
        &surface,
        &vertices[2],
        &vertices[3],
        (0.5, 1.0),
        (0.0, 1.0),
        true,
    );
    let da = cylinder_edge(
        &builder,
        &surface,
        &vertices[3],
        &vertices[0],
        (0.0, 1.0),
        (0.0, 0.0),
        true,
    );
    let missing = cylinder_edge(
        &builder,
        &surface,
        &vertices[1],
        &vertices[4],
        (0.5, 0.0),
        (1.0, 0.0),
        false,
    );
    let ef = cylinder_edge(
        &builder,
        &surface,
        &vertices[4],
        &vertices[5],
        (1.0, 0.0),
        (1.0, 1.0),
        true,
    );
    let fc = cylinder_edge(
        &builder,
        &surface,
        &vertices[5],
        &vertices[2],
        (1.0, 1.0),
        (0.5, 1.0),
        true,
    );
    let left_wire = cylinder_wire(&builder, &[ab, bc.clone(), cd, da]);
    let mut reversed_bc = bc.clone();
    reversed_bc.reverse();
    let right_wire = cylinder_wire(&builder, &[missing.clone(), ef, fc, reversed_bc]);
    let first_face = cylinder_face(&builder, &surface, &left_wire);
    let second_face = cylinder_face(&builder, &surface, &right_wire);
    let shell = cylinder_shell(&builder, &[&first_face, &second_face]);
    MissingPcurveFixture {
        shell,
        reference_face: first_face,
        missing_edge: missing,
        valid_pcurve_edge: bc,
    }
}

fn cylinder_vertices(
    parameters: &[(f64, f64)],
) -> (
    Handle<dyn GeomSurface>,
    Vec<rustycad_tkbrep::topods::Vertex>,
) {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0).unwrap()).cast();
    let builder = BRepBuilder::default();
    let vertices = parameters
        .iter()
        .map(|&(u, v)| {
            let mut vertex = rustycad_tkbrep::topods::Vertex::default();
            builder
                .make_vertex_at(
                    &mut vertex,
                    surface.get().unwrap().value(u, v).unwrap(),
                    precision::confusion(),
                )
                .unwrap();
            vertex
        })
        .collect();
    (surface, vertices)
}

fn cylinder_edge(
    builder: &BRepBuilder,
    surface: &Handle<dyn GeomSurface>,
    first: &rustycad_tkbrep::topods::Vertex,
    last: &rustycad_tkbrep::topods::Vertex,
    uv_first: (f64, f64),
    uv_last: (f64, f64),
    with_pcurve: bool,
) -> rustycad_tkbrep::topods::Edge {
    let edge = BRepBuilderAPIMakeEdge::from_vertices(first, last)
        .unwrap()
        .edge()
        .unwrap();
    if with_pcurve {
        let start = Point2::new(uv_first.0, uv_first.1);
        let end = Point2::new(uv_last.0, uv_last.1);
        let curve: Handle<dyn Geom2dCurve> = Handle::new(pcurve_line(start, end)).cast();
        builder
            .update_edge_pcurve(
                &edge,
                &curve,
                surface,
                &Location::default(),
                precision::confusion(),
                None,
            )
            .unwrap();
        let range = Vector2::between(start, end).magnitude();
        builder
            .range_on_surface(&edge, surface, &Location::default(), 0.0, range)
            .unwrap();
    }
    edge
}

fn cylinder_wire(builder: &BRepBuilder, edges: &[rustycad_tkbrep::topods::Edge]) -> Wire {
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for edge in edges {
        builder.add(&mut wire, &edge.clone().into_shape()).unwrap();
    }
    wire.set_closed(true).unwrap();
    wire
}

fn cylinder_face(
    builder: &BRepBuilder,
    surface: &Handle<dyn GeomSurface>,
    wire: &Wire,
) -> rustycad_tkbrep::topods::Face {
    let mut face = rustycad_tkbrep::topods::Face::default();
    builder
        .make_face_on_surface(&mut face, surface, precision::confusion())
        .unwrap();
    builder.add(&mut face, &wire.clone().into_shape()).unwrap();
    face
}

fn cylinder_shell(builder: &BRepBuilder, faces: &[&rustycad_tkbrep::topods::Face]) -> Shape {
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    for face in faces {
        builder
            .add(&mut shell, &(*face).clone().into_shape())
            .unwrap();
    }
    shell.into_shape()
}

fn add_planar_pair_to_shell(shell: &Shape) -> Result<(), rustycad_tkernel::standard::Failure> {
    let planar_pair = sewn_planar_pair();
    let mut result = shell.clone();
    for face in
        rustycad_tkbrep::topexp::Explorer::new(&planar_pair, ShapeEnum::Face, ShapeEnum::Shape)?
    {
        BRepBuilder::default().add(&mut result, &face)?;
    }
    let unifier = ShapeUpgradeUnifySameDomain::new(&result, false, true, false);
    unifier.allow_internal_edges(true);
    unifier.build()?;
    assert_eq!(face_count(&unifier.shape()), 3);
    Ok(())
}

fn pcurve_line(first: Point2, last: Point2) -> Geom2dLine {
    Geom2dLine::from_point(
        first,
        Direction2::from_vector(Vector2::between(first, last)).unwrap(),
    )
}

fn sewn_planar_pair() -> Shape {
    let first = planar_face(3.0, 4.0);
    let second = planar_face(4.0, 5.0);
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.add(&first).unwrap();
    sewing.add(&second).unwrap();
    sewing.perform(&ProgressRange::default()).unwrap();
    sewing.sewed_shape().clone()
}

fn planar_face(x_min: f64, x_max: f64) -> Shape {
    let wire = rectangle_wire(x_min, x_max, 0.0, 1.0);
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire, true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}
