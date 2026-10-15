// Rust translation of the file static CurveHermite (ChFi3d_Builder_CnCrn.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The connecting curve crossing several faces: a cubic Hermite curve between the ends of
//! two contours (or sharp edges), projected on each face crossed, and its parameters on
//! the edges between the faces.
use crate::chfi_ds::{StripeHandle, lock};
use crate::chfi3d::surf_data_at;
use rustycad_tkbool::brep_algo::BRepAlgoNormalProjection;
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::{ExtCC, ExtPC};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkmath::math::Matrix;
use rustycad_tkmath::plib;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace};

/// One end of the Hermite curve: the contour (`CDic`, its side `jfic`, surface data,
/// parameter and sense) or the sharp edge.
pub(super) struct HermiteEnd<'a> {
    pub cd: Option<&'a StripeHandle>,
    pub jf: i32,
    pub sd: i32,
    pub par: f64,
    pub sens: i32,
    pub sharp: bool,
    pub evive: &'a Edge,
}

/// The projections of `CurveHermite`, appended to the sequences of the caller.
pub(super) struct Projections<'a> {
    pub proj2d: &'a mut Vec<Handle<dyn Geom2dCurve>>,
    pub cproj: &'a mut Vec<Handle<dyn GeomCurve>>,
    pub eproj: &'a mut Vec<Edge>,
}

/// The failure of a missing curve of an edge.
fn construction(what: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", what)
}

/// The 3d curve at an end: of the sharp edge, or of the line of the side of the contour
/// (`jf`, the other side of the second end).
fn end_curve(
    dstr: &TopOpeBRepDSDataStructure,
    end: &HermiteEnd<'_>,
    second: bool,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let c = if end.sharp {
        BRepTool::curve(end.evive)?.0
    } else {
        let jf = if second { 3 - end.jf } else { end.jf };
        let cd = end
            .cd
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "CurveHermite"))?;
        let fd = surf_data_at(cd, end.sd)?;
        let ilin = lock(&fd)
            .interference(if jf == 1 { 1 } else { 2 })
            .line_index();
        dstr.curve(ilin).curve().clone()
    };
    if c.is_null() {
        return Err(construction("Failed to get 3D curve of edge"));
    }
    Ok(c)
}

/// The Bezier curve of the cubic Hermite interpolation of the points and derivatives of
/// the two ends, the derivatives scaled by the chord and oriented by the senses.
fn hermite_bezier(
    dstr: &TopOpeBRepDSDataStructure,
    ends: [&HermiteEnd<'_>; 2],
) -> Result<(Handle<dyn GeomCurve>, Point3), Failure> {
    let c1 = end_curve(dstr, ends[0], false)?;
    let c2 = end_curve(dstr, ends[1], true)?;
    let (p01, d11) = c1
        .get()
        .ok_or_else(|| construction("CurveHermite"))?
        .d1(ends[0].par)?;
    let (p02, d12) = c2
        .get()
        .ok_or_else(|| construction("CurveHermite"))?
        .d1(ends[1].par)?;
    let mut mat = Matrix::with_bounds(1, 4, 1, 4)
        .map_err(|e| Failure::new(&format!("CurveHermite: {e:?}")))?;
    plib::hermite_coefficients(0.0, 1.0, 1, 1, &mut mat)?;
    let l1 = p01.distance(&p02);
    let lambda1 = 1.0 / f64::max(d11.magnitude() / l1, 1.0e-6);
    let lambda2 = 1.0 / f64::max(d12.magnitude() / l1, 1.0e-6);
    let cont: [Xyz; 4] = [
        p01.xyz(),
        *d11.xyz() * if ends[0].sens == 1 { -lambda1 } else { lambda1 },
        p02.xyz(),
        *d12.xyz() * if ends[1].sens == 1 { lambda2 } else { -lambda2 },
    ];
    let mut coeffs = points4()?;
    for ii in 1..=4 {
        for jj in 1..=4 {
            let mut x = coeffs[jj].xyz();
            x += mat[(ii, jj)] * cont[(ii - 1) as usize];
            coeffs[jj] = Point3::from(x);
        }
    }
    let mut poles = points4()?;
    plib::coefficients_poles(&coeffs, None, &mut poles, None)?;
    Ok((Handle::new(GeomBezierCurve::new(&poles)?).cast(), p01))
}

/// An array of four points at the origin, from 1.
fn points4() -> Result<Array1<'static, Point3>, Failure> {
    Array1::with_bounds(1, 4).map_err(|e| Failure::new(&format!("CurveHermite: {e:?}")))
}

/// The edges of the projection of a normal projection.
fn projected_edges(proj: &BRepAlgoNormalProjection) -> Result<IndexedMapOfShape, Failure> {
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(proj.projection(), ShapeEnum::Edge, &mut m)?;
    Ok(m)
}

/// The edges of the projection of `edg` on the face `f`, projected again on the whole
/// surface (the face `f` replaced) when there are several; None when not done.
fn project_on(edg: &Edge, f: &mut Face) -> Result<Option<IndexedMapOfShape>, Failure> {
    let mut ort = BRepAlgoNormalProjection::new();
    ort.init(f.as_ref());
    ort.add(edg.as_ref())?;
    ort.set_params(1.0e-4, 1.0e-4, Continuity::C1, 14, 16);
    ort.build()?;
    if !ort.is_done() {
        return Ok(None);
    }
    let mut map = projected_edges(&ort)?;
    if map.extent() > 1 {
        *f = BRepLibMakeFace::from_surface(&BRepTool::surface(f)?, precision::confusion())?
            .face()?;
        ort.init(f.as_ref());
        ort.build()?;
        map = IndexedMapOfShape::default();
        if ort.is_done() {
            map = projected_edges(&ort)?;
        }
    }
    Ok(Some(map))
}

