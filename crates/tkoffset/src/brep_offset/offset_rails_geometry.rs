//! Rail p-curves, connector edges, and wire assembly for `BRepOffset_Offset`.
use super::offset_rails::PipeFaceOptions;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::vertices,
    topods::{Edge, Face, Vertex, Wire},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurve, geom_adaptor::GeomAdaptorCurve};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom_adaptor::GeomAdaptorSurface};
use rustycad_tkg3d::{geom::GeomSurface, top_abs::Orientation};
use rustycad_tkmath::{
    geom_abs::{CurveType, Shape as Continuity, SurfaceType},
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

struct RailPcurvePlacement {
    range: (f64, f64),
    path_tolerance: f64,
    rail_index: usize,
}

struct ConnectorGeometry<'a> {
    curve: &'a Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    index: usize,
}

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
    for (index, (edge, pcurve)) in edges.into_iter().zip(pcurves).enumerate() {
        let placement = RailPcurvePlacement {
            range: ranges.0,
            path_tolerance,
            rail_index: index,
        };
        set_rail_pcurve(builder, face, edge, pcurve, options, &placement)?;
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
    options: &PipeFaceOptions<'_>,
    placement: &RailPcurvePlacement,
) -> Result<(), Failure> {
    install_missing_3d_rail_curve(builder, edge, options, placement.rail_index)?;
    remove_degenerated_circle_curve(builder, edge)?;
    let tolerance = placement
        .path_tolerance
        .max(BRepTool::edge_tolerance(edge)? + options.error);
    builder.update_edge_pcurve(
        edge,
        pcurve,
        options.surface,
        &Location::default(),
        tolerance,
        None,
    )?;
    builder.range_on_face(edge, face, placement.range.0, placement.range.1)?;
    same_range(edge, tolerance)?;
    if options.error == 0.0 {
        builder.same_parameter(edge, false)?;
        same_parameter_edge(edge, tolerance.max(options.tolerance))?;
    }
    Ok(())
}

fn install_missing_3d_rail_curve(
    builder: &BRepBuilder,
    edge: &Edge,
    options: &PipeFaceOptions<'_>,
    rail_index: usize,
) -> Result<(), Failure> {
    if BRepTool::curve_with_location(edge)?.curve.is_some() {
        return Ok(());
    }
    let bounds = options.surface.bounds();
    let parameter = if rail_index == 0 { bounds.0 } else { bounds.1 };
    let curve = if options.exchange_uv {
        options
            .surface
            .v_iso(if rail_index == 0 { bounds.2 } else { bounds.3 })?
    } else {
        options.surface.u_iso(parameter)?
    };
    builder.update_edge_curve(edge, &curve, BRepTool::edge_tolerance(edge)?)
}

fn remove_degenerated_circle_curve(builder: &BRepBuilder, edge: &Edge) -> Result<(), Failure> {
    if !is_degenerated_circle(edge)? {
        return Ok(());
    }
    builder.degenerated(edge, true)
}

fn is_degenerated_circle(edge: &Edge) -> Result<bool, Failure> {
    let (curve, _, _) = BRepTool::curve(edge)?;
    let adaptor = GeomAdaptorCurve::with_curve(&curve)?;
    if adaptor.get_type()? != CurveType::Circle {
        return Ok(false);
    }
    Ok(adaptor.circle()?.radius() < rustycad_tkernel::precision::confusion())
}

pub(super) fn make_connectors(
    face: &Face,
    edges: [&Edge; 2],
    bounds: Bounds,
    path_tolerance: f64,
    options: &PipeFaceOptions<'_>,
) -> Result<Vec<Edge>, Failure> {
    let endpoints = connector_endpoints(edges);
    let (mut curves, range) = connector_geometry(options.exchange_uv, bounds);
    let builder = BRepBuilder::default();
    if options.closed_path {
        if options.exchange_uv {
            curves.swap(0, 1);
        }
        let edge = super::offset_rails_closed::make_closed_connector(
            &builder,
            face,
            [&curves[0], &curves[1]],
            endpoints[0].clone(),
            range,
            path_tolerance,
            options,
        )?;
        return Ok([edge.clone(), edge].into());
    }
    make_connector_pair(
        &builder,
        face,
        &curves,
        endpoints,
        range,
        path_tolerance,
        options,
    )
}

fn connector_endpoints(edges: [&Edge; 2]) -> [[rustycad_tkbrep::topods::Vertex; 2]; 2] {
    let (first1, last1) = vertices(edges[0], false);
    let (first2, last2) = vertices(edges[1], false);
    [[first1, first2], [last1, last2]]
}

