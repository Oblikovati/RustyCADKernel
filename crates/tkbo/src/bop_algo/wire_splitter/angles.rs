//! The 2D angles of the wire splitter (BOPAlgo_WireSplitter_1.cxx): `Angle2D`, `Angle`,
//! `ClockWiseAngle`, the 2D vertex tolerances and `RefineAngles`/`RefineAngle2D`.
use super::split_block::SmartMap;
use crate::bop_tools::algo_tools2d::curve_on_surface;
use crate::int_tools::Context;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkgeombase::geom_lprop::CLProps2d;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};
use std::f64::consts::PI;

/// The angle of an edge leaving (or entering, `is_in`) `vertex` in the parametric space of
/// `face`, measured from a short step along its p-curve (`Angle2D`).
pub(super) fn angle_2d(
    vertex: &Vertex,
    edge: &Edge,
    face: &Face,
    surface: &BRepAdaptorSurface,
    is_in: bool,
    context: &Context,
) -> Result<f64, Failure> {
    let parameter = BRepTool::parameter_on_face(vertex, edge, face)?;
    if precision::is_infinite(parameter) {
        return Ok(0.);
    }
    let (pcurve, range) = pcurve_of(edge, face, context)?;
    let tolerance = 2. * tolerance_2d(vertex, surface)?;
    let adaptor = Geom2dAdaptorCurve::with_curve(&pcurve)?;
    let step = angle_step(&pcurve, &adaptor, parameter, tolerance, range)?;
    let other = if (parameter - range.0).abs() < (parameter - range.1).abs() {
        parameter + step
    } else {
        parameter - step
    };
    let (step_point, point) = (adaptor.d0(other)?, adaptor.d0(parameter)?);
    let vector = if is_in {
        Vector2::between(step_point, point)
    } else {
        Vector2::between(point, step_point)
    };
    Ok(x_axis_angle(Direction2::from_vector(vector)?))
}

/// The parameter step of `Angle2D`: the 2D tolerance resolution, widened on curved p-curves
/// and capped to keep the direction of the curve.
fn angle_step(
    pcurve: &Handle<dyn Geom2dCurve>,
    adaptor: &Geom2dAdaptorCurve,
    parameter: f64,
    tolerance: f64,
    range: (f64, f64),
) -> Result<f64, Failure> {
    let mut step = adaptor.resolution(tolerance)?.max(precision::p_confusion());
    if adaptor.get_type()? != CurveType::Line {
        let mut props =
            CLProps2d::with_parameter(pcurve.clone(), parameter, 2, precision::p_confusion())?;
        if props.is_tangent_defined()? {
            let curvature = props.curvature()?;
            if curvature > precision::p_confusion() {
                let radius = 1. / curvature;
                let cosphi = radius / (radius + tolerance);
                // To avoid a small step for a big radius.
                step = step.max(cosphi.acos());
            }
        }
    }
    // For case chl/927/r9.
    let mut cap = 0.05 * (range.1 - range.0);
    if cap < 5.0e-5 {
        cap = 5.0e-5_f64.min((range.1 - range.0) / 2.);
    }
    // Saves the direction of the curve as much as possible with big tolerances.
    Ok(if step > cap { cap } else { step })
}

/// A p-curve and its parameter range.
type RangedPCurve = (Handle<dyn Geom2dCurve>, (f64, f64));

/// The p-curve of `edge` on `face` and its range (`BOPTools_AlgoTools2D::CurveOnSurface`).
fn pcurve_of(
    edge: &Edge,
    face: &Face,
    context: &Context,
) -> Result<RangedPCurve, Failure> {
    let mut pcurve = Handle::default();
    let mut range = (0., 0.);
    let mut tolerance = 0.;
    curve_on_surface(
        edge,
        face,
        &mut pcurve,
        &mut range,
        &mut tolerance,
        Some(context),
    )?;
    Ok((pcurve, range))
}

/// The angle of `direction` from the X axis in `[0, 2*PI)` (`Angle`).
pub(super) fn x_axis_angle(direction: Direction2) -> f64 {
    let angle = Direction2::X.angle(direction);
    if angle < 0. { angle + PI + PI } else { angle }
}

/// The clockwise angle turning from the reversed entering direction to the leaving one
/// (`ClockWiseAngle`); `clockwise_angle(0., PI)`.
pub(super) fn clockwise_angle(angle_in: f64, angle_out: f64) -> f64 {
    let two_pi = PI + PI;
    let angle_in = if angle_in >= two_pi {
        angle_in - two_pi
    } else {
        angle_in
    };
    let angle_out = if angle_out >= two_pi {
        angle_out - two_pi
    } else {
        angle_out
    };
    let mut reversed_in = angle_in + PI;
    if reversed_in >= two_pi {
        reversed_in -= two_pi;
    }
    let delta = reversed_in - angle_out;
    if delta <= 0. {
        two_pi + delta
    } else if delta <= 1.0e-14 {
        two_pi
    } else {
        delta
    }
}

