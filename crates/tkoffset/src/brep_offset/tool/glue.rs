// Rust translation of the static BSplineEdges, AngleWireEdge, ReconstructPCurves,
// ConcatPCurves, Glue, AreClosed and AssembleEdge of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Tool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The glueing of split section edges into one edge: their curves and p-curves
//! concatenated across autonomous vertices.
use super::inter3d::{GlueContext, is_autonom_vertex_on_faces};
use super::live;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBoundedCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine,
    Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkgeombase::geom_convert::{ApproxCurve, CompCurveToBSplineCurve};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkgeombase::geom2d_convert::{
    ApproxCurve as ApproxCurve2d, CompCurveToBSplineCurve as CompCurve2d,
};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
use std::f64::consts::{FRAC_PI_2, PI};

/// Whether two dynamic types are the same (`DynamicType() == DynamicType()`).
fn same_dynamic_type(
    a: Option<&'static rustycad_tkernel::standard::RuntimeType>,
    b: Option<&'static rustycad_tkernel::standard::RuntimeType>,
) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => std::ptr::eq(a, b) || a.name() == b.name(),
        _ => false,
    }
}

/// The approximation settings of `Glue` and `ConcatPCurves`.
const APPROX_TOL: f64 = 1.0e-7;
const APPROX_MAX_DEG: i32 = 14;
const APPROX_MAX_SEG: i32 = 16;

/// The 3d curve of the edge without trimming, and its range.
fn basis_curve(e: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let (mut c, f, l) = BRepTool::curve(e)?;
    if c.is_instance_name("Geom_TrimmedCurve") {
        c = live(&c.clone().cast::<GeomTrimmedCurve>(), "curve")?.basis_curve();
    }
    Ok((c, f, l))
}

/// `BSplineEdges(E1, E2, par1, par2, angle)`: the angle between the derivatives of
/// two B-spline edges at their first (0) or last (1) parameters (a right angle for a
/// null derivative); `None` when one is not a B-spline.
fn bspline_edges(e1: &Edge, e2: &Edge, par1: i32, par2: i32) -> Result<Option<f64>, Failure> {
    let (c1, first1, last1) = basis_curve(e1)?;
    let (c2, first2, last2) = basis_curve(e2)?;
    if !c1.is_instance_name("Geom_BSplineCurve") || !c2.is_instance_name("Geom_BSplineCurve") {
        return Ok(None);
    }
    let param1 = if par1 == 0 { first1 } else { last1 };
    let param2 = if par2 == 0 { first2 } else { last2 };
    let (_, der1) = live(&c1, "curve")?.d1(param1)?;
    let (_, der2) = live(&c2, "curve")?.d1(param2)?;
    if der1.magnitude() <= rustycad_tkmath::gp::resolution()
        || der2.magnitude() <= rustycad_tkmath::gp::resolution()
    {
        return Ok(Some(FRAC_PI_2));
    }
    Ok(Some(der1.angle(der2)?))
}

/// `AngleWireEdge(aWire, anEdge)`: the angle of the edge with the edge of the wire at
/// their common vertex (uninitialized natively when one is not a B-spline: 0 here).
pub(super) fn angle_wire_edge(a_wire: &Wire, an_edge: &Edge) -> Result<f64, Failure> {
    let (v11, v12) = topexp::wire_vertices(a_wire);
    let (v21, v22) = topexp::vertices(an_edge, false);
    let cv = if v11.is_same(&v21) || v11.is_same(&v22) {
        v11.clone()
    } else {
        v12.clone()
    };
    let mut first_edge = Edge::default();
    let (mut v11, mut v12) = (v11, v12);
    let mut itw = ShapeIterator::new(a_wire.as_ref(), true, true);
    while itw.more() {
        first_edge = topods::edge(itw.value()?)?;
        itw.next();
        let (v1, v2) = topexp::vertices(&first_edge, false);
        if v1.is_same(&cv) || v2.is_same(&cv) {
            v11 = v1;
            v12 = v2;
            break;
        }
    }
    let angle = |p1, p2| -> Result<f64, Failure> {
        Ok(bspline_edges(&first_edge, an_edge, p1, p2)?.unwrap_or(0.))
    };
    if v11.is_same(&cv) && v21.is_same(&cv) {
        return Ok(PI - angle(0, 0)?);
    }
    if v11.is_same(&cv) && v22.is_same(&cv) {
        return angle(0, 1);
    }
    if v12.is_same(&cv) && v21.is_same(&cv) {
        return angle(1, 0);
    }
    Ok(PI - angle(1, 1)?)
}

