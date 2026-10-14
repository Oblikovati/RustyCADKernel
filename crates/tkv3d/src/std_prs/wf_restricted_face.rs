//! `StdPrs_WFRestrictedFace` and `StdPrs_WFDeflectionRestrictedFace`: the U and V isolines of
//! a face, trimmed by its p-curves with a `Hatch_Hatcher`, drawn with a fixed number of points
//! or with a deflection, and the point matching on the same isolines.
use super::{
    StdPrsCurve, StdPrsDeflectionCurve, StdPrsIsolinePolylines, StdPrsIsolineSurfaceSource,
    StdPrsToolRFace,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::ncollection::{HSequence, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dIsoCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::hatch::Hatcher;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::{CurveType, IsoType, Shape, SurfaceType};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;

/// Parameter bounds `[u_min, u_max, v_min, v_max]`.
type Bounds = [f64; 4];

/// Shrinks the open directions by a thousandth and adds the isolines: `nb` lines spread over
/// `(nb + 1)` steps (the native closed branch is never taken).
fn load_isos(
    hatcher: &mut Hatcher,
    bounds: &mut Bounds,
    closed: [bool; 2],
    draw: [bool; 2],
    numbers: [i32; 2],
) {
    if !closed[0] {
        bounds[0] += (bounds[1] - bounds[0]) / 1000.;
        bounds[1] -= (bounds[1] - bounds[0]) / 1000.;
    }
    if !closed[1] {
        bounds[2] += (bounds[3] - bounds[2]) / 1000.;
        bounds[3] -= (bounds[3] - bounds[2]) / 1000.;
    }
    if draw[0] && numbers[0] > 0 {
        let du = (bounds[1] - bounds[0]) / f64::from(1 + numbers[0]);
        for i in 1..=numbers[0] {
            hatcher.add_x_line(bounds[0] + du * f64::from(i));
        }
    }
    if draw[1] && numbers[1] > 0 {
        let dv = (bounds[3] - bounds[2]) / f64::from(1 + numbers[1]);
        for i in 1..=numbers[1] {
            hatcher.add_y_line(bounds[2] + dv * f64::from(i));
        }
    }
}

/// Trims the hatcher by an oriented segment.
fn trim(
    hatcher: &mut Hatcher,
    orientation: Orientation,
    p1: Point2,
    p2: Point2,
) -> Result<(), Failure> {
    if orientation == Orientation::Forward {
        hatcher.trim_segment(p1, p2, 0)
    } else {
        hatcher.trim_segment(p2, p1, 0)
    }
}

/// Trims by every restriction sampled with `nb_points` points (a line by its two ends).
fn trim_by_points<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
    hatcher: &mut Hatcher,
    tool: &StdPrsToolRFace,
    nb_points: i32,
) -> Result<(), Failure> {
    tool.init()?;
    while tool.more() {
        let orientation = tool.orientation();
        let curve = tool.value();
        let (u1, u2) = (curve.first_parameter()?, curve.last_parameter()?);
        if curve.get_type()? != CurveType::Line {
            let du = (u2 - u1) / f64::from(nb_points - 1);
            let mut p2 = curve.value(u1)?;
            for i in 2..=nb_points {
                let p1 = p2;
                p2 = curve.value(u1 + f64::from(i - 1) * du)?;
                trim(hatcher, orientation, p1, p2)?;
            }
        } else {
            let (p1, p2) = (curve.value(u1)?, curve.value(u2)?);
            trim(hatcher, orientation, p1, p2)?;
        }
        tool.next()?;
    }
    let _ = std::marker::PhantomData::<S>;
    Ok(())
}

/// The bounds of eleven samples of every restriction (a line by its two ends when
/// `lines_by_ends`), from RealLast/RealFirst.
fn sampled_bounds(tool: &StdPrsToolRFace, lines_by_ends: bool) -> Result<Bounds, Failure> {
    let mut bounds = [real_last(), real_first(), real_last(), real_first()];
    let mut add = |p: Point2| {
        if p.x() < bounds[0] {
            bounds[0] = p.x();
        }
        if p.x() > bounds[1] {
            bounds[1] = p.x();
        }
        if p.y() < bounds[2] {
            bounds[2] = p.y();
        }
        if p.y() > bounds[3] {
            bounds[3] = p.y();
        }
    };
    tool.init()?;
    while tool.more() {
        let curve = tool.value();
        let (mut u, v) = (curve.first_parameter()?, curve.last_parameter()?);
        if lines_by_ends && curve.get_type()? == CurveType::Line {
            add(curve.value(u)?);
            add(curve.value(v)?);
        } else {
            let step = (v - u) / 10.;
            for _ in 0..=10 {
                add(curve.value(u)?);
                u += step;
            }
        }
        tool.next()?;
    }
    Ok(bounds)
}

/// The B-spline or Bezier surface of a face adaptor, null otherwise.
fn pole_surface<S: Adaptor3dSurface + ?Sized>(
    face: &S,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(match face.get_type()? {
        SurfaceType::BezierSurface => face.bezier()?.into_cast(),
        SurfaceType::BSplineSurface => face.bspline()?.into_cast(),
        _ => Handle::default(),
    })
}

/// The isoline of a hatcher line on the face, loaded on `[b1, b2]`.
fn load_iso(
    iso: &Adaptor3dIsoCurve,
    hatcher: &Hatcher,
    line: i32,
    coordinate: f64,
    bounds: [f64; 2],
) -> Result<(), Failure> {
    let kind = if hatcher.is_x_line(line)? {
        IsoType::IsoU
    } else {
        IsoType::IsoV
    };
    iso.load_iso_range(kind, coordinate, (bounds[0], bounds[1]))
}

/// The isoline of a hatcher line on a pole surface.
fn pole_iso(
    surface: &Handle<dyn GeomSurface>,
    hatcher: &Hatcher,
    line: i32,
    coordinate: f64,
) -> Result<GeomAdaptorCurve, Failure> {
    let curve = if hatcher.is_x_line(line)? {
        surface.u_iso(coordinate)?
    } else {
        surface.v_iso(coordinate)?
    };
    GeomAdaptorCurve::with_curve(&curve)
}

/// `StdPrs_WFRestrictedFace`: isolines drawn with a fixed number of points.
/// Example: `StdPrsWfRestrictedFace::add(&prs, &face, &drawer)?`.
pub struct StdPrsWfRestrictedFace;

impl StdPrsWfRestrictedFace {
    /// `Add(prs, face, drawUIso, drawVIso, nbUIso, nbVIso, drawer, curves)`: the isolines of
    /// the face, bounded by the box of its p-curves (its natural bounds without p-curve);
    /// the points of every isoline are appended to `curves`. On a pole surface the isos are
    /// sampled with 30 points, else with the drawer discretisation; intervals open at
    /// RealFirst or RealLast are skipped. Nothing is drawn: the points are only collected.
    /// Example: `StdPrsWfRestrictedFace::add_isos(&prs, &face, true, true, 2, 2, &drawer, &mut curves)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_isos<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        draw_u_iso: bool,
        draw_v_iso: bool,
        nb_u_iso: i32,
        nb_v_iso: i32,
        drawer: &Handle<Prs3dDrawer>,
        curves: &mut StdPrsIsolinePolylines,
    ) -> Result<(), Failure> {
        let surface: &S = face.get().ok_or_else(null_face)?;
        let nb_points = drawer.discretisation();
        let tool = StdPrsToolRFace::with_surface(face)?;
        let mut box2d = BoundingBox2::default();
        tool.init()?;
        while tool.more() {
            Add2dCurve::add_adaptor(tool.value(), precision::p_confusion(), &mut box2d)?;
            tool.next()?;
        }
        let mut bounds = if box2d.is_void() {
            [
                surface.first_u_parameter()?,
                surface.last_u_parameter()?,
                surface.first_v_parameter()?,
                surface.last_v_parameter()?,
            ]
        } else {
            let [u_min, v_min, u_max, v_max] = box2d.get()?;
            [u_min, u_max, v_min, v_max]
        };
        let mut hatcher = Hatcher::new(1.0e-5, tool.is_oriented());
        let closed = [surface.is_u_closed()?, surface.is_v_closed()?];
        load_isos(
            &mut hatcher,
            &mut bounds,
            closed,
            [draw_u_iso, draw_v_iso],
            [nb_u_iso, nb_v_iso],
        );
        trim_by_points::<S>(&mut hatcher, &tool, nb_points)?;
        let iso = Adaptor3dIsoCurve::with_surface(&face.clone().into_cast());
        let poles = pole_surface(surface)?;
        for line in 1..=hatcher.nb_lines() {
            let coordinate = hatcher.coordinate(line)?;
            for interval in 1..=hatcher.nb_intervals_on(line)? {
                let (b1, b2) = (hatcher.start(line, interval)?, hatcher.end(line, interval)?);
                if b1 == real_first() || b2 == real_last() {
                    continue;
                }
                let points = Handle::new(HSequence::default());
                let mut sequence = Sequence::default();
                if !poles.is_null() {
                    let curve = pole_iso(&poles, &hatcher, line, coordinate)?;
                    StdPrsCurve::add_range_points(prs, &curve, b1, b2, &mut sequence, 30, false)?;
                } else {
                    load_iso(&iso, &hatcher, line, coordinate, [b1, b2])?;
                    StdPrsCurve::add_points(prs, &iso, drawer, &mut sequence, false)?;
                }
                points.change_sequence(|target| target.append_sequence(&mut sequence));
                curves.append(points);
            }
        }
        Ok(())
    }

    /// `Add(prs, face, drawer)`: both directions with the drawer isoline numbers.
    /// Example: `StdPrsWfRestrictedFace::add(&prs, &face, &drawer)?`.
    pub fn add<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        Self::add_with(prs, face, drawer, [true, true])
    }
    /// `AddUIso`: the U isolines only; `StdPrsWfRestrictedFace::add_u_iso(&prs, &face, &drawer)?`.
    pub fn add_u_iso<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        Self::add_with(prs, face, drawer, [true, false])
    }
    /// `AddVIso`: the V isolines only; `StdPrsWfRestrictedFace::add_v_iso(&prs, &face, &drawer)?`.
    pub fn add_v_iso<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        Self::add_with(prs, face, drawer, [false, true])
    }
    fn add_with<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
        draw: [bool; 2],
    ) -> Result<(), Failure> {
        let mut curves = StdPrsIsolinePolylines::default();
        Self::add_isos(
            prs,
            face,
            draw[0],
            draw[1],
            drawer.u_iso_aspect().number(),
            drawer.v_iso_aspect().number(),
            drawer,
            &mut curves,
        )
    }

    /// `Match(X, Y, Z, distance, face, drawUIso, drawVIso, deflection, nbUIso, nbVIso,
    /// drawer)`: whether a sampled isoline passes within `distance` of the point; the isos
    /// are bounded by eleven samples of every curved p-curve and the ends of the lines,
    /// infinite intervals clamped to the maximal parameter value.
    /// Example: `StdPrsWfRestrictedFace::match_isos(&p, 0.1, &face, true, true, 0.01, 2, 2, &drawer)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn match_isos<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        draw_u_iso: bool,
        draw_v_iso: bool,
        deflection: f64,
        nb_u_iso: i32,
        nb_v_iso: i32,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        let surface: &S = face.get().ok_or_else(null_face)?;
        let limit = drawer.maximal_parameter_value();
        let nb_points = drawer.discretisation();
        let tool = StdPrsToolRFace::with_surface(face)?;
        let mut bounds = sampled_bounds(&tool, true)?;
        let mut hatcher = Hatcher::new(1.0e-5, tool.is_oriented());
        let closed = [surface.is_u_closed()?, surface.is_v_closed()?];
        load_isos(
            &mut hatcher,
            &mut bounds,
            closed,
            [draw_u_iso, draw_v_iso],
            [nb_u_iso, nb_v_iso],
        );
        trim_by_points::<S>(&mut hatcher, &tool, nb_points)?;
        let iso = Adaptor3dIsoCurve::with_surface(&face.clone().into_cast());
        for line in 1..=hatcher.nb_lines() {
            let coordinate = hatcher.coordinate(line)?;
            for interval in 1..=hatcher.nb_intervals_on(line)? {
                let range = clamp_interval(&hatcher, line, interval, limit)?;
                load_iso(&iso, &hatcher, line, coordinate, range)?;
                if StdPrsCurve::matches_with_sampling(
                    point, distance, &iso, deflection, limit, nb_points,
                )? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// `Match(X, Y, Z, distance, face, drawer)`: both directions with the drawer values.
    /// Example: `StdPrsWfRestrictedFace::matches(&p, 0.1, &face, &drawer)?`.
    pub fn matches<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        Self::match_with(point, distance, face, drawer, [true, true])
    }
    /// `MatchUIso`; `StdPrsWfRestrictedFace::match_u_iso(&p, 0.1, &face, &drawer)?`.
    pub fn match_u_iso<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        Self::match_with(point, distance, face, drawer, [true, false])
    }
    /// `MatchVIso`; `StdPrsWfRestrictedFace::match_v_iso(&p, 0.1, &face, &drawer)?`.
    pub fn match_v_iso<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        Self::match_with(point, distance, face, drawer, [false, true])
    }
    fn match_with<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
        draw: [bool; 2],
    ) -> Result<bool, Failure> {
        Self::match_isos(
            point,
            distance,
            face,
            draw[0],
            draw[1],
            drawer.maximal_chordial_deviation(),
            drawer.u_iso_aspect().number(),
            drawer.v_iso_aspect().number(),
            drawer,
        )
    }
}