/// The 2D tolerance of a vertex: the larger parametric resolution of its 3D tolerance, at
/// least the 3D tolerance, widened by 10% on B-spline surfaces (`Tolerance2D`).
pub(super) fn tolerance_2d(vertex: &Vertex, surface: &BRepAdaptorSurface) -> Result<f64, Failure> {
    let tolerance = BRepTool::vertex_tolerance(vertex)?;
    let u_resolution = surface.u_resolution(tolerance)?;
    let v_resolution = surface.v_resolution(tolerance)?;
    let mut tolerance_2d = if v_resolution > u_resolution {
        v_resolution
    } else {
        u_resolution
    };
    if tolerance_2d < tolerance {
        tolerance_2d = tolerance;
    }
    if surface.get_type()? == SurfaceType::BSplineSurface {
        tolerance_2d *= 1.1;
    }
    Ok(tolerance_2d)
}

/// The U and V resolutions of the vertex tolerance (`UTolerance2D`, `VTolerance2D`).
pub(super) fn uv_tolerance_2d(
    vertex: &Vertex,
    surface: &BRepAdaptorSurface,
) -> Result<(f64, f64), Failure> {
    let tolerance = BRepTool::vertex_tolerance(vertex)?;
    Ok((
        surface.u_resolution(tolerance)?,
        surface.v_resolution(tolerance)?,
    ))
}

/// The 2D point of `vertex` on the p-curve of `edge` on `face` (`Coord2d`).
pub(super) fn coord_2d(vertex: &Vertex, edge: &Edge, face: &Face) -> Result<Point2, Failure> {
    let parameter = BRepTool::parameter_on_face(vertex, edge, face)?;
    let found = BRepTool::curve_on_face(edge, face)?;
    let Some(pcurve) = found.pcurve.get() else {
        return Err(Failure::with_type(
            "Standard_NullObject",
            &format!("BOPAlgo_WireSplitter Coord2d: edge {edge:?} has no p-curve on the face"),
            "",
        ));
    };
    pcurve.d0(parameter)
}

/// The 2D point of the first FORWARD vertex of `edge`, else (99, 99) (`Coord2dVf`).
pub(super) fn coord_2d_forward(edge: &Edge, face: &Face) -> Result<Point2, Failure> {
    for vertex in rustycad_tkbrep::topods::ShapeIterator::new(edge, true, true) {
        if vertex.orientation() == rustycad_tkg3d::top_abs::Orientation::Forward {
            return coord_2d(&Vertex::unchecked(vertex), edge, face);
        }
    }
    Ok(Point2::new(99., 99.))
}

/// Refines the angles of the inner edges at every vertex (`RefineAngles`).
pub(super) fn refine_angles(
    face: &Face,
    smart: &mut SmartMap,
    context: &Context,
) -> Result<(), Failure> {
    for index in 1..=smart.extent() {
        let vertex = Vertex::unchecked(smart.find_key(index)?.clone());
        refine_vertex_angles(&vertex, face, smart.change_from_index(index)?, context)?;
    }
    Ok(())
}

/// Moves the angles of the inner edges leaving a vertex between its two boundary edges
/// (`RefineAngles` for one vertex).
fn refine_vertex_angles(
    vertex: &Vertex,
    face: &Face,
    infos: &mut [super::EdgeInfo],
    context: &Context,
) -> Result<(), Failure> {
    let (mut leaving, mut entering, mut boundaries, mut inner) = (0., 0., 0, 0);
    for info in infos.iter() {
        if info.is_inside() {
            inner += 1;
            continue;
        }
        boundaries += 1;
        if info.is_in() {
            entering = info.angle();
        } else {
            leaving = info.angle();
        }
    }
    if boundaries != 2 {
        return Ok(());
    }
    let bounds = AngleBounds {
        leaving,
        entering,
        delta: clockwise_angle(entering, leaving),
    };
    let mut refined: DataMap<rustycad_tkbrep::topods::Shape, f64, ShapeMapHasher> =
        DataMap::with_hasher(ShapeMapHasher, 1);
    for info in infos.iter() {
        if !info.is_inside() || info.is_in() {
            continue;
        }
        if let Some(angle) = refined_angle(vertex, info, face, &bounds, inner, context)? {
            refined.bind(info.edge().clone().into_shape(), angle);
        }
    }
    apply_refined_angles(infos, &refined);
    Ok(())
}

