// Rust translation of the second static BuildEdge (S, isUiso, ValIso, VFirst, VLast, Tol),
// UpdateEdge and IsDegen (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The iso edges of the sweep surfaces: new edges on an iso, existing edges put on an
//! iso, and the test of a degenerated surface.
use super::build_face::oriente;
use super::helpers::couture;
use super::same_param::{check_same_parameter, same_parameter};
use super::topology::null_edge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::gp::{Direction2, Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// Standard_NullObject for a missing geometry (a crash natively).
fn null(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepFill_Sweep: null {what}; expected a {what}"),
        "BRepFill_Sweep",
    )
}

/// The U or V iso of the surface.
fn iso_of(
    s: &dyn GeomSurface,
    is_uiso: bool,
    val_iso: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    if is_uiso {
        s.u_iso(val_iso)
    } else {
        s.v_iso(val_iso)
    }
}

/// Whether the iso is singular at the vertex: its middle within `tol` of the vertex
/// and its length (integrated within `tol / 4`) below `tol`.
fn singular_iso(
    iso: &dyn GeomCurve,
    iso_handle: &Handle<dyn GeomCurve>,
    v: &Vertex,
    tol: f64,
) -> Result<bool, Failure> {
    let p: Point3 = iso.d0((iso.first_parameter() + iso.last_parameter()) / 2.)?;
    if p.distance(&BRepTool::pnt(v)?) >= tol {
        return Ok(false);
    }
    let ac = GeomAdaptorCurve::with_curve(iso_handle)?;
    Ok(AbscissaPoint::length_with_tolerance(&ac as &dyn Adaptor3dCurve, tol / 4.)? < tol)
}

/// `BuildEdge(S, isUiso, ValIso, VFirst, VLast, Tol)`: the edge on an iso of the
/// surface between the vertices (a degenerated edge at a singular iso), with its line
/// p-curve and the tolerance of CheckSameParameter.
pub(super) fn build_iso_edge(
    s: &Handle<dyn GeomSurface>,
    is_uiso: bool,
    val_iso: f64,
    v_first: &Shape,
    v_last: &Shape,
    tol: f64,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let surface = s.get().ok_or_else(|| null("surface"))?;
    let iso_handle = iso_of(surface, is_uiso, val_iso)?;
    let iso = iso_handle.get().ok_or_else(|| null("iso curve"))?;
    let mut sing: bool = false;
    if v_first.is_same(v_last) {
        // A singular case?
        let v: Vertex = topods::vertex(v_first)?;
        let t: f64 = BRepTool::vertex_tolerance(&v)?.max(tol);
        sing = singular_iso(iso, &iso_handle, &v, t)?;
    }
    let e: Edge = if sing {
        let mut v = v_first.clone();
        let e = null_edge(&mut v)?;
        builder.degenerated(&e, true)?;
        e
    } else {
        edge_on_iso(
            &iso_handle,
            &topods::vertex(v_first)?,
            &topods::vertex(v_last)?,
        )?
    };
    // Associate the 2d curve.
    let (u_min, _, v_min, _) = surface.bounds();
    let l: Handle<dyn Geom2dCurve> = if is_uiso {
        Handle::new(Geom2dLine::from_point(
            Point2::new(val_iso, v_min - iso.first_parameter()),
            Direction2::Y,
        ))
        .cast()
    } else {
        Handle::new(Geom2dLine::from_point(
            Point2::new(u_min - iso.first_parameter(), val_iso),
            Direction2::X,
        ))
        .cast()
    };
    builder.update_edge_pcurve(
        &e,
        &l,
        s,
        &Location::default(),
        precision::confusion(),
        None,
    )?;
    if sing {
        builder.range_on_surface(
            &e,
            s,
            &Location::default(),
            iso.first_parameter(),
            iso.last_parameter(),
        )?;
    }
    let gahiso = GeomAdaptorCurve::with_curve(&iso_handle)?;
    let gahsurf = GeomAdaptorSurface::with_surface(s)?;
    let pcurve = l.get().ok_or_else(|| null("pcurve"))?;
    let (_, the_tol) = check_same_parameter(&gahiso, pcurve, &gahsurf, 1.0e-4)?;
    builder.update_edge_tolerance(&e, the_tol)?;
    Ok(e)
}