fn make_connector_pair(
    builder: &BRepBuilder,
    face: &Face,
    curves: &[Handle<dyn Geom2dCurve>; 2],
    endpoints: [[rustycad_tkbrep::topods::Vertex; 2]; 2],
    range: (f64, f64),
    path_tolerance: f64,
    options: &PipeFaceOptions<'_>,
) -> Result<Vec<Edge>, Failure> {
    let start = make_connector(
        builder,
        face,
        [&endpoints[0][0], &endpoints[0][1]],
        path_tolerance,
        options,
        ConnectorGeometry {
            curve: &curves[0],
            range,
            index: 0,
        },
    )?;
    let end = make_connector(
        builder,
        face,
        [&endpoints[1][0], &endpoints[1][1]],
        path_tolerance,
        options,
        ConnectorGeometry {
            curve: &curves[1],
            range,
            index: 1,
        },
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
    face: &Face,
    vertices: [&rustycad_tkbrep::topods::Vertex; 2],
    path_tolerance: f64,
    options: &PipeFaceOptions<'_>,
    geometry: ConnectorGeometry<'_>,
) -> Result<Edge, Failure> {
    let supplied = options.supplied_connectors[geometry.index];
    if let Some((edge, reversed)) = supplied_connector(supplied, vertices)? {
        let curve = reverse_connector_curve(geometry.curve, reversed)?;
        let range = reverse_connector_range(geometry.range, reversed);
        return update_supplied_connector(
            builder,
            edge,
            &curve,
            face,
            range,
            path_tolerance,
            options,
        );
    }
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    add_connector_vertices(builder, &mut edge, vertices)?;
    attach_connector_geometry(
        builder,
        &edge,
        face,
        geometry.curve,
        options.surface,
        geometry.range,
        path_tolerance,
    )?;
    Ok(edge)
}

pub(super) fn supplied_connector(
    supplied: Option<&Edge>,
    endpoints: [&Vertex; 2],
) -> Result<Option<(Edge, bool)>, Failure> {
    let Some(edge) = supplied else {
        return Ok(None);
    };
    let (first, _) = vertices(edge, false);
    if first.is_same(endpoints[0]) {
        return Ok(Some((
            Edge::unchecked(edge.oriented(Orientation::Forward)),
            false,
        )));
    }
    if first.is_same(endpoints[1]) {
        return Ok(Some((
            Edge::unchecked(edge.oriented(Orientation::Reversed)),
            true,
        )));
    }
    Ok(None)
}

pub(super) fn reverse_connector_curve(
    curve: &Handle<dyn Geom2dCurve>,
    reverse: bool,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if reverse {
        curve.reversed()
    } else {
        Ok(curve.clone())
    }
}

pub(super) fn reverse_connector_range(range: (f64, f64), reverse: bool) -> (f64, f64) {
    if reverse { (-range.1, -range.0) } else { range }
}

fn update_supplied_connector(
    builder: &BRepBuilder,
    edge: Edge,
    pcurve: &Handle<dyn Geom2dCurve>,
    face: &Face,
    range: (f64, f64),
    path_tolerance: f64,
    options: &PipeFaceOptions<'_>,
) -> Result<Edge, Failure> {
    builder.update_edge_pcurve(
        &edge,
        pcurve,
        options.surface,
        &Location::default(),
        path_tolerance,
        None,
    )?;
    builder.range_on_face(&edge, face, range.0, range.1)?;
    build_curve3d(&edge, path_tolerance, Continuity::C1, 11, 30)?;
    builder.same_range(&edge, false)?;
    builder.same_parameter(&edge, false)?;
    same_range(&edge, path_tolerance)?;
    same_parameter_edge(&edge, options.tolerance)?;
    Ok(edge)
}

pub(super) fn add_connector_vertices(
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
    face: &Face,
    pcurve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    range: (f64, f64),
    tolerance: f64,
) -> Result<(), Failure> {
    builder.update_edge_pcurve_on_face(edge, pcurve, face, tolerance)?;
    builder.range_on_face(edge, face, range.0, range.1)?;
    compute_native_connector_curve(edge, surface)
}

pub(super) fn compute_native_connector_curve(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    let surface_type = GeomAdaptorSurface::with_surface(surface)?.get_type()?;
    if super::offset_rails_iso::store_exact_iso_curve(edge, surface)? {
        return Ok(());
    }
    if supports_native_curve_image(surface_type) {
        let _computed = build_curve3d(
            edge,
            rustycad_tkernel::precision::approximation(),
            Continuity::C1,
            11,
            30,
        )?;
    }
    Ok(())
}

pub(super) fn reverse_connector_curves(
    curves: [&Handle<dyn Geom2dCurve>; 2],
    reverse: bool,
) -> Result<[Handle<dyn Geom2dCurve>; 2], Failure> {
    Ok([
        reverse_connector_curve(curves[0], reverse)?,
        reverse_connector_curve(curves[1], reverse)?,
    ])
}

fn supports_native_curve_image(surface_type: SurfaceType) -> bool {
    matches!(
        surface_type,
        SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
    )
}

pub(super) fn unsupported_edge(name: &str) -> Failure {
    let message =
        format!("BRepOffset_Offset::{name}: expected an identity-located edge with a 3D curve");
    Failure::with_type("Standard_NotImplemented", &message, "BRepOffset_Offset")
}

pub(super) fn construction_error(reason: &str) -> Failure {
    Failure::with_type("Standard_ConstructionError", reason, "BRepOffset_Offset")
}