/// The boundary angles at one vertex: leaving (`aA1`), entering (`aA2`) and the clockwise
/// sector between them (`aDelta`).
struct AngleBounds {
    leaving: f64,
    entering: f64,
    delta: f64,
}

/// The new angle of one inner leaving edge outside the boundary sector, if any.
fn refined_angle(
    vertex: &Vertex,
    info: &super::EdgeInfo,
    face: &Face,
    bounds: &AngleBounds,
    inner: i32,
    context: &Context,
) -> Result<Option<f64>, Failure> {
    let mut angle = info.angle();
    if clockwise_angle(bounds.entering, angle) < bounds.delta {
        // Already inside.
        return Ok(None);
    }
    if refine_angle_2d(vertex, info.edge(), face, bounds, &mut angle, context)? {
        return Ok(Some(angle));
    }
    if inner != 2 {
        return Ok(None);
    }
    Ok(Some(if angle <= bounds.leaving {
        bounds.leaving + precision::angular()
    } else {
        bounds.entering - precision::angular()
    }))
}

fn apply_refined_angles(
    infos: &mut [super::EdgeInfo],
    refined: &DataMap<rustycad_tkbrep::topods::Shape, f64, ShapeMapHasher>,
) {
    if refined.is_empty() {
        return;
    }
    for info in infos.iter_mut() {
        let Some(angle) = refined.seek(info.edge()) else {
            continue;
        };
        let angle = if info.is_in() { angle + PI } else { *angle };
        info.set_angle(angle);
    }
}

/// Looks for a direction of the edge inside the boundary sector: intersects its p-curve
/// with the rays of the boundary directions and takes a point a little beyond the farthest
/// hit (`RefineAngle2D`).
fn refine_angle_2d(
    vertex: &Vertex,
    edge: &Edge,
    face: &Face,
    bounds: &AngleBounds,
    angle: &mut f64,
    context: &Context,
) -> Result<bool, Failure> {
    const TOLERANCE: f64 = 1.0e-10;
    let (pcurve, (first, last)) = pcurve_of(edge, face, context)?;
    let curve = Geom2dAdaptorCurve::default();
    curve.load_range(&pcurve, first, last)?;
    let parameter = BRepTool::parameter_on_face(vertex, edge, face)?;
    let origin = curve.d0(parameter)?;
    let opposite = if (parameter - first).abs() < (parameter - last).abs() {
        last
    } else {
        first
    };
    let max_step = 0.3 * (last - first);
    let domain = Domain::bounded(
        curve.d0(first)?,
        first,
        TOLERANCE,
        curve.d0(last)?,
        last,
        TOLERANCE,
    );
    let mut intersector = GInter::new();
    for ray_angle in [bounds.leaving, bounds.entering + PI] {
        let ray: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
            origin,
            Direction2::new(ray_angle.cos(), ray_angle.sin())?,
        ))
        .cast();
        let ray = Geom2dAdaptorCurve::with_curve(&ray)?;
        intersector.perform(&curve, &domain, &ray, &Domain::new(), TOLERANCE, TOLERANCE)?;
        if !intersector.is_done() {
            continue;
        }
        let Some(hit) = farthest_hit(&intersector, parameter, max_step)? else {
            continue;
        };
        let step = opposite - hit;
        if step.abs() < TOLERANCE {
            continue;
        }
        let point = curve.d0(hit + 0.01 * step)?;
        let candidate = x_axis_angle(Direction2::from_vector(Vector2::between(origin, point))?);
        if clockwise_angle(bounds.entering, candidate) < bounds.delta {
            *angle = candidate;
            return Ok(true);
        }
    }
    Ok(false)
}

/// The p-curve parameter of the hit farthest along the ray within `max_step` of the vertex.
fn farthest_hit(
    intersector: &GInter,
    parameter: f64,
    max_step: f64,
) -> Result<Option<f64>, Failure> {
    let (mut on_curve, mut on_ray) = (parameter, -1.);
    for index in 1..=intersector.nb_points()? {
        let point = intersector.point(index)?;
        let (curve_parameter, ray_parameter) = (point.param_on_first(), point.param_on_second());
        if ray_parameter > on_ray && (curve_parameter - parameter).abs() < max_step {
            on_ray = ray_parameter;
            on_curve = curve_parameter;
        }
    }
    Ok((on_ray > 0.).then_some(on_curve))
}
