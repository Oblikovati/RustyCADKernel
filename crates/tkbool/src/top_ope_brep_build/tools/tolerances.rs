// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_Tools_1.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tolerance corrections of `TopOpeBRepBuild_Tools` (Tools_1.cxx): the vertex
//! tolerances by the 3d curves of their edges, the edge tolerances by their curves on
//! the surfaces of their faces, and the 2d closure check of a face.
use super::{TopOpeBRepBuildTools, sub_shapes};
use rustycad_tkbrep::brep::access::{edge_tshape, face_tshape, vertex_tshape};
use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepGCurve, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::{Geom2dAdaptorCurve, make_curve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomElementarySurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::extrema::LocateExtPC;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_check::{BRepCheckStatus, BRepCheckWire};

/// `NCONTROL`: the number of control points of `Validate`.
const NCONTROL: i32 = 23;

/// The largest square distance of `Validate` and whether it exceeds the start.
struct MaxDistance {
    max: f64,
    flag: bool,
}

impl MaxDistance {
    fn take(&mut self, d: f64) {
        if d > self.max {
            self.max = d;
            self.flag = true;
        }
    }
}

/// The parameter `i` of `NCONTROL` on (f, l).
fn control(i: i32, f: f64, l: f64) -> f64 {
    let n1 = f64::from(NCONTROL - 1);
    ((n1 - f64::from(i)) * f + f64::from(i) * l) / n1
}

/// The samples of `Validate` on different parameterizations: the ends, then each inner
/// control point located on the other curve.
fn validate_projected(
    cref: &dyn Adaptor3dCurve,
    other: &dyn Adaptor3dCurve,
    tol: f64,
    md: &mut MaxDistance,
) -> Result<(), Failure> {
    let (first, last) = (cref.first_parameter()?, cref.last_parameter()?);
    let (ofirst, olast) = (other.first_parameter()?, other.last_parameter()?);
    md.take(cref.value(first)?.square_distance(&other.value(ofirst)?));
    md.take(cref.value(last)?.square_distance(&other.value(olast)?));
    let mut refd = LocateExtPC::new();
    refd.initialize(cref, (first, last), cref.resolution(tol)?)?;
    let mut otherd = LocateExtPC::new();
    otherd.initialize(other, (ofirst, olast), other.resolution(tol)?)?;
    for i in 2..NCONTROL - 1 {
        let rprm = control(i, first, last);
        let pref = cref.value(rprm)?;
        let oprm = control(i, ofirst, olast);
        let pother = other.value(oprm)?;
        refd.perform(pother, rprm)?;
        if refd.is_done() && refd.square_distance()? > tol * tol {
            md.take(refd.square_distance()?);
        }
        otherd.perform(pref, oprm)?;
        if otherd.is_done() && otherd.square_distance()? > tol * tol {
            md.take(otherd.square_distance()?);
        }
    }
    Ok(())
}

/// The static `Validate(CRef, Other, Tol, SameParameter, aNewTolerance)`: 1.05 times
/// the largest distance between the curves at the control points (by the same
/// parameters, else located) when larger than `tol`, None otherwise.
fn validate(
    cref: &dyn Adaptor3dCurve,
    other: &dyn Adaptor3dCurve,
    tol: f64,
    same_parameter: bool,
) -> Result<Option<f64>, Failure> {
    let (first, last) = (cref.first_parameter()?, cref.last_parameter()?);
    let mut md = MaxDistance {
        max: tol * tol,
        flag: false,
    };
    let proj =
        !same_parameter || first != other.first_parameter()? || last != other.last_parameter()?;
    if proj {
        validate_projected(cref, other, tol, &mut md)?;
    } else {
        for i in 0..NCONTROL {
            let prm = control(i, first, last);
            md.take(cref.value(prm)?.square_distance(&other.value(prm)?));
        }
    }
    Ok(md.flag.then(|| md.max.sqrt() * 1.05))
}

/// The vertex tolerance raised to the distance of its point to `p` (plus a tenth of
/// the start tolerance) when larger than the square tolerance and below the maximum.
fn raise_vertex(
    v: &Shape,
    p_curve: rustycad_tkmath::gp::Point3,
    tol2: f64,
    dd: f64,
    max_tol: f64,
) -> Result<(), Failure> {
    let tv = vertex_tshape(v, "TopOpeBRepBuild_Tools::CheckEdge")?;
    let d2 = tv.pnt().square_distance(&p_curve);
    if d2 > tol2 {
        let new_tol = d2.sqrt() + dd;
        if new_tol < max_tol {
            tv.update_tolerance(new_tol);
        }
    }
    Ok(())
}

/// The vertex checks on one 3d curve of the edge: its points on the curve and, for a
/// FORWARD (REVERSED) vertex, the first (last) point of the curve range.
fn check_vertex_on_curve(
    v: &Shape,
    r: &dyn BRepCurveRepresentation,
    cr: &Handle<dyn BRepCurveRepresentation>,
    l: &Location,
    (tol2, dd, max_tol): (f64, f64, f64),
) -> Result<(), Failure> {
    let c = r.curve3d()?;
    let Some(curve) = c.get() else {
        return Ok(());
    };
    let tv = vertex_tshape(v, "TopOpeBRepBuild_Tools::CheckEdge")?;
    for pr in tv.points() {
        let Some(p) = pr.get().filter(|p| p.is_point_on_curve_at(&c, l)) else {
            continue;
        };
        raise_vertex(
            v,
            curve.value(p.parameter())?.transformed(l.transformation()),
            tol2,
            dd,
            max_tol,
        )?;
    }
    let orv = v.orientation();
    if matches!(orv, Orientation::Forward | Orientation::Reversed) {
        let gc = cr.cast::<dyn BRepGCurve>();
        let g = gc.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "TopOpeBRepBuild_Tools::CheckEdge: not a BRep_GCurve",
            )
        })?;
        let prm = if orv == Orientation::Forward {
            g.first()
        } else {
            g.last()
        };
        raise_vertex(
            v,
            curve.value(prm)?.transformed(l.transformation()),
            tol2,
            dd,
            max_tol,
        )?;
    }
    Ok(())
}

