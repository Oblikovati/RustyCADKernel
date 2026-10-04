//! Rail p-curves, connector edges, and wire assembly for `BRepOffset_Offset`.
use super::offset_rails::PipeFaceOptions;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::vertices,
    topods::{Edge, Face, Wire},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{geom::GeomSurface, top_abs::Orientation};
use rustycad_tkmath::{
    geom_abs::Shape as Continuity,
    gp::{Direction2, Point2},
    toploc::Location,
};
use rustycad_tktopalgo::brep_lib::{build_curve3d, same_parameter_edge, same_range};

pub(super) fn assemble_pipe_face(
    builder: &BRepBuilder,
    face: &mut Face,
    rails: [&Edge; 2],
    connectors: &[Edge],
    exchange_uv: bool,
) -> Result<(), Failure> {
    let wire = make_pipe_wire(builder, rails, connectors, exchange_uv)?;
    attach_pipe_wire(builder, face, &wire, exchange_uv)
}

fn make_pipe_wire(
    builder: &BRepBuilder,
    rails: [&Edge; 2],
    connectors: &[Edge],
    exchange_uv: bool,
) -> Result<Wire, Failure> {
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &rails[0].oriented(Orientation::Reversed))?;
    builder.add(&mut wire, &rails[1].oriented(Orientation::Forward))?;
    builder.add(&mut wire, &connectors[1].oriented(Orientation::Reversed))?;
    builder.add(&mut wire, &connectors[0])?;
    if exchange_uv {
        wire.reverse();
    }
    Ok(wire)
}

fn attach_pipe_wire(
    builder: &BRepBuilder,
    face: &mut Face,
    wire: &Wire,
    exchange_uv: bool,
) -> Result<(), Failure> {
    builder.add(face, &wire.clone().into_shape())?;
    if exchange_uv {
        face.reverse();
    }
    Ok(())
}

pub(super) type Bounds = (f64, f64, f64, f64);
pub(super) type RailRanges = ((f64, f64), (f64, f64));

pub(super) fn install_rail_pcurves(
    builder: &BRepBuilder,
    face: &Face,
    edges: [&Edge; 2],
    options: &PipeFaceOptions<'_>,
    path_tolerance: f64,
) -> Result<RailRanges, Failure> {
    let (pcurves, ranges) = rail_geometry(options.exchange_uv, options.surface.bounds());
    install_rail_pair(
        builder,
        face,
        edges,
        &pcurves,
        ranges,
        options,
        path_tolerance,
    )?;
    Ok(ranges)
}

fn rail_geometry(exchange_uv: bool, bounds: Bounds) -> ([Handle<dyn Geom2dCurve>; 2], RailRanges) {
    let (f1, l1, f2, l2) = bounds;
    let (pcurve1, pcurve2, range) = if exchange_uv {
        (
            line(Point2::new(0.0, f2), Direction2::X),
            line(Point2::new(0.0, l2), Direction2::X),
            (f1, l1),
        )
    } else {
        (
            line(Point2::new(f1, 0.0), Direction2::Y),
            line(Point2::new(l1, 0.0), Direction2::Y),
            (f2, l2),
        )
    };
    ([pcurve1, pcurve2], (range, range))
}

fn install_rail_pair(
    builder: &BRepBuilder,
    face: &Face,
    edges: [&Edge; 2],
    pcurves: &[Handle<dyn Geom2dCurve>; 2],
    ranges: RailRanges,
    options: &PipeFaceOptions<'_>,
    path_tolerance: f64,
) -> Result<(), Failure> {
    for (edge, pcurve) in edges.into_iter().zip(pcurves) {
        set_rail_pcurve(
            builder,
            face,
            edge,
            pcurve,
            ranges.0,
            options,
            path_tolerance,
        )?;
    }
    Ok(())
}

fn line(point: Point2, direction: Direction2) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(point, direction)).cast()
}

fn set_rail_pcurve(
    builder: &BRepBuilder,
    face: &Face,
    edge: &Edge,
    pcurve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    options: &PipeFaceOptions<'_>,
    path_tolerance: f64,
) -> Result<(), Failure> {
    let tolerance = path_tolerance.max(BRepTool::edge_tolerance(edge)? + options.error);
    builder.update_edge_pcurve(
        edge,
        pcurve,
        options.surface,
        &Location::default(),
        tolerance,
        None,
    )?;
    builder.range_on_face(edge, face, range.0, range.1)?;
    same_range(edge, tolerance)?;
    if options.error == 0.0 {
        builder.same_parameter(edge, false)?;
        same_parameter_edge(edge, tolerance.max(options.tolerance))?;
    }
    Ok(())
}

