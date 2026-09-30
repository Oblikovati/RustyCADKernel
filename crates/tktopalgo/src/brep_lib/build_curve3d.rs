// Rust translation of BRepLib::BuildCurve3d and BRepLib::BuildCurves3d (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::BuildCurve3d` and `BRepLib::BuildCurves3d`: the 3d curves of edges that have
//! only curves on surfaces.
// pmn 26/09/97 Add parameters of approximation in BuildCurve3d
use super::same_range::{check_same_range, same_range};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool, EdgeCurveOnSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomElementarySurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity, SurfaceType};
use rustycad_tkmath::toploc::Location;

/// The default arguments of `BRepLib::BuildCurve3d` beyond the tolerance
/// (`GeomAbs_C1`, 14, 0).
pub const BUILD_CURVE3D_DEFAULTS: (Continuity, i32, i32) = (Continuity::C1, 14, 0);

/// `evaluateMaxSegment`: `max_segment` when set, else 30 plus the larger knot count of
/// a B-spline surface or 2d curve.
fn evaluate_max_segment(
    max_segment: i32,
    curve_on_surface: &Adaptor3dCurveOnSurface,
) -> Result<i32, Failure> {
    if max_segment != 0 {
        return Ok(max_segment);
    }
    let surface: Handle<dyn Adaptor3dSurface> = curve_on_surface.get_surface();
    let curve2d: Handle<dyn Adaptor2dCurve2d> = curve_on_surface.get_curve();
    let (mut nb_s_knots, mut nb_c2d_knots): (f64, f64) = (0.0, 0.0);
    if let Some(s) = surface.get()
        && s.get_type()? == SurfaceType::BSplineSurface
        && let Some(bspline) = s.bspline()?.get()
    {
        nb_s_knots = f64::from(bspline.nb_u_knots().max(bspline.nb_v_knots()));
    }
    if let Some(c) = curve2d.get()
        && c.get_type()? == CurveType::BSplineCurve
    {
        nb_c2d_knots = f64::from(c.nb_knots()?);
    }
    Ok((30.0 + std_max(nb_s_knots, nb_c2d_knots)) as i32)
}

/// The plane of a surface, or of the basis of a rectangular trimmed surface.
fn plane_of(surface: &Handle<dyn GeomSurface>) -> Handle<GeomPlane> {
    match surface.cast::<GeomRectangularTrimmedSurface>().get() {
        Some(trimmed) => trimmed.basis_surface().cast::<GeomPlane>(),
        None => surface.cast::<GeomPlane>(),
    }
}

/// "search a curve on a plane": the first curve on surface of the edge whose surface is
/// a plane (the last one read when there is none), and that plane.
fn curve_on_plane(edge: &Edge) -> Result<(EdgeCurveOnSurface, Handle<GeomPlane>), Failure> {
    let mut index: i32 = 0;
    loop {
        index += 1;
        let found = BRepTool::curve_on_surface_by_index(edge, index)?;
        let plane = plane_of(&found.surface);
        if found.surface.is_null() || plane.is_some() {
            return Ok((found, plane));
        }
    }
}

/// `BRepLib::BuildCurve3d(E, Tolerance, Continuity, MaxDegree, MaxSegment)`: true when
/// the edge has a 3d curve or gets one: the image of a p-curve on a plane, else the
/// GeomLib::BuildCurve3d approximation of its first curve on surface (the edge tolerance
/// becomes at least `tolerance`, and an edge with one curve on surface is flagged
/// SameParameter). A degenerated edge and a failed approximation give false.
/// `brep_lib::build_curve3d(&edge, 1e-5, Continuity::C1, 14, 0)?`.
pub fn build_curve3d(
    edge: &Edge,
    tolerance: f64,
    continuity: Continuity,
    max_degree: i32,
    max_segment: i32,
) -> Result<bool, Failure> {
    // if the edge has a 3d curve returns true
    if BRepTool::curve_with_location(edge)?.curve.is_some() {
        return Ok(true);
    }
    // this should not exists but UpdateEdge makes funny things if the edge is not same range
    if !check_same_range(edge, precision::confusion())? {
        same_range(edge, tolerance)?;
    }
    let (on_plane, plane) = curve_on_plane(edge)?;
    if let Some(p) = plane.get() {
        return image_on_plane(edge, &on_plane, p);
    }
    if BRepTool::degenerated(edge)? {
        return Ok(false);
    }
    approximated(edge, tolerance, (continuity, max_degree, max_segment))
}