/// A hatcher interval with RealFirst/RealLast replaced by the parameter limit.
fn clamp_interval(
    hatcher: &Hatcher,
    line: i32,
    interval: i32,
    limit: f64,
) -> Result<[f64; 2], Failure> {
    let (b1, b2) = (hatcher.start(line, interval)?, hatcher.end(line, interval)?);
    Ok([
        if b1 == real_first() { -limit } else { b1 },
        if b2 == real_last() { limit } else { b2 },
    ])
}

fn null_face() -> Failure {
    Failure::with_message("Standard_NullObject", "BRepAdaptor_Surface handle is null")
}

/// `FindLimits`: the curve range intersected with `[first, last]`; an infinite end is pushed
/// out by doubling steps until the end points are `limit` apart.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // A NaN distance ends the native loops.
fn find_limits(
    curve: &dyn Adaptor3dCurve,
    limit: f64,
    bounds: &mut [f64; 2],
) -> Result<(), Failure> {
    bounds[0] = std_max(curve.first_parameter()?, bounds[0]);
    bounds[1] = std_min(curve.last_parameter()?, bounds[1]);
    let first_inf = precision::is_negative_infinite(bounds[0]);
    let last_inf = precision::is_positive_infinite(bounds[1]);
    let mut delta = 1.0;
    if first_inf && last_inf {
        loop {
            delta *= 2.;
            bounds[0] = -delta;
            bounds[1] = delta;
            let (p1, p2) = (curve.value(bounds[0])?, curve.value(bounds[1])?);
            if !(p1.distance(&p2) < limit) {
                break;
            }
        }
    } else if first_inf {
        let p2 = curve.value(bounds[1])?;
        loop {
            delta *= 2.;
            bounds[0] = bounds[1] - delta;
            if !(curve.value(bounds[0])?.distance(&p2) < limit) {
                break;
            }
        }
    } else if last_inf {
        let p1 = curve.value(bounds[0])?;
        loop {
            delta *= 2.;
            bounds[1] = bounds[0] + delta;
            if !(p1.distance(&curve.value(bounds[1])?) < limit) {
                break;
            }
        }
    }
    Ok(())
}

