//! BRepFill_Generator: ruled shells through circles, squares and punctual sections.
use rustycad_tkbool::brep_fill::{BRepFillGenerator, ThruSectionErrorStatus};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeWire,
};

/// A closed circle wire of `radius` around the Z axis at height `z`.
fn circle_wire(z: f64, radius: f64) -> Result<Wire, Failure> {
    let frame = Axis2::new(Point3::new(0., 0., z), Direction3::Z)?;
    let edge = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(frame, radius)?)?.edge()?;
    BRepBuilderAPIMakeWire::from_edge(&edge)?.wire()
}

/// A closed square wire of side 2 at height `z`.
fn square_wire(z: f64, half: f64) -> Result<Wire, Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(-half, -half), (half, -half), (half, half), (-half, half)] {
        polygon.add_point(Point3::new(x, y, z))?;
    }
    polygon.close()?;
    polygon.wire()
}

/// A punctual section: a degenerated edge without curve on one vertex.
fn point_wire(point: Point3) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder.make_vertex_at(&mut vertex, point, 1e-7)?;
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    builder.add(&mut edge, &vertex.oriented(Orientation::Forward))?;
    builder.add(&mut edge, &vertex.oriented(Orientation::Reversed))?;
    builder.degenerated(&edge, true)?;
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge)?;
    Ok(wire)
}

/// The faces of a shape in exploration order.
fn faces(shape: &Shape) -> Vec<Face> {
    let mut found = Vec::new();
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        found.push(topods::face(explorer.current()).unwrap());
        explorer.next().unwrap();
    }
    found
}

/// The dynamic type name of the basis surface of each face.
fn surface_types(shape: &Shape) -> Vec<String> {
    faces(shape)
        .iter()
        .map(|face| {
            let surface = BRepTool::surface(face).unwrap();
            surface.get().unwrap().dynamic_type().name().to_string()
        })
        .collect()
}

#[test]
fn coaxial_circles_of_one_radius_give_a_cylinder() -> Result<(), Failure> {
    let mut generator = BRepFillGenerator::new();
    generator.add_wire(&circle_wire(0., 1.)?);
    generator.add_wire(&circle_wire(2., 1.)?);
    assert_eq!(generator.get_status(), ThruSectionErrorStatus::NotDone);
    generator.perform()?;
    assert_eq!(generator.get_status(), ThruSectionErrorStatus::Done);
    assert_eq!(
        surface_types(generator.shell()),
        ["Geom_RectangularTrimmedSurface"]
    );
    let face = &faces(generator.shell())[0];
    let surface = BRepTool::surface(face)?;
    let (_, _, v1, v2) = surface.get().unwrap().bounds();
    assert_eq!((v1, v2), (0., 2.));
    Ok(())
}

#[test]
fn squares_of_one_size_give_four_planes_and_a_cone_closes_on_an_apex() -> Result<(), Failure> {
    let mut generator = BRepFillGenerator::new();
    generator.add_wire(&square_wire(0., 1.)?);
    generator.add_wire(&square_wire(1., 1.)?);
    generator.perform()?;
    assert_eq!(surface_types(generator.shell()), ["Geom_Plane"; 4]);
    let mut cone = BRepFillGenerator::new();
    cone.add_wire(&point_wire(Point3::new(0., 0., -1.))?);
    cone.add_wire(&circle_wire(0., 1.)?);
    cone.add_wire(&point_wire(Point3::new(0., 0., 1.))?);
    cone.perform()?;
    assert_eq!(cone.get_status(), ThruSectionErrorStatus::Done);
    assert_eq!(surface_types(cone.shell()).len(), 2);
    assert!(cone.shell().closed()?);
    Ok(())
}

#[test]
fn squares_of_other_sizes_give_ruled_bsplines() -> Result<(), Failure> {
    let mut generator = BRepFillGenerator::new();
    generator.add_wire(&square_wire(0., 1.)?);
    generator.add_wire(&square_wire(1., 0.5)?);
    generator.perform()?;
    assert_eq!(surface_types(generator.shell()), ["Geom_BSplineSurface"; 4]);
    Ok(())
}

#[test]
fn an_edge_without_curve_stops_with_null3dcurve() -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let (mut a, mut b) = (Vertex::default(), Vertex::default());
    builder.make_vertex_at(&mut a, Point3::new(0., 0., 1.), 1e-7)?;
    builder.make_vertex_at(&mut b, Point3::new(1., 0., 1.), 1e-7)?;
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    builder.add(&mut edge, &a.oriented(Orientation::Forward))?;
    builder.add(&mut edge, &b.oriented(Orientation::Reversed))?;
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge)?;
    let segment =
        BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(1., 0., 0.))?.edge()?;
    let mut generator = BRepFillGenerator::new();
    generator.add_wire(&BRepBuilderAPIMakeWire::from_edge(&segment)?.wire()?);
    generator.add_wire(&wire);
    generator.perform()?;
    assert_eq!(generator.get_status(), ThruSectionErrorStatus::Null3DCurve);
    assert_eq!(faces(generator.shell()).len(), 0);
    Ok(())
}

#[test]
fn an_immutable_input_keeps_the_sections_and_maps_their_copies() -> Result<(), Failure> {
    let first = circle_wire(0., 1.)?;
    let second = circle_wire(1., 2.)?;
    let mut generator = BRepFillGenerator::new();
    generator.set_mutable_input(false);
    assert!(!generator.is_mutable_input());
    generator.add_wire(&first);
    generator.add_wire(&second);
    generator.perform()?;
    let original: Edge =
        topods::edge(Explorer::new(&first, ShapeEnum::Edge, ShapeEnum::Shape)?.current())?;
    // The input edge keeps no curve on the ruled face; its copy in the shell has one.
    let face = &faces(generator.shell())[0];
    assert!(BRepTool::curve_on_face(&original, face)?.pcurve.is_null());
    let copy: Shape = generator.result_shape(&original)?;
    assert!(!copy.is_same(&original));
    let new_wire: Shape = generator.result_shape(&first)?;
    assert!(!new_wire.is_same(&first));
    Ok(())
}

#[test]
fn generated_faces_are_keyed_by_the_explorer_edge_after_its_advance() -> Result<(), Failure> {
    // Native quirk: `aREd` copies the explorer's current edge after `Next`, so the face
    // of the only edge of a wire is recorded under the null edge.
    let mut generator = BRepFillGenerator::new();
    generator.add_wire(&circle_wire(0., 1.)?);
    generator.add_wire(&circle_wire(2., 1.)?);
    generator.perform()?;
    assert_eq!(generator.generated().extent(), 1);
    assert_eq!(generator.generated_shapes(&Shape::default()).len(), 1);
    Ok(())
}