/// "compute the 3d curve" of a p-curve on a plane: GeomLib::To3d in the plane frame,
/// stored with the location of the curve on surface and the range of the edge on the
/// plane (looked up at the identity location, as natively).
fn image_on_plane(
    edge: &Edge,
    on_plane: &EdgeCurveOnSurface,
    plane: &GeomPlane,
) -> Result<bool, Failure> {
    let axes = plane.position().ax2()?;
    let c3d: Handle<dyn GeomCurve> = geom_lib::to_3d(axes, &on_plane.pcurve)?;
    if c3d.is_null() {
        return Ok(false);
    }
    // update the edge
    let builder = BRepBuilder::default();
    builder.update_edge_located_curve(edge, &c3d, &on_plane.location, 0.0e0)?;
    let (first, last) = BRepTool::range_on_surface(edge, &on_plane.surface, &Location::default())?;
    builder.range(edge, first, last, false)?; // Do not forget 3D range.(PRO6412)
    Ok(true)
}

/// "compute the 3d curve using existing surface": the approximation of the first of the
/// (at most two) curves on surface of the non-degenerated edge.
fn approximated(
    edge: &Edge,
    tolerance: f64,
    (continuity, max_degree, max_segment): (Continuity, i32, i32),
) -> Result<bool, Failure> {
    // BRep_Tool::CurveOnSurface expects Index >= 1, so start from 1.
    let mut found: Vec<EdgeCurveOnSurface> = Vec::new();
    for index in 1..3 {
        let on_surface = BRepTool::curve_on_surface_by_index(edge, index)?;
        if on_surface.pcurve.is_some() && found.len() < 2 {
            found.push(on_surface);
        }
    }
    // Natively the first curve of an edge without any is uninitialized (a null pcurve):
    // the adaptor below raises Standard_NullObject.
    let first = found.first().cloned().unwrap_or_default();
    let curve2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_range(
        &first.pcurve,
        first.first,
        first.last,
    )?)
    .cast();
    let surface: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&first.surface)?).cast();
    let curve_on_surface = Adaptor3dCurveOnSurface::with_curve(&curve2d, &surface)?;
    let mut new_curve: Handle<dyn GeomCurve> = Handle::default();
    let segments: i32 = evaluate_max_segment(max_segment, &curve_on_surface)?;
    let range = (first.first, first.last);
    geom_lib::build_curve3d(
        tolerance,
        &curve_on_surface,
        range,
        &mut new_curve,
        continuity,
        max_degree,
        segments,
    )?;
    // Patch: max_deviation = std::max(tolerance, max_deviation)
    let max_deviation: f64 = std_max(BRepTool::edge_tolerance(edge)?, tolerance);
    if new_curve.is_null() {
        return Ok(false);
    }
    let builder = BRepBuilder::default();
    builder.update_edge_located_curve(edge, &new_curve, &first.location, max_deviation)?;
    if found.len() == 1 {
        // if there is only one curve on surface attached to the edge
        // than it can be qualified sameparameter
        builder.same_parameter(edge, true)?;
    }
    Ok(true)
}

/// `BRepLib::BuildCurves3d(S, Tolerance, Continuity, MaxDegree, MaxSegment)`: builds the
/// 3d curve of every edge of the shape (each once); false when one failed;
/// `brep_lib::build_curves3d(&shape, 1e-5, Continuity::C1, 14, 0)?`.
pub fn build_curves3d(
    shape: &Shape,
    tolerance: f64,
    continuity: Continuity,
    max_degree: i32,
    max_segment: i32,
) -> Result<bool, Failure> {
    let mut ok: bool = true;
    let mut counter = MapOfShape::default();
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        if counter.add(explorer.current().clone()) {
            let edge = Edge::unchecked(explorer.current().clone());
            let built: bool = build_curve3d(&edge, tolerance, continuity, max_degree, max_segment)?;
            ok = ok && built;
        }
        explorer.next()?;
    }
    Ok(ok)
}

/// `BRepLib::BuildCurves3d(S)`: `BuildCurves3d(S, 1.0e-5)` with the default continuity,
/// degree and segments; `brep_lib::build_curves3d_default(&shape)?`.
pub fn build_curves3d_default(shape: &Shape) -> Result<bool, Failure> {
    let (continuity, max_degree, max_segment) = BUILD_CURVE3D_DEFAULTS;
    build_curves3d(shape, 1.0e-5, continuity, max_degree, max_segment)
}