/// `ReconstructPCurves(anEdge)`: each p-curve of the edge replaced by the projection of
/// its 3d curve on the located surface.
pub(super) fn reconstruct_pcurves(an_edge: &Edge) -> Result<(), Failure> {
    let (c3d, f, l) = BRepTool::curve(an_edge)?;
    let tedge = edge_tshape(an_edge.as_ref(), "ReconstructPCurves")?;
    for curve_rep in tedge.curves() {
        let rep = live(&curve_rep, "curve representation")?;
        if !rep.is_curve_on_surface() {
            continue;
        }
        let the_surf = rep.surface()?;
        let the_loc = (an_edge.location() * &rep.location())?;
        let transformed: Handle<dyn GeomSurface> = live(&the_surf, "surface")?
            .transformed(the_loc.transformation())?
            .cast();
        let proj_pcurve = geom_proj_lib::curve2d_range(&c3d, (f, l), &transformed)?;
        if !proj_pcurve.is_null() {
            rep.set_pcurve(&proj_pcurve)?;
        }
    }
    Ok(())
}

/// The p-curve of the edge on the face without trimming, and its range.
fn basis_pcurve(e: &Edge, f: &Face) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let pc = BRepTool::curve_on_face(e, f)?;
    let mut c = pc.pcurve;
    if c.is_instance_name("Geom2d_TrimmedCurve") {
        c = live(&c.clone().cast::<Geom2dTrimmedCurve>(), "pcurve")?.basis_curve();
    }
    Ok((c, pc.first, pc.last))
}

/// The parameters of the points on the basis of a 2D line or conic.
fn conic_parameters(
    c: &Handle<dyn Geom2dCurve>,
    p1: Point2,
    p2: Point2,
) -> Result<Option<(f64, f64)>, Failure> {
    if c.is_instance_name("Geom2d_Line") {
        let lin = live(&c.clone().cast::<Geom2dLine>(), "line")?.lin2d();
        return Ok(Some((
            elclib::parameter(&lin, p1)?,
            elclib::parameter(&lin, p2)?,
        )));
    }
    if c.is_instance_name("Geom2d_Circle") {
        let circ = live(&c.clone().cast::<Geom2dCircle>(), "circle")?.circ2d()?;
        return Ok(Some((
            elclib::parameter(&circ, p1)?,
            elclib::parameter(&circ, p2)?,
        )));
    }
    if c.is_instance_name("Geom2d_Ellipse") {
        let elips = live(&c.clone().cast::<Geom2dEllipse>(), "ellipse")?.elips2d()?;
        return Ok(Some((
            elclib::parameter(&elips, p1)?,
            elclib::parameter(&elips, p2)?,
        )));
    }
    if c.is_instance_name("Geom2d_Parabola") {
        let parab = live(&c.clone().cast::<Geom2dParabola>(), "parabola")?.parab2d()?;
        return Ok(Some((
            elclib::parameter(&parab, p1)?,
            elclib::parameter(&parab, p2)?,
        )));
    }
    if c.is_instance_name("Geom2d_Hyperbola") {
        let hypr = live(&c.clone().cast::<Geom2dHyperbola>(), "hyperbola")?.hypr2d()?;
        return Ok(Some((
            elclib::parameter(&hypr, p1)?,
            elclib::parameter(&hypr, p2)?,
        )));
    }
    Ok(None)
}