/// The static `CheckEdge(E, aMaxTol)`: each vertex tolerance raised to the distances of
/// its point to the points of the 3d curves of the forward edge.
fn check_edge(ed: &Edge, max_tol: f64) -> Result<(), Failure> {
    let e = ed.oriented(Orientation::Forward);
    let te = edge_tshape(&e, "TopOpeBRepBuild_Tools::CheckEdge")?;
    for v in Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let mut tol = BRepTool::vertex_tolerance(&topods::vertex(&v)?)?;
        tol = tol.max(BRepTool::edge_tolerance(&topods::edge(&e)?)?);
        let dd = 0.1 * tol;
        for cr in te.curves() {
            let Some(r) = cr.get().filter(|r| r.is_curve3d()) else {
                continue;
            };
            let l = (e.location() * &r.location())?.predivided(v.location())?;
            check_vertex_on_curve(&v, r, &cr, &l, (tol * tol, dd, max_tol))?;
        }
    }
    Ok(())
}

/// The reference curve of `CorrectEdgeTolerance`: the first 3d curve with a curve
/// (only when unique), else the first curve on surface of a not degenerated edge;
/// None when the edge has no or several 3d curves, or a 3d curve while degenerated.
fn reference_representation(
    curves: &[Handle<dyn BRepCurveRepresentation>],
    degenerated: bool,
) -> Result<Option<Handle<dyn BRepCurveRepresentation>>, Failure> {
    let mut cref: Handle<dyn BRepCurveRepresentation> = Handle::default();
    let mut unique = 0;
    for cr in curves {
        let Some(r) = cr.get().filter(|r| r.is_curve3d()) else {
            continue;
        };
        unique += 1;
        if cref.is_null() && !r.curve3d()?.is_null() {
            cref = cr.clone();
        }
    }
    if unique != 1 {
        return Ok(None);
    }
    if cref.is_null() && !degenerated {
        if let Some(cr) = curves
            .iter()
            .find(|cr| cr.get().is_some_and(|r| r.is_curve_on_surface()))
        {
            cref = cr.clone();
        }
    } else if !cref.is_null() && degenerated {
        return Ok(None);
    }
    Ok((!cref.is_null()).then_some(cref))
}

/// `new Adaptor3d_CurveOnSurface(new Geom2dAdaptor_Curve(pc, f, l), new
/// GeomAdaptor_Surface(S))`.
fn curve_on_surface(
    pc: &Handle<dyn Geom2dCurve>,
    (f, l): (f64, f64),
    s: &Handle<dyn GeomSurface>,
) -> Result<Adaptor3dCurveOnSurface, Failure> {
    let curve: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(pc, f, l)?).cast();
    let surface: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(s)?).cast();
    Adaptor3dCurveOnSurface::with_curve(&curve, &surface)
}

/// A geometry moved by a location.
fn moved<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    g: &Handle<dyn GeomSurface>,
    l: &Location,
) -> Result<Handle<T>, Failure> {
    let s = g.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "TopOpeBRepBuild_Tools::CorrectCurveOnSurface: null surface",
        )
    })?;
    Ok(s.transformed(l.transformation())?.cast::<T>())
}

