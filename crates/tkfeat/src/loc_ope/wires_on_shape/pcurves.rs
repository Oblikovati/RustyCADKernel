// Rust translation of the static PutPCurve and PutPCurves of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_WiresOnShape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The p-curves of the splitting edges on the faces they split: projected on a
//! non-planar face when missing or outside its bounds, and copied from the edge of the
//! shape an edge of a wire lies on (both p-curves on a seam).
use super::project::{located_curve, project_2d};
use crate::errors::{construction_error, null_object};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkshhealing::shape_construct::ShapeConstructProjectCurveOnSurface;
use rustycad_tktopalgo::brep_lib;

/// The face bounds `(Umin, Umax, Vmin, Vmax)`.
type UvBounds = (f64, f64, f64, f64);

/// The forward and reversed p-curves of a seam edge.
type SeamPCurves = (Handle<dyn Geom2dCurve>, Handle<dyn Geom2dCurve>);

/// Whether the basis of the (unlocated) surface of `fac` is a plane, a rectangular
/// trimmed surface standing for its basis.
fn is_planar(fac: &Face) -> Result<bool, Failure> {
    let (mut s, _) = BRepTool::surface_with_location(fac)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    if let Some(trimmed) = trimmed.get() {
        s = trimmed.basis_surface();
    }
    let plane: Handle<GeomPlane> = s.cast();
    Ok(!plane.is_null())
}

/// The located 3D curve of `edg`, trimmed to its range unless it is already trimmed.
fn trimmed_curve(edg: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let (c, f, l) = located_curve(edg)?;
    let as_trimmed: Handle<GeomTrimmedCurve> = c.cast();
    if !as_trimmed.is_null() {
        return Ok((c, f, l));
    }
    Ok((
        Handle::new(GeomTrimmedCurve::new(&c, f, l, true, true)?).cast(),
        f,
        l,
    ))
}

/// The number of periods to shift `[min(a, b), max(a, b)]` into `[lo, hi]`: up while
/// its low end is below, else down while its high end is above.
fn period_shift(a: f64, b: f64, lo: f64, hi: f64, period: f64) -> i32 {
    let tol = precision::p_confusion(); // Epsilon(up);
    let (mut low, mut high) = (a.min(b), a.max(b));
    let mut nbtra = 0;
    if low < lo - tol {
        while low < lo - tol {
            low += period;
            nbtra += 1;
        }
    } else if high > hi + tol {
        while high > hi + tol {
            high -= period;
            nbtra -= 1;
        }
    }
    nbtra
}

/// Translates `c2d` by whole periods of `s` so that its ends `pf`, `pl` fall in the
/// face bounds.
fn shift_into_period(
    c2d: &Handle<dyn Geom2dCurve>,
    s: &dyn GeomSurface,
    pf: Point2,
    pl: Point2,
    b: UvBounds,
) -> Result<(), Failure> {
    let curve = c2d
        .get()
        .ok_or_else(|| null_object("LocOpe_WiresOnShape::PutPCurve", "null p-curve"))?;
    if s.is_u_periodic() {
        let up = s.u_period()?;
        let nbtra = period_shift(pf.x(), pl.x(), b.0, b.1, up);
        if nbtra != 0 {
            curve.translate(Vector2::new(f64::from(nbtra) * up, 0.))?;
        }
    }
    if s.is_v_periodic() {
        let vp = s.v_period()?;
        let nbtra = period_shift(pf.y(), pl.y(), b.2, b.3, vp);
        if nbtra != 0 {
            curve.translate(Vector2::new(0., f64::from(nbtra) * vp))?;
        }
    }
    Ok(())
}

/// Whether the middle of the p-curve of `edg` on `fac` lies in the face bounds.
fn pcurve_inside(edg: &Edge, fac: &Face, b: UvBounds) -> Result<bool, Failure> {
    let pc = BRepTool::curve_on_face(edg, fac)?;
    let Some(c2d) = pc.pcurve.get() else {
        return Ok(false);
    };
    let p2d = c2d.d0((pc.first + pc.last) * 0.5)?;
    let tol = precision::p_confusion();
    let out_u = p2d.x() < b.0 - tol || p2d.x() > b.1 + tol;
    let out_v = p2d.y() < b.2 - tol || p2d.y() > b.3 + tol;
    Ok(!out_u && !out_v)
}

/// The tolerance of a vertex, -1 for a null one.
fn vertex_tolerance_or(v: &Vertex) -> Result<f64, Failure> {
    if v.is_null() {
        return Ok(-1.);
    }
    BRepTool::vertex_tolerance(v)
}

