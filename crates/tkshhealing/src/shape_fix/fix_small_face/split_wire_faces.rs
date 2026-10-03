use super::geometry::SplitBoundary;
use crate::shape_analysis::ShapeAnalysisWire;
use crate::shape_fix::{ShapeFixEdge, ShapeFixWire};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{Explorer, first_vertex, last_vertex};
use rustycad_tkbrep::topods::{Builder, Edge, Face, Shape, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis1, Direction3, Vector3};

pub(super) fn split_face_wires(
    face: &Face,
    boundary: &SplitBoundary,
) -> Result<Option<(Face, Face)>, Failure> {
    let surface = BRepTool::surface(face)?;
    let Some((mut wire, connector)) = prepared_split_wire(face, boundary)? else {
        return Ok(None);
    };
    insert_split_edges(&mut wire, boundary)?;
    wire = reorder_wire(&wire, face)?;
    let (first_wire, second_wire) = divide_wire(&wire, boundary, &connector)?;
    if first_wire.is_null() || second_wire.is_null() {
        return Ok(None);
    }
    let faces = faces_on_surface(&surface, &first_wire, &second_wire)?;
    Ok(Some(faces))
}

fn prepared_split_wire(
    face: &Face,
    boundary: &SplitBoundary,
) -> Result<Option<(Wire, Edge)>, Failure> {
    let connector = make_connector(boundary)?;
    ShapeFixEdge::new().fix_add_pcurve(&connector, face, false, 0.0)?;
    let Some(wire) = single_wire(face)? else {
        return Ok(None);
    };
    Ok(Some((reorder_wire(&wire, face)?, connector)))
}

fn make_connector(boundary: &SplitBoundary) -> Result<Edge, Failure> {
    let origin = BRepTool::pnt(&boundary.vertex)?;
    let destination = BRepTool::pnt(&boundary.split_vertex)?;
    let vector = Vector3::between(origin, destination);
    let direction = Direction3::from_vector(vector)?;
    let line = Handle::new(GeomLine::new(Axis1::new(origin, direction)));
    let curve: Handle<dyn GeomCurve> = line.cast();
    let builder = BRepBuilder::default();
    let mut connector = Edge::default();
    builder.make_edge_with_curve(&mut connector, &curve, precision::confusion())?;
    builder.range(&connector, 0.0, vector.magnitude(), true)?;
    add_endpoints(&mut connector, &boundary.vertex, &boundary.split_vertex)?;
    Ok(connector)
}

fn add_endpoints(
    edge: &mut Edge,
    start: &rustycad_tkbrep::topods::Vertex,
    end: &rustycad_tkbrep::topods::Vertex,
) -> Result<(), Failure> {
    let mut shape = edge.clone().into_shape();
    Builder.add(&mut shape, &start.oriented(Orientation::Forward))?;
    Builder.add(&mut shape, &end.oriented(Orientation::Reversed))?;
    *edge = rustycad_tkbrep::topods::edge(&shape)?;
    Ok(())
}

fn single_wire(face: &Face) -> Result<Option<Wire>, Failure> {
    let mut wires = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    )?;
    if !wires.more() {
        return Ok(None);
    }
    let wire = rustycad_tkbrep::topods::wire(wires.current())?;
    wires.next()?;
    if wires.more() {
        return Ok(None);
    }
    Ok(Some(wire))
}

fn reorder_wire(wire: &Wire, face: &Face) -> Result<Wire, Failure> {
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire(
        wire,
        face,
        precision::confusion(),
    )?);
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    fixer.fix_reorder()?;
    fixer.wire()
}

fn insert_split_edges(wire: &mut Wire, boundary: &SplitBoundary) -> Result<(), Failure> {
    let mut shape = wire.clone().into_shape();
    let mut edges = Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while edges.more() {
        let edge = rustycad_tkbrep::topods::edge(edges.current())?;
        if edge.is_same(&boundary.source_edge.clone().into_shape()) {
            replace_source_edge_in_wire(&mut shape, &edge, boundary)?;
        }
        edges.next()?;
    }
    *wire = rustycad_tkbrep::topods::wire(&shape)?;
    Ok(())
}

fn replace_source_edge_in_wire(
    wire: &mut Shape,
    edge: &Edge,
    boundary: &SplitBoundary,
) -> Result<(), Failure> {
    Builder.remove(wire, &edge.clone().into_shape())?;
    add_forward_split_edges(wire, boundary)
}

fn add_forward_split_edges(wire: &mut Shape, boundary: &SplitBoundary) -> Result<(), Failure> {
    Builder.add(wire, &boundary.first_edge.oriented(Orientation::Forward))?;
    Builder.add(wire, &boundary.second_edge.oriented(Orientation::Forward))
}