/// `StdPrs_WFDeflectionRestrictedFace`: isolines drawn with a deflection.
/// Example: `StdPrsWfDeflectionRestrictedFace::add(&prs, &face, &drawer)?`.
pub struct StdPrsWfDeflectionRestrictedFace;

impl StdPrsWfDeflectionRestrictedFace {
    /// `Add(prs, face, drawUIso, drawVIso, deflection, nbUIso, nbVIso, drawer, curves)`: the
    /// isolines bounded by the p-curves sampled with the deviation coefficient (a line by its
    /// ends, kept within the parameter limit about its middle, OCC13144), trimmed with a
    /// tolerance of a tenth of the smallest squared sample step (within
    /// [Confusion, 1e-5]), each interval limited by `FindLimits` and kept when longer than
    /// Confusion. The points of every isoline are appended to `curves`; nothing is drawn.
    /// Example: `StdPrsWfDeflectionRestrictedFace::add_isos(&prs, &face, true, true, 0.01, 2, 2, &drawer, &mut curves)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_isos<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        draw_u_iso: bool,
        draw_v_iso: bool,
        deflection: f64,
        nb_u_iso: i32,
        nb_v_iso: i32,
        drawer: &Handle<Prs3dDrawer>,
        curves: &mut StdPrsIsolinePolylines,
    ) -> Result<(), Failure> {
        let surface: &S = face.get().ok_or_else(null_face)?;
        let tool = StdPrsToolRFace::with_surface(face)?;
        let limit = drawer.maximal_parameter_value();
        let (uf, ul) = (surface.first_u_parameter()?, surface.last_u_parameter()?);
        let (vf, vl) = (surface.first_v_parameter()?, surface.last_v_parameter()?);
        let natural = [
            std_max(uf, -limit),
            std_min(ul, limit),
            std_max(vf, -limit),
            std_min(vl, limit),
        ];
        let ddefle = std_max(natural[1] - natural[0], natural[3] - natural[2])
            * drawer.deviation_coefficient();
        let mut segments: Vec<Point2> = Vec::new();
        let mut hatching_tol = 1.0e100;
        let mut bounds = [1.0e100, -1.0e100, 1.0e100, -1.0e100];
        let add = |p: Point2, bounds: &mut Bounds| {
            bounds[0] = std_min(p.x(), bounds[0]);
            bounds[1] = std_max(p.x(), bounds[1]);
            bounds[2] = std_min(p.y(), bounds[2]);
            bounds[3] = std_max(p.y(), bounds[3]);
        };
        tool.init()?;
        while tool.more() {
            let orientation = tool.orientation();
            let curve = tool.value();
            let pair = |p1: Point2, p2: Point2, segments: &mut Vec<Point2>| {
                if orientation == Orientation::Forward {
                    segments.extend([p1, p2]);
                } else {
                    segments.extend([p2, p1]);
                }
            };
            if curve.get_type()? != CurveType::Line {
                let sampler = QuasiUniformDeflection::with_curve(curve, ddefle, Shape::C1)?;
                if sampler.is_done() && sampler.nb_points()? >= 2 {
                    let first = sampler.value(1)?;
                    let mut p2 = Point2::new(first.x(), first.y());
                    add(p2, &mut bounds);
                    for i in 2..=sampler.nb_points()? {
                        let p1 = p2;
                        let next = sampler.value(i)?;
                        p2 = Point2::new(next.x(), next.y());
                        add(p2, &mut bounds);
                        hatching_tol = std_min(p1.square_distance(p2), hatching_tol);
                        pair(p1, p2, &mut segments);
                    }
                }
            } else {
                let (mut u1, mut u2) = (curve.first_parameter()?, curve.last_parameter()?);
                let mut origin = 0.;
                if !precision::is_negative_infinite(u1) || !precision::is_positive_infinite(u2) {
                    origin = if precision::is_negative_infinite(u1) {
                        u2 - limit
                    } else if precision::is_positive_infinite(u2) {
                        u1 + limit
                    } else {
                        (u1 + u2) * 0.5
                    };
                }
                u1 = std_max(origin - limit, u1);
                u2 = std_min(origin + limit, u2);
                let (p1, p2) = (curve.value(u1)?, curve.value(u2)?);
                add(p1, &mut bounds);
                add(p2, &mut bounds);
                hatching_tol = std_min(p1.square_distance(p2), hatching_tol);
                pair(p1, p2, &mut segments);
            }
            tool.next()?;
        }
        hatching_tol *= 0.1;
        hatching_tol = std_max(precision::confusion(), hatching_tol);
        hatching_tol = std_min(1.0e-5, hatching_tol);
        let mut hatcher = Hatcher::new(hatching_tol, tool.is_oriented());
        let closed = [surface.is_u_closed()?, surface.is_v_closed()?];
        load_isos(
            &mut hatcher,
            &mut bounds,
            closed,
            [draw_u_iso, draw_v_iso],
            [nb_u_iso, nb_v_iso],
        );
        for pair in segments.chunks(2) {
            hatcher.trim_segment(pair[0], pair[1], 0)?;
        }
        let iso = Adaptor3dIsoCurve::with_surface(&face.clone().into_cast());
        let poles = pole_surface(surface)?;
        let angle = drawer.deviation_angle();
        for line in 1..=hatcher.nb_lines() {
            let coordinate = hatcher.coordinate(line)?;
            for interval in 1..=hatcher.nb_intervals_on(line)? {
                let mut range = [hatcher.start(line, interval)?, hatcher.end(line, interval)?];
                let mut sequence = Sequence::default();
                if !poles.is_null() {
                    let curve = pole_iso(&poles, &hatcher, line, coordinate)?;
                    find_limits(&curve, limit, &mut range)?;
                    if range[1] - range[0] > precision::confusion() {
                        StdPrsDeflectionCurve::add_range_points(
                            prs,
                            &curve,
                            range[0],
                            range[1],
                            deflection,
                            &mut sequence,
                            angle,
                            false,
                        )?;
                    } else {
                        continue;
                    }
                } else {
                    load_iso(&iso, &hatcher, line, coordinate, range)?;
                    find_limits(&iso, limit, &mut range)?;
                    if range[1] - range[0] > precision::confusion() {
                        StdPrsDeflectionCurve::add_range_points(
                            prs,
                            &iso,
                            range[0],
                            range[1],
                            deflection,
                            &mut sequence,
                            angle,
                            false,
                        )?;
                    } else {
                        continue;
                    }
                }
                let points = Handle::new(HSequence::default());
                points.change_sequence(|target| target.append_sequence(&mut sequence));
                curves.append(points);
            }
        }
        Ok(())
    }

    /// `Add(prs, face, drawer)`: both directions with the drawer chordial deviation.
    /// Example: `StdPrsWfDeflectionRestrictedFace::add(&prs, &face, &drawer)?`.
    pub fn add<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        Self::add_with(prs, face, drawer, [true, true])
    }
    /// `AddUIso`; `StdPrsWfDeflectionRestrictedFace::add_u_iso(&prs, &face, &drawer)?`.
    pub fn add_u_iso<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        Self::add_with(prs, face, drawer, [true, false])
    }
    /// `AddVIso`; `StdPrsWfDeflectionRestrictedFace::add_v_iso(&prs, &face, &drawer)?`.
    pub fn add_v_iso<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        Self::add_with(prs, face, drawer, [false, true])
    }
    fn add_with<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
        draw: [bool; 2],
    ) -> Result<(), Failure> {
        let mut curves = StdPrsIsolinePolylines::default();
        Self::add_isos(
            prs,
            face,
            draw[0],
            draw[1],
            drawer.maximal_chordial_deviation(),
            drawer.u_iso_aspect().number(),
            drawer.v_iso_aspect().number(),
            drawer,
            &mut curves,
        )
    }

    /// `Match(X, Y, Z, distance, face, drawer, drawUIso, drawVIso, deflection, nbUIso,
    /// nbVIso)`: the isos bounded by eleven samples of every p-curve, trimmed by the
    /// p-curves sampled with `deflection`, matched with the deflection curve tolerance.
    /// Example: `StdPrsWfDeflectionRestrictedFace::match_isos(&p, 0.1, &face, &drawer, true, true, 0.01, 2, 2)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn match_isos<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
        draw_u_iso: bool,
        draw_v_iso: bool,
        deflection: f64,
        nb_u_iso: i32,
        nb_v_iso: i32,
    ) -> Result<bool, Failure> {
        let surface: &S = face.get().ok_or_else(null_face)?;
        let tool = StdPrsToolRFace::with_surface(face)?;
        let limit = drawer.maximal_parameter_value();
        let mut bounds = sampled_bounds(&tool, false)?;
        let mut hatcher = Hatcher::new(1.0e-5, tool.is_oriented());
        let closed = [surface.is_u_closed()?, surface.is_v_closed()?];
        load_isos(
            &mut hatcher,
            &mut bounds,
            closed,
            [draw_u_iso, draw_v_iso],
            [nb_u_iso, nb_v_iso],
        );
        tool.init()?;
        while tool.more() {
            let orientation = tool.orientation();
            let sampler = QuasiUniformDeflection::with_curve(tool.value(), deflection, Shape::C1)?;
            if sampler.is_done() && sampler.nb_points()? >= 2 {
                let first = sampler.value(1)?;
                let mut p2 = Point2::new(first.x(), first.y());
                for i in 2..=sampler.nb_points()? {
                    let p1 = p2;
                    let next = sampler.value(i)?;
                    p2 = Point2::new(next.x(), next.y());
                    trim(&mut hatcher, orientation, p1, p2)?;
                }
            }
            tool.next()?;
        }
        let iso = Adaptor3dIsoCurve::with_surface(&face.clone().into_cast());
        let angle = drawer.deviation_angle();
        for line in 1..=hatcher.nb_lines() {
            let coordinate = hatcher.coordinate(line)?;
            for interval in 1..=hatcher.nb_intervals_on(line)? {
                let range = clamp_interval(&hatcher, line, interval, limit)?;
                load_iso(&iso, &hatcher, line, coordinate, range)?;
                if StdPrsDeflectionCurve::matches_range_with_deflection(
                    point, distance, &iso, range[0], range[1], deflection, angle,
                )? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// `Match(X, Y, Z, distance, face, drawer)`; `StdPrsWfDeflectionRestrictedFace::matches(&p, 0.1, &face, &drawer)?`.
    pub fn matches<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        Self::match_with(point, distance, face, drawer, [true, true])
    }
    /// `MatchUIso`; `StdPrsWfDeflectionRestrictedFace::match_u_iso(&p, 0.1, &face, &drawer)?`.
    pub fn match_u_iso<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        Self::match_with(point, distance, face, drawer, [true, false])
    }
    /// `MatchVIso`; `StdPrsWfDeflectionRestrictedFace::match_v_iso(&p, 0.1, &face, &drawer)?`.
    pub fn match_v_iso<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        Self::match_with(point, distance, face, drawer, [false, true])
    }
    fn match_with<S: StdPrsIsolineSurfaceSource + RuntimeTyped + ?Sized>(
        point: &Point3,
        distance: f64,
        face: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
        draw: [bool; 2],
    ) -> Result<bool, Failure> {
        Self::match_isos(
            point,
            distance,
            face,
            drawer,
            draw[0],
            draw[1],
            drawer.maximal_chordial_deviation(),
            drawer.u_iso_aspect().number(),
            drawer.v_iso_aspect().number(),
        )
    }
}