/// The native handling of "internal vertices", reached when the edge has a first
/// vertex and no last one: the tolerance of the null last vertex is read, which fails.
fn update_internal_vertices(
    edg: &Edge,
    ends: (Point2, Point2),
    s: &dyn GeomSurface,
) -> Result<(), Failure> {
    let reversed = edg.orientation() == Orientation::Reversed;
    let mut v1 = if reversed {
        topexp::last_vertex(edg, false)
    } else {
        topexp::first_vertex(edg, false)
    };
    let mut v2 = if reversed {
        topexp::first_vertex(edg, false)
    } else {
        topexp::last_vertex(edg, false)
    };
    if reversed {
        v1.reverse();
        v2.reverse();
    }
    if v1.is_null() || !v2.is_null() {
        return Ok(());
    }
    let pf = s.d0(ends.0.x(), ends.0.y())?;
    let pl = s.d0(ends.1.x(), ends.1.y())?;
    let b = BRepBuilder::default();
    let old1 = BRepTool::vertex_tolerance(&v1)?;
    let old2 = BRepTool::vertex_tolerance(&v2)?;
    let tol1 = BRepTool::pnt(&v1)?.distance(&pf);
    let tol2 = BRepTool::pnt(&v2)?.distance(&pl);
    b.update_vertex_tolerance(&v1, old1.max(tol1))?;
    b.update_vertex_tolerance(&v2, old2.max(tol2))
}

/// `PutPCurve(Edg, Fac)`: on a non-planar face without a p-curve of `edg` inside its
/// bounds, projects the 3D curve (ShapeConstruct, Confusion), shifts it by periods into
/// the bounds, stores it and recomputes the same parameter.
pub(super) fn put_pcurve(edg: &Edge, fac: &Face) -> Result<(), Failure> {
    if is_planar(fac)? {
        return Ok(());
    }
    let bounds = brep_tools::uv_bounds(fac)?;
    if pcurve_inside(edg, fac, bounds)? {
        return Ok(());
    }
    let (c, f, l) = trimmed_curve(edg)?;
    let s = BRepTool::surface(fac)?;
    let (v1, v2) = topexp::vertices(edg, false);
    let (tol_first, tol_last) = (vertex_tolerance_or(&v1)?, vertex_tolerance_or(&v2)?);
    let tol2d = precision::confusion();
    let mut c2d: Handle<dyn Geom2dCurve> = Handle::default();
    let tool_proj = ShapeConstructProjectCurveOnSurface::new();
    tool_proj.init(&s, tol2d)?;
    tool_proj.perform(&c, (f, l), &mut c2d, (tol_first, tol_last))?;
    let Some(curve2d) = c2d.get() else {
        return Ok(());
    };
    let pf = curve2d.value(f)?;
    let pl = curve2d.value(l)?;
    let surface = s
        .get()
        .ok_or_else(|| null_object("LocOpe_WiresOnShape::PutPCurve", "face without surface"))?;
    update_internal_vertices(edg, (pf, pl), surface)?;
    shift_into_period(&c2d, surface, pf, pl, bounds)?;
    let b = BRepBuilder::default();
    b.update_edge_pcurve_on_face(edg, &c2d, fac, BRepTool::edge_tolerance(edg)?)?;
    b.same_parameter(edg, false)?;
    brep_lib::same_parameter_edge(edg, tol2d)
}

/// The faces of `my_shape` holding `eto`, once per occurrence.
fn faces_of(eto: &Edge, my_shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut lfaces = Vec::new();
    for face in Explorer::new(my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        for e in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if e.is_same(eto) {
                lfaces.push(face.clone());
            }
        }
    }
    Ok(lfaces)
}

/// The basis surface of a face surface, a rectangular trimmed surface standing for its
/// basis.
fn basis_of(s: Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    match trimmed.get() {
        Some(trimmed) => trimmed.basis_surface(),
        None => s,
    }
}