/// The edge of the iso curve between the vertices, in the direction where its ends are
/// nearer to them (the vertices enlarged to 1.01 times those distances when needed);
/// Standard_ConstructionError when the edge cannot be made.
fn edge_on_iso(iso: &Handle<dyn GeomCurve>, vf: &Vertex, vl: &Vertex) -> Result<Edge, Failure> {
    let curve = iso.get().ok_or_else(|| null("iso curve"))?;
    let (p1, p2) = (curve.first_parameter(), curve.last_parameter());
    let (pp1, pp2) = (curve.value(p1)?, curve.value(p2)?);
    let (t1, t2) = (
        BRepTool::vertex_tolerance(vf)?,
        BRepTool::vertex_tolerance(vl)?,
    );
    let p11: f64 = pp1.distance(&BRepTool::pnt(vf)?);
    let p22: f64 = pp2.distance(&BRepTool::pnt(vl)?);
    let p12: f64 = pp1.distance(&BRepTool::pnt(vl)?);
    let p21: f64 = pp2.distance(&BRepTool::pnt(vf)?);
    let fwd: bool = p11 < p12 && p22 < p21;
    let builder = BRepBuilder::default();
    let updates: [(bool, &Vertex, f64); 2] = if fwd {
        [(p11 >= t1, vf, p11), (p22 >= t2, vl, p22)]
    } else {
        [(p12 >= t2, vl, p12), (p21 >= t1, vf, p21)]
    };
    for (needed, v, d) in updates {
        if needed {
            builder.update_vertex_tolerance(v, 1.01 * d)?;
        }
    }
    let (a, b) = if fwd { (vf, vl) } else { (vl, vf) };
    let mut maker = BRepLibMakeEdge::new();
    maker.init_curve_vertices_params(iso, a, b, p1, p2)?;
    if !maker.is_done() {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            "BRepFill_Sweep::BuildEdge",
            "BRepFill_Sweep",
        ));
    }
    maker.edge()
}

/// The trimmed line p-curve of an iso and its range.
fn iso_pcurve(
    s: &dyn GeomSurface,
    is_uiso: bool,
    val_iso: f64,
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let (u_first, u_last, v_first, v_last) = s.bounds();
    let (l, f2d, l2d) = if is_uiso {
        (
            Geom2dLine::from_point(Point2::new(val_iso, 0.), Direction2::Y),
            v_first,
            v_last,
        )
    } else {
        (
            Geom2dLine::from_point(Point2::new(0., val_iso), Direction2::X),
            u_first,
            u_last,
        )
    };
    Ok((Handle::new(l).cast(), f2d, l2d))
}

/// `UpdateEdge(E, S, isUiso, ValIso)`: an existing edge put on an iso of the surface:
/// its line p-curve reversed when the edge runs the other way, made degenerated at a
/// singular iso, brought to the edge range, the vertices enlarged to the surface points,
/// the edge made SameParameter and its p-curves (both for a seam) stored.
pub(super) fn update_iso_edge(
    e: &Edge,
    s: &Handle<dyn GeomSurface>,
    is_uiso: bool,
    val_iso: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let surface = s.get().ok_or_else(|| null("surface"))?;
    let iso_handle = iso_of(surface, is_uiso, val_iso)?;
    let iso = iso_handle.get().ok_or_else(|| null("iso curve"))?;
    let (vf, vl) = topexp::vertices(e, false);
    let mut sing: bool = false;
    if vf.is_same(&vl) {
        sing = singular_iso(iso, &iso_handle, &vf, BRepTool::vertex_tolerance(&vf)?)?;
    }
    let (l, mut f2d, mut l2d) = iso_pcurve(surface, is_uiso, val_iso)?;
    let mut cl: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&l, f2d, l2d, true, true)?).cast();
    // Control the direction and the range.
    let (mut first, mut last) = BRepTool::range(e)?;
    if needs_reverse(e, surface, &cl, (f2d, first), (&vf, &vl), sing)? {
        let reversed = Geom2dTrimmedCurve::new(&l, f2d, l2d, true, true)?;
        reversed.reverse()?;
        (f2d, l2d) = (reversed.first_parameter(), reversed.last_parameter());
        cl = Handle::new(reversed).cast();
    }
    if sing {
        builder.update_edge_curve(e, &Handle::default(), 0.)?;
        builder.degenerated(e, true)?;
        builder.range(e, f2d, l2d, false)?;
        (first, last) = (f2d, l2d);
    }
    if first != f2d || last != l2d {
        let mut c2d: Handle<dyn Geom2dCurve> = Handle::default();
        geom_lib::same_range(
            precision::p_confusion(),
            &cl,
            (f2d, l2d),
            (first, last),
            &mut c2d,
        )?;
        cl = Handle::new(Geom2dTrimmedCurve::new(&c2d, first, last, true, true)?).cast();
    }
    update_end_vertices(e, surface, &cl, (first, last))?;
    if !sing {
        let (ok, r) = same_parameter(e, &mut cl, s, 1.0e-4)?;
        if ok {
            builder.update_edge_tolerance(e, r)?;
        }
    }
    store_iso_pcurves(e, s, &cl)?;
    // Attention to the case not SameRange on its shapes (PRO13551).
    if !BRepTool::same_range(e)? {
        builder.range_on_surface(e, s, &Location::default(), first, last)?;
    }
    Ok(())
}