pub(super) fn make_connectors(
    surface: &Handle<dyn GeomSurface>,
    edges: [&Edge; 2],
    exchange_uv: bool,
    bounds: Bounds,
    tolerance: f64,
) -> Result<Vec<Edge>, Failure> {
    let endpoints = connector_endpoints(edges);
    let (curves, range) = connector_geometry(exchange_uv, bounds);
    let builder = BRepBuilder::default();
    make_connector_pair(&builder, surface, &curves, endpoints, range, tolerance)
}

fn connector_endpoints(edges: [&Edge; 2]) -> [[rustycad_tkbrep::topods::Vertex; 2]; 2] {
    let (first1, last1) = vertices(edges[0], false);
    let (first2, last2) = vertices(edges[1], false);
    [[first1, first2], [last1, last2]]
}

fn make_connector_pair(
    builder: &BRepBuilder,
    surface: &Handle<dyn GeomSurface>,
    curves: &[Handle<dyn Geom2dCurve>; 2],
    endpoints: [[rustycad_tkbrep::topods::Vertex; 2]; 2],
    range: (f64, f64),
    tolerance: f64,
) -> Result<Vec<Edge>, Failure> {
    let start = make_connector(
        builder,
        &curves[0],
        surface,
        [&endpoints[0][0], &endpoints[0][1]],
        range,
        tolerance,
    )?;
    let end = make_connector(
        builder,
        &curves[1],
        surface,
        [&endpoints[1][0], &endpoints[1][1]],
        range,
        tolerance,
    )?;
    Ok([start, end].into())
}

fn connector_geometry(
    exchange_uv: bool,
    (f1, l1, f2, l2): Bounds,
) -> ([Handle<dyn Geom2dCurve>; 2], (f64, f64)) {
    if exchange_uv {
        return (
            [
                line(Point2::new(f1, 0.0), Direction2::Y),
                line(Point2::new(l1, 0.0), Direction2::Y),
            ],
            (f2, l2),
        );
    }
    (
        [
            line(Point2::new(0.0, f2), Direction2::X),
            line(Point2::new(0.0, l2), Direction2::X),
        ],
        (f1, l1),
    )
}

fn make_connector(
    builder: &BRepBuilder,
    pcurve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    vertices: [&rustycad_tkbrep::topods::Vertex; 2],
    range: (f64, f64),
    tolerance: f64,
) -> Result<Edge, Failure> {
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    add_connector_vertices(builder, &mut edge, vertices)?;
    attach_connector_geometry(builder, &edge, pcurve, surface, range, tolerance)?;
    Ok(edge)
}

fn add_connector_vertices(
    builder: &BRepBuilder,
    edge: &mut Edge,
    vertices: [&rustycad_tkbrep::topods::Vertex; 2],
) -> Result<(), Failure> {
    builder.add(edge, &vertices[0].oriented(Orientation::Forward))?;
    builder.add(edge, &vertices[1].oriented(Orientation::Reversed))
}

fn attach_connector_geometry(
    builder: &BRepBuilder,
    edge: &Edge,
    pcurve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    range: (f64, f64),
    tolerance: f64,
) -> Result<(), Failure> {
    builder.update_edge_pcurve(edge, pcurve, surface, &Location::default(), tolerance, None)?;
    builder.range_on_surface(edge, surface, &Location::default(), range.0, range.1)?;
    build_curve3d(edge, 1.0e-7, Continuity::C1, 11, 30)?;
    same_parameter_edge(edge, 1.0e-7)?;
    Ok(())
}

pub(super) fn unsupported_edge(name: &str) -> Failure {
    let message =
        format!("BRepOffset_Offset::{name}: expected an identity-located edge with a 3D curve");
    Failure::with_type("Standard_NotImplemented", &message, "BRepOffset_Offset")
}

pub(super) fn construction_error(reason: &str) -> Failure {
    Failure::with_type("Standard_ConstructionError", reason, "BRepOffset_Offset")
}