/// The projection of the trimmed 3D curve of `efrom` on the surface `s` restricted to
/// the bounds of `fac`, at the 2D resolution of the larger of the edge and face
/// tolerances.
fn projected_pcurve(
    efrom: &Edge,
    fac: &Face,
    s: &Handle<dyn GeomSurface>,
) -> Result<(Handle<dyn Geom2dCurve>, UvBounds), Failure> {
    let bounds = brep_tools::uv_bounds(fac)?;
    let (c, _, _) = trimmed_curve(efrom)?;
    // Compute the tol2d
    let tol3d = BRepTool::edge_tolerance(efrom)?.max(BRepTool::face_tolerance(fac)?);
    let gas = GeomAdaptorSurface::with_bounds(s, bounds)?;
    let tol2d = gas.u_resolution(tol3d)?.max(gas.v_resolution(tol3d)?);
    let (c2d, _) = geom_proj_lib::curve2d_bounds_tolerance(&c, s, bounds, tol2d)?;
    Ok((c2d, bounds))
}

/// One face of two distinct faces sharing `eto`: projects `efrom` on it when it has no
/// p-curve there and the face is not planar. False when the projection failed (the
/// native function returns then).
fn put_pcurve_on_neighbour(efrom: &Edge, fac: &Face) -> Result<bool, Failure> {
    if !BRepTool::curve_on_face(efrom, fac)?.pcurve.is_null() || is_planar(fac)? {
        return Ok(true);
    }
    let s = BRepTool::surface(fac)?;
    let (c2d, bounds) = projected_pcurve(efrom, fac, &s)?;
    let Some(curve2d) = c2d.get() else {
        return Ok(false);
    };
    let (_, f, l) = located_curve(efrom)?;
    let pf = curve2d.value(f)?;
    let pl = curve2d.value(l)?;
    let surface = s
        .get()
        .ok_or_else(|| null_object("LocOpe_WiresOnShape::PutPCurves", "face without surface"))?;
    shift_into_period(&c2d, surface, pf, pl, bounds)?;
    BRepBuilder::default().update_edge_pcurve_on_face(
        efrom,
        &c2d,
        fac,
        BRepTool::edge_tolerance(efrom)?,
    )?;
    Ok(true)
}

/// The p-curve of the edge `e` with orientation `or` on the FORWARD face `fac`.
fn oriented_pcurve(
    e: &Edge,
    or: Orientation,
    fac: &Face,
) -> Result<(Handle<dyn Geom2dCurve>, f64), Failure> {
    let local_e = topods::edge(&e.oriented(or))?;
    let local_f = topods::face(&fac.oriented(Orientation::Forward))?;
    let pc = BRepTool::curve_on_face(&local_e, &local_f)?;
    Ok((pc.pcurve, pc.first))
}

/// `c2d` translated by `shift` when the shift is above the epsilon of `reference`.
fn shifted(
    c2d: Handle<dyn Geom2dCurve>,
    shift: f64,
    reference: f64,
    along_u: bool,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if shift.abs() <= epsilon(reference) {
        return Ok(c2d);
    }
    let curve = c2d
        .get()
        .ok_or_else(|| null_object("LocOpe_WiresOnShape::PutPCurves", "null p-curve"))?;
    let vector = if along_u {
        Vector2::new(shift, 0.)
    } else {
        Vector2::new(0., shift)
    };
    Ok(curve.translated(vector)?.cast())
}

/// Whether `efrom` and `eto` run in the same direction, comparing the tangent of `efrom`
/// at its first parameter with the tangent of `eto` at the projection of the first
/// vertex of `efrom`.
fn same_orientation(efrom: &Edge, eto: &Edge, fac: &Face) -> Result<bool, Failure> {
    let (c, f, _) = located_curve(efrom)?;
    let curve = c
        .get()
        .ok_or_else(|| null_object("LocOpe_WiresOnShape::PutPCurves", "edge without 3D curve"))?;
    let (_, d1f) = curve.d1(f)?;
    let first_vertex = topexp::first_vertex(efrom, false);
    let vtx_param = BRepTool::parameter(&first_vertex, efrom)?;
    let p2d = BRepAdaptorCurve2d::with_edge_on_face(efrom, fac)?.value(vtx_param)?;
    let prmproj = project_2d(p2d, eto, fac)?;
    let (c, _, _) = located_curve(eto)?;
    let curve = c
        .get()
        .ok_or_else(|| null_object("LocOpe_WiresOnShape::PutPCurves", "edge without 3D curve"))?;
    let (_, d1t) = curve.d1(prmproj)?;
    Ok(d1t.dot(d1f) > 0.)
}

