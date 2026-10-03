use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkgeomalgo::geom_fill::GeomFillGenerator;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

pub(super) fn loft_wall(
    original: &Edge,
    offset: &Edge,
    first: &Edge,
    second: &Edge,
    wire: &Wire,
) -> Result<Face, Failure> {
    let mut generator = GeomFillGenerator::new();
    generator.add_curve(&trimmed_edge_curve(original)?)?;
    generator.add_curve(&trimmed_edge_curve(offset)?)?;
    generator.perform(precision::p_confusion())?;
    let surface = generator.surface();
    let bounds = surface
        .get()
        .expect("generated wall surface must be non-null")
        .bounds();
    set_wall_pcurve(original, line(bounds.2, false), &surface)?;
    set_wall_pcurve(offset, line(bounds.3, false), &surface)?;
    set_connecting_curves(first, second, &surface, bounds)?;
    BRepLibMakeFace::from_surface_wire(&surface, wire, true)?.face()
}

fn trimmed_edge_curve(edge: &Edge) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    Ok(Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?).cast())
}

fn line(coordinate: f64, vertical: bool) -> Handle<dyn Geom2dCurve> {
    let (point, direction) = if vertical {
        (
            Point2::new(coordinate, 0.0),
            Direction2::new(0.0, 1.0).unwrap(),
        )
    } else {
        (
            Point2::new(0.0, coordinate),
            Direction2::new(1.0, 0.0).unwrap(),
        )
    };
    Handle::new(Geom2dLine::new(Axis2d::new(point, direction))).cast()
}

fn set_wall_pcurve(
    edge: &Edge,
    curve: Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    BRepBuilder::default().update_edge_pcurve(
        edge,
        &curve,
        surface,
        &Location::default(),
        precision::confusion(),
        None,
    )
}

fn set_connecting_curves(
    first: &Edge,
    second: &Edge,
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Result<(), Failure> {
    let (umin, umax, vmin, vmax) = bounds;
    if first.is_same(second) {
        return set_seam_curves(first, surface, bounds);
    }
    let builder = BRepBuilder::default();
    for edge in [first, second] {
        builder.same_parameter(edge, false)?;
        builder.same_range(edge, false)?;
    }
    set_connecting_pcurve(first, surface, umax, vmin, vmax)?;
    set_connecting_pcurve(second, surface, umin, vmin, vmax)?;
    set_connecting_curve(first, surface, umax, vmin, vmax)?;
    set_connecting_curve(second, surface, umin, vmin, vmax)
}

fn set_connecting_pcurve(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    u: f64,
    vmin: f64,
    vmax: f64,
) -> Result<(), Failure> {
    set_wall_pcurve(edge, line(u, true), surface)?;
    BRepBuilder::default().range_on_surface(edge, surface, &Location::default(), vmin, vmax)
}

fn set_connecting_curve(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    u: f64,
    vmin: f64,
    vmax: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let curve = surface
        .get()
        .expect("wall surface must be non-null")
        .u_iso(u)?;
    builder.update_edge_curve(edge, &curve, precision::confusion())?;
    builder.range(edge, vmin, vmax, true)
}

fn set_seam_curves(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Result<(), Failure> {
    let (umin, umax, vmin, vmax) = bounds;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurves(
        edge,
        (&line(umax, true), &line(umin, true)),
        surface,
        &Location::default(),
        precision::confusion(),
        None,
    )?;
    let curve = surface
        .get()
        .expect("wall surface must be non-null")
        .u_iso(umin)?;
    builder.update_edge_curve(edge, &curve, precision::confusion())?;
    builder.range(edge, vmin, vmax, false)
}