/// The projection of the edge on one face appended (null curves when it is empty), and
/// the smallest tolerance of the projected edges in `error`.
fn append_projection(
    map: &IndexedMapOfShape,
    f: &Face,
    out: &mut Projections<'_>,
    error: &mut f64,
) -> Result<(), Failure> {
    let mut e1 = Edge::default();
    for ind in 1..=map.extent() {
        e1 = topods::edge(map.find_key(ind)?)?;
        if !BRepTool::degenerated(&e1)? {
            break;
        }
    }
    out.eproj.push(e1.clone());
    if map.extent() == 0 {
        out.proj2d.push(Handle::default());
        out.cproj.push(Handle::default());
        return Ok(());
    }
    let pc = BRepTool::curve_on_face(&e1, f)?;
    if pc.pcurve.is_null() {
        return Err(construction("Failed to get p-curve of edge"));
    }
    out.proj2d.push(
        Handle::new(Geom2dTrimmedCurve::new(
            &pc.pcurve, pc.first, pc.last, true, true,
        )?)
        .cast(),
    );
    let (c3, up1, up2) = BRepTool::curve(&e1)?;
    if c3.is_null() {
        return Err(construction("Failed to get 3D curve of edge"));
    }
    out.cproj
        .push(Handle::new(GeomTrimmedCurve::new(&c3, up1, up2, true, true)?).cast());
    *error = f64::min(*error, BRepTool::edge_tolerance(&e1)?);
    Ok(())
}

/// `CurveHermite(DStr, CDicmoins, ..., nbface, Ecom, Face, proj2d, cproj, Eproj, param,
/// error)`: the Hermite curve between the two ends projected on the `nbface` faces, and
/// the parameters of its crossings of the edges `Ecom` between them; the reached
/// tolerance (1e-30 unless a projected edge has a smaller one, as natively).
pub(super) fn curve_hermite(
    dstr: &TopOpeBRepDSDataStructure,
    ends: [&HermiteEnd<'_>; 2],
    (nbface, ecom, faces): (i32, &[Edge], &[Face]),
    out: &mut Projections<'_>,
    param: &mut [f64],
) -> Result<f64, Failure> {
    let (bezier, mut p01) = hermite_bezier(dstr, ends)?;
    let edg = BRepLibMakeEdge::from_curve(&bezier)?.edge()?;
    let mut error = 1.0e-30;
    for nb in 1..=nbface as usize {
        let mut f = faces
            .get(nb - 1)
            .cloned()
            .ok_or_else(|| Failure::out_of_range("CurveHermite: faces"))?;
        let Some(map) = project_on(&edg, &mut f)? else {
            continue;
        };
        append_projection(&map, &f, out, &mut error)?;
    }
    for nb in 1..nbface as usize {
        p01 = crossing(
            &bezier,
            (&ecom[nb - 1], &mut param[nb - 1]),
            (out.cproj.as_slice(), nb),
            p01,
        )?;
    }
    Ok(error)
}

/// The end of the projection next to the edge between the faces `nb` and `nb + 1`
/// (the last end of the projection on `nb`, else the first of the one on `nb + 1`);
/// `p01` unchanged when both are null.
fn projection_end(
    cproj: &[Handle<dyn GeomCurve>],
    nb: usize,
    p01: Point3,
) -> Result<Point3, Failure> {
    let at = |k: usize| {
        cproj
            .get(k - 1)
            .cloned()
            .ok_or_else(|| Failure::out_of_range("CurveHermite: projections"))
    };
    if let Some(c) = at(nb)?.get() {
        return c.value(c.last_parameter());
    }
    if let Some(c) = at(nb + 1)?.get() {
        return c.value(c.first_parameter());
    }
    Ok(p01)
}

/// The parameter on the edge `ecom` of the crossing of the Hermite curve (by the
/// extrema between them, else by the projection of the end of the projected curves).
fn crossing(
    bezier: &Handle<dyn GeomCurve>,
    (ecom, param): (&Edge, &mut f64),
    (cproj, nb): (&[Handle<dyn GeomCurve>], usize),
    mut p01: Point3,
) -> Result<Point3, Failure> {
    let c = BRepAdaptorCurve::with_edge(ecom)?;
    let p02 = c.value(*param)?;
    let l = GeomAdaptorCurve::with_curve(bezier)?;
    let ext = ExtCC::with_curves(&c, &l, 1.0e-10, 1.0e-10)?;
    if ext.is_done() && !ext.is_parallel()? && ext.nb_ext()? != 0 {
        let (ponc, ponl) = ext.points(1)?;
        if ponc.value().distance(ponl.value()) < precision::confusion() {
            *param = ponc.parameter();
        } else {
            p01 = projection_end(cproj, nb, p01)?;
        }
    }
    if !ext.is_done() || ext.nb_ext()? == 0 {
        p01 = projection_end(cproj, nb, p01)?;
        if p01.distance(&p02) > 1.0e-4 {
            let ext1 = ExtPC::with_curve(p01, &c as &dyn Adaptor3dCurve, 1.0e-10)?;
            if ext1.is_done() && ext1.nb_ext()? != 0 {
                *param = ext1.point(1)?.parameter();
            }
        }
    }
    Ok(p01)
}