/// The adaptor of the reference curve on its range (None for an empty range).
fn reference_adaptor(
    cref: &dyn BRepCurveRepresentation,
    gc: &dyn BRepGCurve,
) -> Result<Option<Handle<dyn Adaptor3dCurve>>, Failure> {
    let (first, last) = gc.range();
    if last <= first {
        return Ok(None);
    }
    if cref.is_curve3d() {
        let c3d = cref
            .curve3d()?
            .get()
            .ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "TopOpeBRepBuild_Tools: null 3d curve",
                )
            })?
            .transformed(cref.location().transformation())?
            .cast::<dyn GeomCurve>();
        return Ok(Some(
            Handle::new(GeomAdaptorCurve::with_range(&c3d, first, last)?).cast(),
        ));
    }
    let sref = moved::<dyn GeomSurface>(&cref.surface()?, &cref.location())?;
    Ok(Some(
        Handle::new(curve_on_surface(&cref.pcurve()?, (first, last), &sref)?).cast(),
    ))
}

/// The edge tolerance raised (below the maximum) to a validated distance.
fn raise_edge(e: &Edge, new_tol: Option<f64>, max_tol: f64) -> Result<(), Failure> {
    if let Some(t) = new_tol.filter(|&t| t < max_tol) {
        edge_tshape(e, "TopOpeBRepBuild_Tools::CorrectEdgeTolerance")?.update_tolerance(t);
    }
    Ok(())
}

/// The curves on the face surface of the edge (other than the reference): each one
/// (and the second one of a closed surface) validated against the reference; false
/// when there is none, None when the same range flag is violated.
fn check_pcurves(
    e: &Edge,
    ctx: &EdgeContext<'_>,
    su: &Handle<dyn GeomSurface>,
    l: &Location,
) -> Result<Option<bool>, Failure> {
    let hc = ctx.hcurve.get().expect("reference adaptor");
    let (first, last) = (hc.first_parameter()?, hc.last_parameter()?);
    let mut found = false;
    for cr in edge_tshape(e, "TopOpeBRepBuild_Tools::CorrectEdgeTolerance")?.curves() {
        let Some(r) = cr.get() else { continue };
        if cr.object_address() == ctx.cref.object_address() || !r.is_curve_on_surface_at(su, l) {
            continue;
        }
        found = true;
        let gc = cr.cast::<dyn BRepGCurve>();
        let (f, la) = gc.get().map(|g| g.range()).unwrap_or_default();
        if ctx.same_range && (f != first || la != last) {
            return Ok(None);
        }
        let sb = moved::<dyn GeomSurface>(su, l)?;
        let acs = curve_on_surface(&r.pcurve()?, (f, la), &sb)?;
        raise_edge(
            e,
            validate(hc, &acs, ctx.tol, ctx.same_parameter)?,
            ctx.max_tol,
        )?;
        if r.is_curve_on_closed_surface() {
            let acs2 = curve_on_surface(&r.pcurve2()?, (f, la), &sb)?;
            raise_edge(
                e,
                validate(hc, &acs2, ctx.tol, ctx.same_parameter)?,
                ctx.max_tol,
            )?;
        }
    }
    Ok(Some(found))
}

/// The frame of `CorrectEdgeTolerance`.
struct EdgeContext<'a> {
    cref: &'a Handle<dyn BRepCurveRepresentation>,
    hcurve: &'a Handle<dyn Adaptor3dCurve>,
    tol: f64,
    same_parameter: bool,
    same_range: bool,
    max_tol: f64,
}

/// No curve on the face surface: on a plane (or its rectangular trimmed basis), the
/// reference curve projected on the fly (as BRep_Tool does) and validated.
fn check_projection(
    e: &Edge,
    ctx: &EdgeContext<'_>,
    su: &Handle<dyn GeomSurface>,
    l: &Location,
) -> Result<(), Failure> {
    let plane = match su.cast::<GeomRectangularTrimmedSurface>().get() {
        Some(t) => t.basis_surface().cast::<GeomPlane>(),
        None => su.cast::<GeomPlane>(),
    };
    if plane.is_null() {
        return Ok(());
    }
    let p = moved::<GeomPlane>(&plane.clone().cast(), l)?;
    let gahs: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&p.clone().cast())?).cast();
    let hc = ctx.hcurve.get().expect("reference adaptor");
    let (first, last) = (hc.first_parameter()?, hc.last_parameter()?);
    let gac = ctx.hcurve.cast::<GeomAdaptorCurve>();
    let c3d = gac.get().ok_or_else(|| Failure::with_message("Standard_NullObject", "TopOpeBRepBuild_Tools::CorrectEdgeTolerance: the reference is not a GeomAdaptor_Curve"))?.curve();
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&c3d, first, last, true, true)?).cast();
    let direction = p.get().expect("moved plane").position().direction();
    let on_plane = geom_proj_lib::project_on_plane(&trimmed, &p, direction, true)?;
    let curve: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_curve(&on_plane)?).cast();
    let pc = make_curve(&ProjectedCurve::with_curve(&gahs, &curve)?)?;
    let ghpc: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&pc, first, last)?).cast();
    let acs = Adaptor3dCurveOnSurface::with_curve(&ghpc, &gahs)?;
    raise_edge(e, validate(hc, &acs, ctx.tol, true)?, ctx.max_tol)
}