/// `ConcatPCurves(E1, E2, F, After, newFirst, newLast)`: the p-curve of the glued edge
/// on the face: the common basis over both ranges, the basis line or conic of the first
/// over the parameters of the ends of the second, or the concatenated (and C1
/// approximated) B-spline; with its range.
fn concat_pcurves(
    e1: &Edge,
    e2: &Edge,
    f: &Face,
    after: bool,
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let (pcurve1, first1, last1) = basis_pcurve(e1, f)?;
    let (pcurve2, mut first2, mut last2) = basis_pcurve(e2, f)?;
    if pcurve1 == pcurve2 {
        return Ok((pcurve1, first1.min(first2), last1.max(last2)));
    }
    let same_type = same_dynamic_type(pcurve1.dynamic_type(), pcurve2.dynamic_type());
    if same_type
        && (pcurve1.is_instance_name("Geom2d_Line") || pcurve1.is_kind_name("Geom2d_Conic"))
    {
        let c2 = live(&pcurve2, "pcurve")?;
        let (p1, p2) = (c2.value(first2)?, c2.value(last2)?);
        if let Some((a, b)) = conic_parameters(&pcurve1, p1, p2)? {
            (first2, last2) = (a, b);
        }
        return Ok((pcurve1, first1.min(first2), last1.max(last2)));
    }
    let tc1: Handle<dyn Geom2dBoundedCurve> = Handle::new(Geom2dTrimmedCurve::new(
        &pcurve1, first1, last1, true, true,
    )?)
    .cast();
    let tc2: Handle<dyn Geom2dBoundedCurve> = Handle::new(Geom2dTrimmedCurve::new(
        &pcurve2, first2, last2, true, true,
    )?)
    .cast();
    let mut concat = CompCurve2d::with_curve(&tc1, ParameterisationType::TgtThetaOver2)?;
    concat.add(&tc2, precision::confusion(), after)?;
    let mut new_pcurve: Handle<dyn Geom2dCurve> = concat.bspline_curve().cast();
    if live(&new_pcurve, "pcurve")?.continuity() < Continuity::C1 {
        let approx = ApproxCurve2d::from_curve(
            &new_pcurve,
            APPROX_TOL,
            Continuity::C1,
            APPROX_MAX_SEG,
            APPROX_MAX_DEG,
        )?;
        if approx.has_result() {
            new_pcurve = approx.curve().cast();
        }
    }
    let c = live(&new_pcurve, "pcurve")?;
    let (nf, nl) = (c.first_parameter(), c.last_parameter());
    Ok((new_pcurve, nf, nl))
}

/// The face, its flag and the edges of the glueing of two edges.
struct GlueEnds<'a> {
    vfirst: &'a Vertex,
    vlast: &'a Vertex,
    after: bool,
    glue_tol: f64,
}

/// `Glue(E1, E2, Vfirst, Vlast, After, F1, addPCurve1, F2, addPCurve2, theGlueTol)`: the
/// edge of the common basis curve, of the basis line or conic of the first edge, or of
/// the concatenated B-spline (a null edge when the curves do not concatenate within
/// the tolerance), between the vertices, with concatenated p-curves on the flagged faces.
fn glue(e1: &Edge, e2: &Edge, ends: &GlueEnds, context: &GlueContext) -> Result<Edge, Failure> {
    let (c1, first1, last1) = basis_curve(e1)?;
    let (c2, first2, last2) = basis_curve(e2)?;
    let mut is_canonic = false;
    let (new_curve, fparam, lparam) = if c1 == c2 {
        (c1, first1.min(first2), last1.max(last2))
    } else if same_dynamic_type(c1.dynamic_type(), c2.dynamic_type())
        && (c1.is_instance_name("Geom_Line") || c1.is_kind_name("Geom_Conic"))
    {
        is_canonic = true;
        (c1, 0., 0.)
    } else {
        let Some(concat) = concat_curves((&c1, first1, last1), (&c2, first2, last2), ends)? else {
            return Ok(Edge::default());
        };
        concat
    };
    let new_edge = if is_canonic {
        BRepLibMakeEdge::from_curve_vertices(&new_curve, ends.vfirst, ends.vlast)?.edge()?
    } else {
        BRepLibMakeEdge::from_curve_vertices_params(
            &new_curve,
            ends.vfirst,
            ends.vlast,
            fparam,
            lparam,
        )?
        .edge()?
    };
    let builder = BRepBuilder::default();
    for (add, face) in [
        (context.add_pcurve1, context.f1),
        (context.add_pcurve2, context.f2),
    ] {
        if !add {
            continue;
        }
        let (new_pcurve, new_first, new_last) = concat_pcurves(e1, e2, face, ends.after)?;
        builder.update_edge_pcurve_on_face(&new_edge, &new_pcurve, face, 0.)?;
        builder.range_on_face(&new_edge, face, new_first, new_last)?;
    }
    Ok(new_edge)
}

/// A curve with its parameter range.
type TrimmedCurve = (Handle<dyn GeomCurve>, f64, f64);

