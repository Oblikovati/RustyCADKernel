// Rust translation of BRepLib::UpdateEdgeTol and BRepLib::UpdateEdgeTolerance (BRepLib.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::UpdateEdgeTol`: the edge tolerance as the largest distance between its
//! reference curve and its curves on surfaces.
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepGCurve, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::toploc::Location;

const FACTOR: f64 = 100.0e0;
const SAFE_FACTOR: f64 = 1.4e0;
const MIN_SAMPLING_POINTS: i32 = 30;
const MAX_SAMPLING_POINTS: i32 = 90;

/// The surface of a representation, transformed by its location when not the identity.
fn located_surface(g: &dyn BRepGCurve) -> Result<Handle<dyn GeomSurface>, Failure> {
    let location: Location = g.location();
    if location.is_identity() {
        return g.surface();
    }
    let surface = g.surface()?;
    let s = surface
        .get()
        .ok_or_else(|| super::errors::null_object("BRepLib::UpdateEdgeTol: null surface"))?;
    Ok(s.transformed(location.transformation())?
        .cast::<dyn GeomSurface>())
}

/// The curve on surface of a p-curve on a surface.
fn curve_on_surface(
    pcurve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
) -> Result<Adaptor3dCurveOnSurface, Failure> {
    let curve2d: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(pcurve)?).cast();
    let adapted: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surface)?).cast();
    Adaptor3dCurveOnSurface::with_curve(&curve2d, &adapted)
}

/// The reference curve of the check: the (located) 3d curve, else the first curve on
/// surface, with the index of the representation to skip (-1 with a 3d curve).
enum Reference {
    Curve3d(Box<GeomAdaptorCurve>),
    OnSurface(Box<Adaptor3dCurveOnSurface>, i32),
}

/// The reference curve and the deflection sampler on its range.
fn reference(
    edge: &Edge,
    reps: &[Handle<dyn BRepCurveRepresentation>],
    min_tol: f64,
) -> Result<(Reference, QuasiUniformDeflection), Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let mut sampler = QuasiUniformDeflection::new();
    if let Some(c) = found.curve.get() {
        let curve: Handle<dyn GeomCurve> = if found.location.is_identity() {
            found.curve.clone()
        } else {
            c.transformed(found.location.transformation())?.cast()
        };
        let adaptor = GeomAdaptorCurve::with_curve(&curve)?;
        let curve3d: &dyn Adaptor3dCurve = &adaptor;
        sampler.initialize_range(
            curve3d,
            min_tol * FACTOR,
            found.first,
            found.last,
            Continuity::C1,
        )?;
        return Ok((Reference::Curve3d(Box::new(adaptor)), sampler));
    }
    let (pcurve, surface, first, last) = first_curve_on_surface(reps)?;
    // The native search never advances its iterator: it counts one representation and
    // stops at the first one (or loops forever when that one is not a curve on surface).
    let index: i32 = if reps.is_empty() { 0 } else { 1 };
    let adaptor = curve_on_surface(&pcurve, &surface)?;
    let curve: &dyn Adaptor3dCurve = &adaptor;
    sampler.initialize_range(curve, min_tol * FACTOR, first, last, Continuity::C1)?;
    Ok((Reference::OnSurface(Box::new(adaptor), index), sampler))
}

/// A p-curve, its located surface and its range.
type SurfaceCurve = (Handle<dyn Geom2dCurve>, Handle<dyn GeomSurface>, f64, f64);

/// The native search of the first curve on surface, which reads the first
/// representation again and again: its p-curve, located surface and range; an edge
/// without representation gives null curves (Standard_NullObject below).
fn first_curve_on_surface(
    reps: &[Handle<dyn BRepCurveRepresentation>],
) -> Result<SurfaceCurve, Failure> {
    let Some(first) = reps.first() else {
        return Ok((Handle::default(), Handle::default(), 0.0, 0.0));
    };
    let gcurve = as_gcurve(first);
    loop {
        if let Some(g) = gcurve.get().filter(|g| g.is_curve_on_surface()) {
            let (f, l) = g.range();
            return Ok((g.pcurve()?, located_surface(g)?, f, l));
        }
        // must be inverted: the native loop spins here without end
        std::hint::spin_loop();
    }
}

/// The sampling parameters, densified to 30 or thinned to 90.
fn sampling_parameters(sampler: &QuasiUniformDeflection) -> Result<Array1<'static, f64>, Failure> {
    let count: i32 = sampler.nb_points()?;
    let mut sampling = Array1::<f64>::with_bounds(1, count)
        .map_err(|e| e.into_failure("BRepLib::UpdateEdgeTol"))?;
    for index in 1..=count {
        sampling[index] = sampler.parameter(index)?;
    }
    let mut parameters = Array1::<f64>::default();
    if count < MIN_SAMPLING_POINTS {
        geom_lib::densify_array1_of_real(MIN_SAMPLING_POINTS, &sampling, &mut parameters)?;
    } else if count > MAX_SAMPLING_POINTS {
        geom_lib::remove_points_from_array(MAX_SAMPLING_POINTS, &sampling, &mut parameters)?;
    } else {
        parameters = sampling;
    }
    Ok(parameters)
}