/// The static `CorrectEdgeTolerance(E, S, aMaxTol)`: the edge tolerance raised to the
/// distances between its reference curve and its curves on the face surface.
fn correct_edge_tolerance(e: &Edge, s: &Face, max_tol: f64) -> Result<(), Failure> {
    let te = edge_tshape(e, "TopOpeBRepBuild_Tools::CorrectEdgeTolerance")?;
    let (same_parameter, same_range) = (te.same_parameter(), te.same_range());
    if !same_range && same_parameter {
        return Ok(());
    }
    let Some(cref) = reference_representation(&te.curves(), te.degenerated())? else {
        return Ok(());
    };
    let r = cref.get().expect("reference");
    let gc = cref.cast::<dyn BRepGCurve>();
    let g = gc.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "TopOpeBRepBuild_Tools: not a BRep_GCurve",
        )
    })?;
    let Some(hcurve) = reference_adaptor(r, g)? else {
        return Ok(());
    };
    let ctx = EdgeContext {
        cref: &cref,
        hcurve: &hcurve,
        tol: BRepTool::edge_tolerance(e)?,
        same_parameter,
        same_range,
        max_tol,
    };
    let tf = face_tshape(s, "TopOpeBRepBuild_Tools::CorrectEdgeTolerance")?;
    let l = (s.location() * &tf.location())?.predivided(e.location())?;
    let su = tf.surface();
    match check_pcurves(e, &ctx, &su, &l)? {
        Some(false) => check_projection(e, &ctx, &su, &l),
        _ => Ok(()),
    }
}

impl TopOpeBRepBuildTools {
    /// `CorrectTolerances(aShape, aMaxTol)`: the vertex then the edge tolerances of the
    /// shape corrected (raised below `max_tol`);
    /// `TopOpeBRepBuildTools::correct_tolerances(&s, 0.0001)?`.
    pub fn correct_tolerances(shape: &Shape, max_tol: f64) -> Result<(), Failure> {
        Self::correct_point_on_curve(shape, max_tol)?;
        Self::correct_curve_on_surface(shape, max_tol)
    }

    /// `CorrectPointOnCurve(S, aMaxTol)`: the vertex tolerances of each edge raised to
    /// the distances of their points to its 3d curves;
    /// `TopOpeBRepBuildTools::correct_point_on_curve(&s, 0.0001)?`.
    pub fn correct_point_on_curve(s: &Shape, max_tol: f64) -> Result<(), Failure> {
        for e in sub_shapes(s, ShapeEnum::Edge)?.iter() {
            check_edge(&topods::edge(e)?, max_tol)?;
        }
        Ok(())
    }

    /// `CorrectCurveOnSurface(S, aMaxTol)`: the edge tolerances of each face raised to the
    /// distances of their curves on its surface to their reference curves;
    /// `TopOpeBRepBuildTools::correct_curve_on_surface(&s, 0.0001)?`.
    pub fn correct_curve_on_surface(s: &Shape, max_tol: f64) -> Result<(), Failure> {
        for f in sub_shapes(s, ShapeEnum::Face)?.iter() {
            let face = topods::face(f)?;
            for e in sub_shapes(f, ShapeEnum::Edge)?.iter() {
                correct_edge_tolerance(&topods::edge(e)?, &face, max_tol)?;
            }
        }
        Ok(())
    }

    /// `CheckFaceClosed2d(theFace)`: whether each wire of the face has a valid orientation
    /// in it (BRepCheck_Wire::Orientation without error);
    /// `TopOpeBRepBuildTools::check_face_closed2d(&f)?`.
    pub fn check_face_closed2d(face: &Face) -> Result<bool, Failure> {
        for w in Explorer::new(face.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)? {
            let chk = BRepCheckWire::new(&topods::wire(&w)?)?;
            if chk.orientation(face, false)? != BRepCheckStatus::NoError {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