fn divide_wire(
    wire: &Wire,
    boundary: &SplitBoundary,
    connector: &Edge,
) -> Result<(Wire, Wire), Failure> {
    let mut wires = empty_wire_pair();
    copy_edges_between_splits(wire, boundary, connector, &mut wires)?;
    typed_wire_pair(wires)
}

fn empty_wire_pair() -> (Wire, Wire) {
    let mut first = Wire::default();
    let mut second = Wire::default();
    Builder.make_wire(&mut first);
    Builder.make_wire(&mut second);
    (first, second)
}

fn copy_edges_between_splits(
    wire: &Wire,
    boundary: &SplitBoundary,
    connector: &Edge,
    wires: &mut (Wire, Wire),
) -> Result<(), Failure> {
    let mut edges = Explorer::new(
        &wire.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        let edge = rustycad_tkbrep::topods::edge(edges.current())?;
        append_source_wire_edge(&mut wires.0, &edge)?;
        append_connector_and_swap(&mut wires.0, &mut wires.1, &edge, boundary, connector)?;
        edges.next()?;
    }
    Ok(())
}

fn append_source_wire_edge(wire: &mut Wire, edge: &Edge) -> Result<(), Failure> {
    Builder.add(&mut wire.clone().into_shape(), &edge.clone().into_shape())
}

fn typed_wire_pair(wires: (Wire, Wire)) -> Result<(Wire, Wire), Failure> {
    Ok((
        rustycad_tkbrep::topods::wire(&wires.0.into_shape())?,
        rustycad_tkbrep::topods::wire(&wires.1.into_shape())?,
    ))
}

fn append_connector_and_swap(
    first: &mut Wire,
    second: &mut Wire,
    edge: &Edge,
    boundary: &SplitBoundary,
    connector: &Edge,
) -> Result<(), Failure> {
    let control = oriented_control_vertex(edge);
    append_boundary_connectors(first, second, &control, boundary, connector)
}

fn oriented_control_vertex(edge: &Edge) -> rustycad_tkbrep::topods::Vertex {
    if edge.orientation() == Orientation::Forward {
        last_vertex(edge, false)
    } else {
        first_vertex(edge, false)
    }
}

fn append_boundary_connectors(
    first: &mut Wire,
    second: &mut Wire,
    control: &rustycad_tkbrep::topods::Vertex,
    boundary: &SplitBoundary,
    connector: &Edge,
) -> Result<(), Failure> {
    append_matching_connector(
        first,
        second,
        connector,
        control,
        &boundary.vertex,
        Orientation::Forward,
    )?;
    append_matching_connector(
        first,
        second,
        connector,
        control,
        &boundary.split_vertex,
        Orientation::Reversed,
    )?;
    Ok(())
}

fn append_matching_connector(
    first: &mut Wire,
    second: &mut Wire,
    connector: &Edge,
    control: &rustycad_tkbrep::topods::Vertex,
    target: &rustycad_tkbrep::topods::Vertex,
    orientation: Orientation,
) -> Result<(), Failure> {
    if control.is_same(&target.clone().into_shape()) {
        append_connector_to_first_and_swap(first, second, connector, orientation)?;
    }
    Ok(())
}

fn append_connector_to_first_and_swap(
    first: &mut Wire,
    second: &mut Wire,
    connector: &Edge,
    orientation: Orientation,
) -> Result<(), Failure> {
    append_connector(first, connector, orientation)?;
    std::mem::swap(first, second);
    Ok(())
}

fn append_connector(
    wire: &mut Wire,
    connector: &Edge,
    orientation: Orientation,
) -> Result<(), Failure> {
    Builder.add(
        &mut wire.clone().into_shape(),
        &connector.oriented(orientation),
    )
}

fn faces_on_surface(
    surface: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    first: &Wire,
    second: &Wire,
) -> Result<(Face, Face), Failure> {
    let builder = BRepBuilder::default();
    let mut left = Face::default();
    let mut right = Face::default();
    builder.make_face_on_surface(&mut left, surface, precision::confusion())?;
    builder.make_face_on_surface(&mut right, surface, precision::confusion())?;
    let mut left_shape = left.into_shape();
    let mut right_shape = right.into_shape();
    Builder.add(&mut left_shape, &first.clone().into_shape())?;
    Builder.add(&mut right_shape, &second.clone().into_shape())?;
    Ok((
        rustycad_tkbrep::topods::face(&left_shape)?,
        rustycad_tkbrep::topods::face(&right_shape)?,
    ))
}