/// The concatenated (and C1 approximated) B-spline of the trimmed curves with its
/// range, `None` when they do not concatenate.
fn concat_curves(
    (c1, first1, last1): (&Handle<dyn GeomCurve>, f64, f64),
    (c2, first2, last2): (&Handle<dyn GeomCurve>, f64, f64),
    ends: &GlueEnds,
) -> Result<Option<TrimmedCurve>, Failure> {
    let tc1: Handle<dyn GeomBoundedCurve> =
        Handle::new(GeomTrimmedCurve::new(c1, first1, last1, true, true)?).cast();
    let tc2: Handle<dyn GeomBoundedCurve> =
        Handle::new(GeomTrimmedCurve::new(c2, first2, last2, true, true)?).cast();
    let mut concat =
        CompCurveToBSplineCurve::with_curve(&tc1, ParameterisationType::TgtThetaOver2)?;
    if !concat.add(&tc2, ends.glue_tol, ends.after, true, 0)? {
        return Ok(None);
    }
    let mut new_curve: Handle<dyn GeomCurve> = concat.bspline_curve().cast();
    if live(&new_curve, "curve")?.continuity() < Continuity::C1 {
        let approx = ApproxCurve::from_curve(
            &new_curve,
            APPROX_TOL,
            Continuity::C1,
            APPROX_MAX_SEG,
            APPROX_MAX_DEG,
        )?;
        if approx.has_result() {
            new_curve = approx.curve().cast();
        }
    }
    let c = live(&new_curve, "curve")?;
    let (fp, lp) = (c.first_parameter(), c.last_parameter());
    Ok(Some((new_curve, fp, lp)))
}

/// `AreClosed(E1, E2)`: two edges sharing both vertices.
fn are_closed(e1: &Edge, e2: &Edge) -> bool {
    let (v11, v12) = topexp::vertices(e1, false);
    let (v21, v22) = topexp::vertices(e2, false);
    (v11.is_same(&v21) && v12.is_same(&v22)) || (v11.is_same(&v22) && v12.is_same(&v21))
}

/// `AssembleEdge(pDS, F1, F2, addPCurve1, addPCurve2, EdgesForConcat)`: the edges
/// glued one after another across autonomous vertices; a null edge when two edges
/// meet at a non-autonomous vertex or do not glue.
pub(super) fn assemble_edge(
    context: &GlueContext,
    edges_for_concat: &[Shape],
) -> Result<Edge, Failure> {
    let Some(first) = edges_for_concat.first() else {
        return Err(Failure::with_message(
            "Standard_OutOfRange",
            "AssembleEdge: no edge to concatenate",
        ));
    };
    let mut cur_edge = topods::edge(first)?;
    let mut glue_tol = precision::confusion();
    for shape in &edges_for_concat[1..] {
        let an_edge = topods::edge(shape)?;
        let Some((vfirst, vlast, after)) = glue_ends(context, &cur_edge, &an_edge, &mut glue_tol)?
        else {
            return Ok(Edge::default());
        };
        let ends = GlueEnds {
            vfirst: &vfirst,
            vlast: &vlast,
            after,
            glue_tol,
        };
        let new_edge = glue(&cur_edge, &an_edge, &ends, context)?;
        if new_edge.is_null() {
            return Ok(Edge::default());
        }
        cur_edge = new_edge;
    }
    Ok(cur_edge)
}

/// The end vertices and the `After` flag of the glueing of two edges: for a closed
/// pair, the vertex that is not autonomous; for an open pair, the outer vertices when
/// their common vertex is autonomous (whose tolerance becomes the glue tolerance).
fn glue_ends(
    context: &GlueContext,
    cur_edge: &Edge,
    an_edge: &Edge,
    glue_tol: &mut f64,
) -> Result<Option<(Vertex, Vertex, bool)>, Failure> {
    let ds = context.ds;
    if are_closed(cur_edge, an_edge) {
        let (v1, v2) = topexp::vertices(cur_edge, false);
        if is_autonom_vertex_on_faces(v1.as_ref(), ds, context.f1, context.f2)? {
            return Ok(Some((v2.clone(), v2, false)));
        }
        if is_autonom_vertex_on_faces(v2.as_ref(), ds, context.f1, context.f2)? {
            return Ok(Some((v1.clone(), v1, true)));
        }
        return Ok(None);
    }
    let cv = topexp::common_vertex(cur_edge, an_edge);
    let is_autonom_cv = match &cv {
        Some(cv) => is_autonom_vertex_on_faces(cv.as_ref(), ds, context.f1, context.f2)?,
        None => false,
    };
    let Some(cv) = cv.filter(|_| is_autonom_cv) else {
        return Ok(None);
    };
    *glue_tol = BRepTool::vertex_tolerance(&cv)?;
    let (v11, v12) = topexp::vertices(cur_edge, false);
    let (v21, v22) = topexp::vertices(an_edge, false);
    let ends = if v11.is_same(&cv) && v21.is_same(&cv) {
        (v22, v12)
    } else if v11.is_same(&cv) && v22.is_same(&cv) {
        (v21, v12)
    } else if v12.is_same(&cv) && v21.is_same(&cv) {
        (v11, v22)
    } else {
        (v11, v21)
    };
    Ok(Some((ends.0, ends.1, false)))
}