/// The two p-curves of `efrom` on the seam face `fac`: those it has (one standing for
/// the missing other), or the projection of its 3D curve on the basis surface.
fn seam_pcurves(efrom: &Edge, fac: &Face) -> Result<SeamPCurves, Failure> {
    let (c2dff, _) = oriented_pcurve(efrom, Orientation::Forward, fac)?;
    let (c2dfr, _) = oriented_pcurve(efrom, Orientation::Reversed, fac)?;
    if c2dff.is_null() && c2dfr.is_null() {
        let s = basis_of(BRepTool::surface(fac)?);
        let (c2d, _) = projected_pcurve(efrom, fac, &s)?;
        return Ok((c2d.clone(), c2d));
    }
    if c2dfr.is_null() {
        return Ok((c2dff.clone(), c2dff));
    }
    if c2dff.is_null() {
        return Ok((c2dfr.clone(), c2dfr));
    }
    Ok((c2dff, c2dfr))
}

/// The coordinate along the iso direction: X on a U iso, Y otherwise.
fn iso_coord(p: Point2, iso_u: bool) -> f64 {
    if iso_u { p.x() } else { p.y() }
}

/// `efrom` on the seam `eto` of `fac`: its two p-curves set on the two sides of the
/// seam, matched by orientation.
fn put_pcurves_on_seam(efrom: &Edge, eto: &Edge, fac: &Face) -> Result<(), Failure> {
    if !BRepTool::is_closed_on_face(eto, fac)? {
        return Err(construction_error(
            "LocOpe_WiresOnShape::PutPCurves",
            "the edge of two faces is not closed on its single face",
        ));
    }
    let (c2dtf, _) = oriented_pcurve(eto, Orientation::Forward, fac)?;
    // The native parameter is the first one of the last p-curve read.
    let (c2dtr, f) = oriented_pcurve(eto, Orientation::Reversed, fac)?;
    let null_seam = || null_object("LocOpe_WiresOnShape::PutPCurves", "seam without p-curves");
    let ptf = c2dtf.get().ok_or_else(null_seam)?.value(f)?; // sur courbe frw
    let ptr = c2dtr.get().ok_or_else(null_seam)?.value(f)?; // sur courbe rev
    let iso_u = (ptf.y() - ptr.y()).abs() < epsilon(ptf.x()); // meme V
    // Efrom et Eto dans le meme sens???
    let same_ori = same_orientation(efrom, eto, fac)?;
    let (c2dff, c2dfr) = seam_pcurves(efrom, fac)?;
    let (f, _) = BRepTool::range(efrom)?;
    let null_from = || {
        null_object(
            "LocOpe_WiresOnShape::PutPCurves",
            "null p-curve of the splitting edge",
        )
    };
    let p2f = c2dff.get().ok_or_else(null_from)?.value(f)?;
    let p2r = c2dfr.get().ok_or_else(null_from)?.value(f)?;
    let (tf, tr) = (iso_coord(ptf, iso_u), iso_coord(ptr, iso_u));
    let (target_f, target_r) = if same_ori { (tf, tr) } else { (tr, tf) };
    let c2dff = shifted(c2dff, target_f - iso_coord(p2f, iso_u), target_f, iso_u)?;
    let c2dfr = shifted(c2dfr, target_r - iso_coord(p2r, iso_u), target_r, iso_u)?;
    BRepBuilder::default().update_edge_pcurves_on_face(
        efrom,
        (&c2dff, &c2dfr),
        fac,
        BRepTool::edge_tolerance(efrom)?,
    )
}

/// `PutPCurves(Efrom, Eto, myShape)`: the p-curves of the wire edge `efrom` on the
/// faces of the shape edge `eto` it lies on: nothing for a free boundary, the
/// projections on the two neighbour faces, or both p-curves on a seam.
pub(super) fn put_pcurves(efrom: &Edge, eto: &Edge, my_shape: &Shape) -> Result<(), Failure> {
    let lfaces = faces_of(eto, my_shape)?;
    if lfaces.len() != 1 && lfaces.len() != 2 {
        return Err(construction_error(
            "LocOpe_WiresOnShape::PutPCurves",
            "the edge of the shape has neither one nor two faces",
        ));
    }
    // soit bord libre, soit connexite entre 2 faces, eventuellement edge closed
    if lfaces.len() == 1 {
        return Ok(()); // sera fait par PutPCurve.... on l`espere
    }
    if lfaces[0].is_same(&lfaces[1]) {
        return put_pcurves_on_seam(efrom, eto, &topods::face(&lfaces[0])?);
    }
    for face in &lfaces {
        if !put_pcurve_on_neighbour(efrom, &topods::face(face)?)? {
            return Ok(());
        }
    }
    Ok(())
}