/// Whether the p-curve runs against the edge: its start is nearer to the last vertex,
/// or, for a closed non-singular edge, its first tangent leaves the edge tangent by more
/// than 1e-4 radians.
fn needs_reverse(
    e: &Edge,
    s: &dyn GeomSurface,
    cl: &Handle<dyn Geom2dCurve>,
    (f2d, first): (f64, f64),
    (vf, vl): (&Vertex, &Vertex),
    sing: bool,
) -> Result<bool, Failure> {
    let curve = cl.get().ok_or_else(|| null("pcurve"))?;
    if !vf.is_same(vl) {
        let uv = curve.value(f2d)?;
        let p_on_s: Point3 = s.value(uv.x(), uv.y())?;
        return Ok(p_on_s.distance(&BRepTool::pnt(vl)?) < p_on_s.distance(&BRepTool::pnt(vf)?));
    }
    if sing {
        return Ok(false);
    }
    let (_, d_c3d) = BRepAdaptorCurve::with_edge(e)?.d1(first)?;
    let (p2d, v2d): (Point2, Vector2) = curve.d1(f2d)?;
    let d = s.d1(p2d.x(), p2d.y())?;
    let mut v3d = Vector3::default();
    v3d.set_linear_form(v2d.x(), d.d1u, v2d.y(), d.d1v);
    Ok(d_c3d.angle(v3d)? > 1.0e-4)
}

/// The vertices of the edge enlarged to their distance to the surface points at the
/// ends of the p-curve.
fn update_end_vertices(
    e: &Edge,
    s: &dyn GeomSurface,
    cl: &Handle<dyn Geom2dCurve>,
    (first, last): (f64, f64),
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let curve = cl.get().ok_or_else(|| null("pcurve"))?;
    for (t, v) in [
        (first, topexp::first_vertex(e, false)),
        (last, topexp::last_vertex(e, false)),
    ] {
        let uv = curve.value(t)?;
        let r: f64 = s.value(uv.x(), uv.y())?.distance(&BRepTool::pnt(&v)?);
        builder.update_vertex_tolerance(&v, r)?;
    }
    Ok(())
}

/// The p-curve stored on the surface, as the matching half of a seam when the edge
/// already has a p-curve there.
fn store_iso_pcurves(
    e: &Edge,
    s: &Handle<dyn GeomSurface>,
    cl: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let loc = Location::default();
    let pcurve = couture(e, s, &loc)?;
    if pcurve.is_null() {
        return builder.update_edge_pcurve(e, cl, s, &loc, precision::confusion(), None);
    }
    // A sewing edge.
    let mut oriented = e.clone();
    oriente(s, &mut oriented)?;
    if oriented.orientation() == Orientation::Reversed {
        builder.update_edge_pcurves(e, (cl, &pcurve), s, &loc, precision::confusion(), None)
    } else {
        builder.update_edge_pcurves(e, (&pcurve, cl), s, &loc, precision::confusion(), None)
    }
}

/// `IsDegen(S, Tol)`: the surface is degenerated in U (its U isos shorter than `tol`)
/// or in V.
pub(super) fn is_degen(s: &Handle<dyn GeomSurface>, tol: f64) -> Result<bool, Failure> {
    let surface = s.get().ok_or_else(|| null("surface"))?;
    let (u_min, u_max, v_min, v_max) = surface.bounds();
    if degenerated_direction(surface, true, (u_min, u_max), (v_min, v_max), tol)? {
        return Ok(true);
    }
    degenerated_direction(surface, false, (v_min, v_max), (u_min, u_max), tol)
}

/// The test of one direction: the three points of the middle iso within `tol`, then
/// five isos shorter than `tol`.
fn degenerated_direction(
    s: &dyn GeomSurface,
    u_isos: bool,
    (a_min, a_max): (f64, f64),
    (b_min, b_max): (f64, f64),
    tol: f64,
) -> Result<bool, Failure> {
    const NB: i32 = 5;
    let t: f64 = (a_min + a_max) / 2.;
    let at = |a: f64, b: f64| if u_isos { s.d0(a, b) } else { s.d0(b, a) };
    let (p1, p2, p3) = (at(t, b_min)?, at(t, (b_min + b_max) / 2.)?, at(t, b_max)?);
    let mut b: bool = p1.distance(&p2) + p2.distance(&p3) < tol;
    let dt: f64 = (a_max - a_min) / f64::from(NB + 1);
    let mut ii: i32 = 1;
    while b && ii <= NB {
        let ti: f64 = a_min + f64::from(ii) * dt;
        let iso = if u_isos { s.u_iso(ti)? } else { s.v_iso(ti)? };
        let ac = GeomAdaptorCurve::with_curve(&iso)?;
        b = AbscissaPoint::length_with_tolerance(&ac as &dyn Adaptor3dCurve, tol / 4.)? <= tol;
        ii += 1;
    }
    Ok(b)
}