/// `BRepLib::UpdateEdgeTol(E, MinToleranceRequest, MaxToleranceToCheck)`: for a
/// non-degenerated edge of tolerance at most `max_tolerance_to_check`, sets the edge
/// tolerance (up or down) to 1.4 times the largest distance between its reference curve
/// and each other curve on surface (the second p-curve of a seam only), sampled by a
/// quasi-uniform deflection of 100 times `min_tolerance_requested`; true when set.
/// `brep_lib::update_edge_tol(&edge, 1e-5, 1.0)?`.
pub fn update_edge_tol(
    edge: &Edge,
    min_tolerance_requested: f64,
    max_tolerance_to_check: f64,
) -> Result<bool, Failure> {
    if BRepTool::degenerated(edge)? {
        return Ok(false);
    }
    if BRepTool::edge_tolerance(edge)? > max_tolerance_to_check {
        return Ok(false);
    }
    let tedge = edge_tshape(edge, "BRepLib::UpdateEdgeTol")?;
    let reps = tedge.curves();
    let (reference, sampler) = reference(edge, &reps, min_tolerance_requested)?;
    let parameters = sampling_parameters(&sampler)?;
    let skipped: i32 = match &reference {
        Reference::Curve3d(_) => -1,
        Reference::OnSurface(_, index) => *index,
    };
    let same_parameter: bool = BRepTool::same_parameter(edge)?;
    let mut edge_tolerance: f64 = 0.0e0;
    for (curve_index, rep) in (0..).zip(reps.iter()) {
        let gcurve = as_gcurve(rep);
        let Some(g) = gcurve.get() else { continue };
        if curve_index == skipped {
            continue;
        }
        let Some(pcurve) = checked_pcurve(g)? else {
            continue;
        };
        let on_surface = curve_on_surface(&pcurve, &located_surface(g)?)?;
        let max_distance: f64 = deviation(
            &on_surface,
            &reference,
            same_parameter,
            min_tolerance_requested,
            &parameters,
        )?;
        edge_tolerance = std_max(max_distance * SAFE_FACTOR, edge_tolerance);
    }
    tedge.set_tolerance(edge_tolerance);
    Ok(true)
}

/// The p-curve checked for a representation: the second one of a seam, the first one of
/// another curve on surface, none otherwise.
fn checked_pcurve(g: &dyn BRepGCurve) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    if g.is_curve_on_closed_surface() {
        return Ok(Some(g.pcurve2()?));
    }
    if g.is_curve_on_surface() {
        return Ok(Some(g.pcurve()?));
    }
    Ok(None)
}

/// The distance of a curve on surface to the reference: parametric for a SameParameter
/// edge, along the parameter otherwise. The parametric distance of an edge without 3d
/// curve reads the unloaded 3d adaptor natively (a crash); the port raises
/// Standard_NullObject there.
fn deviation(
    on_surface: &Adaptor3dCurveOnSurface,
    reference: &Reference,
    same_parameter: bool,
    min_tol: f64,
    parameters: &Array1<'_, f64>,
) -> Result<f64, Failure> {
    let empty = GeomAdaptorCurve::default();
    let (curve3d, reference_curve): (&dyn Adaptor3dCurve, &dyn Adaptor3dCurve) = match reference {
        Reference::Curve3d(c) => (c.as_ref(), c.as_ref()),
        Reference::OnSurface(c, _) => (&empty, c.as_ref()),
    };
    if same_parameter {
        return geom_lib::eval_max_parametric_distance(on_surface, curve3d, min_tol, parameters);
    }
    geom_lib::eval_max_distance_along_parameter(on_surface, reference_curve, min_tol, parameters)
}

/// `BRepLib::UpdateEdgeTolerance(S, MinToleranceRequest, MaxToleranceToCheck)`:
/// `UpdateEdgeTol` on every edge of the shape (each once); true when one was updated;
/// `brep_lib::update_edge_tolerance(&shape, 1e-5, 1.0)?`.
pub fn update_edge_tolerance(
    shape: &Shape,
    min_tolerance_requested: f64,
    max_tolerance_to_check: f64,
) -> Result<bool, Failure> {
    let mut counter = MapOfShape::default();
    let mut return_status: bool = false;
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        if counter.add(explorer.current().clone()) {
            let edge = Edge::unchecked(explorer.current().clone());
            let local_flag: bool =
                update_edge_tol(&edge, min_tolerance_requested, max_tolerance_to_check)?;
            return_status = return_status || local_flag;
        }
        explorer.next()?;
    }
    Ok(return_status)
}
