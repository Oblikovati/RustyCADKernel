// Rust translation of the data structure functions of ChFi3d_Builder_0.cxx (ChFi3d_SetPointTolerance,
// ChFi3d_EnlargeBox, ChFi3d_ExtrSpineCarac, ChFi3d_IsInFront, ChFi3d_FilCurveInDS,
// ChFi3d_SolidIndex, ChFi3d_IndexPointInDS, ChFi3d_FilPointInDS, ChFi3d_FilVertexInDS,
// ChFi3d_StripeEdgeInter, ChFi3d_BoundSurf, ChFi3d_SearchFD) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The free functions of the fillet builders that read or write the TopOpeBRepDS data
//! structure.
use super::traces::int_traces;
use crate::chfi_ds::{CommonPoint, Map, SpineHandle, StripeHandle, SurfDataHandle, lock};
use crate::chfi3d::same_side;
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSCurvePointInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSInterference,
    TopOpeBRepDSKind, TopOpeBRepDSPoint, TopOpeBRepDSSurfaceCurveInterference,
    TopOpeBRepDSTransition,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Face, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

/// Standard_NullObject of a null handle read natively without check.
pub(crate) fn null(what: &str) -> Failure {
    Failure::with_message("Standard_NullObject", what)
}

/// The static `Geometry(DStr, ind)`: the face (positive index) or the surface (negative
/// index) of the structure as an adaptor (null for 0 or a null shape or surface).
pub fn ds_geometry(
    dstr: &TopOpeBRepDSDataStructure,
    ind: i32,
) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    if ind == 0 {
        return Ok(Handle::default());
    }
    if ind > 0 {
        let f = topods::face(dstr.shape(ind, true))?;
        if f.is_null() {
            return Ok(Handle::default());
        }
        let hs = BRepAdaptorSurface::default();
        hs.initialize(&f, false)?;
        return Ok(Handle::new(hs).cast());
    }
    let s = dstr.surface(-ind).surface().clone();
    if s.is_null() {
        return Ok(Handle::default());
    }
    Ok(Handle::new(GeomAdaptorSurface::with_surface(&s)?).cast())
}

/// `ChFi3d_SetPointTolerance(DStr, box, IP)`: the tolerance of the point set to 1.5 times
/// the diagonal of the box; `set_point_tolerance(&mut ds, &b, 1)?`.
pub fn set_point_tolerance(
    dstr: &mut TopOpeBRepDSDataStructure,
    bx: &BoundingBox3,
    ip: i32,
) -> Result<(), Failure> {
    let [a, b, c, mut d, mut e, mut f] = bx.get()?;
    d -= a;
    e -= b;
    f -= c;
    d *= d;
    e *= e;
    f *= f;
    let vtol = (d + e + f).sqrt() * 1.5;
    dstr.change_point(ip)?.set_tolerance(vtol);
    Ok(())
}

/// The points of an interference end added to a box: on its 3d curve, its pcurve on the
/// fillet surface and its pcurve on the face.
fn add_interference_end(
    dstr: &TopOpeBRepDSDataStructure,
    sd: &SurfDataHandle,
    on_s: i32,
    isfirst: bool,
    bx: &mut BoundingBox3,
) -> Result<f64, Failure> {
    let g = lock(sd);
    let fi = g.interference(on_s).clone();
    let s = dstr.surface(g.surf()).surface().clone();
    let f = ds_geometry(dstr, g.index(on_s))?;
    drop(g);
    let p = fi.parameter(isfirst);
    let c3d = dstr.curve(fi.line_index()).curve().clone();
    if let Some(c) = c3d.get() {
        bx.add_point(c.value(p)?);
    }
    if let Some(pcs) = fi.pcurve_on_surf().get() {
        let uv = pcs.value(p)?;
        bx.add_point(
            s.get()
                .ok_or_else(|| null("ChFi3d_EnlargeBox: null fillet surface"))?
                .value(uv.x(), uv.y())?,
        );
    }
    if let (Some(fs), Some(pcf)) = (f.get(), fi.pcurve_on_face().get()) {
        let uv = pcf.value(p)?;
        bx.add_point(fs.value(uv.x(), uv.y())?);
    }
    Ok(p)
}

/// `ChFi3d_EnlargeBox(DStr, st, sd, b1, b2, isfirst)`: the boxes of the ends on the two
/// faces of the fillet extended by its curves and, with a stripe, by its end curve;
/// `enlarge_box_ds(&ds, Some(&st), &sd, (&mut b1, &mut b2), true)?`.
pub fn enlarge_box_ds(
    dstr: &TopOpeBRepDSDataStructure,
    st: Option<&StripeHandle>,
    sd: &SurfDataHandle,
    (b1, b2): (&mut BoundingBox3, &mut BoundingBox3),
    isfirst: bool,
) -> Result<(), Failure> {
    {
        let g = lock(sd);
        b1.add_point(g.vertex(isfirst, 1).point());
        b2.add_point(g.vertex(isfirst, 2).point());
    }
    add_interference_end(dstr, sd, 1, isfirst, b1)?;
    add_interference_end(dstr, sd, 2, isfirst, b2)?;
    let Some(st) = st else {
        return Ok(());
    };
    let s = dstr.surface(lock(sd).surf()).surface().clone();
    let g = lock(st);
    let c3d = dstr.curve(g.curve(isfirst)).curve().clone();
    let c2d = g.pcurve(isfirst).clone();
    let (a, b) = g.parameters(isfirst);
    let (p1, p2) = if g.orientation_at_end(isfirst) == Orientation::Forward {
        (a, b)
    } else {
        (b, a)
    };
    drop(g);
    if let Some(c) = c3d.get() {
        b1.add_point(c.value(p1)?);
        b2.add_point(c.value(p2)?);
    }
    if let Some(c) = c2d.get() {
        let ss = s
            .get()
            .ok_or_else(|| null("ChFi3d_EnlargeBox: null fillet surface"))?;
        let uv = c.value(p1)?;
        b1.add_point(ss.value(uv.x(), uv.y())?);
        let uv = c.value(p2)?;
        b2.add_point(ss.value(uv.x(), uv.y())?);
    }
    Ok(())
}

/// `ChFi3d_ExtrSpineCarac(DStr, cd, i, p, jf, sens, P, V, R)`: `(P, V, R)`, the point of
/// the trace `jf` of the fillet `i` at `p`, the unit direction of the spine there
/// (reversed for `sens` 1) and the radius; `extr_spine_carac(&ds, &cd, 1, 0.5, 1, 1)?`.
pub fn extr_spine_carac(
    dstr: &TopOpeBRepDSDataStructure,
    cd: &StripeHandle,
    i: i32,
    p: f64,
    jf: i32,
    sens: i32,
) -> Result<(Point3, Vector3, f64), Failure> {
    let (sd, sp) = {
        let g = lock(cd);
        let seq = g
            .set_of_surf_data()
            .ok_or_else(|| null("ChFi3d_ExtrSpineCarac: no surface data"))?
            .clone();
        let sd = lock(&seq)
            .get(usize::try_from(i - 1).unwrap_or(usize::MAX))
            .cloned()
            .ok_or_else(|| Failure::out_of_range("ChFi3d_ExtrSpineCarac: no surface data i"))?;
        (sd, g.spine().cloned())
    };
    let (fffil, pc) = {
        let g = lock(&sd);
        (
            dstr.surface(g.surf()).surface().clone(),
            g.interference(jf).pcurve_on_surf().clone(),
        )
    };
    let pp = pc
        .get()
        .ok_or_else(|| null("ChFi3d_ExtrSpineCarac: null pcurve"))?
        .value(p)?;
    let surf = fffil
        .get()
        .ok_or_else(|| null("ChFi3d_ExtrSpineCarac: null surface"))?;
    let gs = GeomAdaptorSurface::with_surface(&fffil)?;
    let pt = surf.value(pp.x(), pp.y())?;
    let (r, mut v) = match gs.get_type()? {
        SurfaceType::Cylinder => {
            let cyl = gs.cylinder()?;
            (cyl.radius(), elslib::d1(pp.x(), pp.y(), &cyl).2)
        }
        SurfaceType::Torus => {
            let tor = gs.torus()?;
            (tor.minor_radius(), elslib::d1(pp.x(), pp.y(), &tor).1)
        }
        _ => spine_radius_direction(
            sp.as_ref()
                .ok_or_else(|| null("ChFi3d_ExtrSpineCarac: no spine"))?,
            p,
        )?,
    };
    v = v.normalized()?;
    if sens == 1 {
        v.reverse();
    }
    Ok((pt, v, r))
}

/// The radius of the fillet spine (or of its law) at `p` and the derivative of its
/// elementary spine.
fn spine_radius_direction(sp: &SpineHandle, p: f64) -> Result<(f64, Vector3), Failure> {
    let g = lock(sp);
    let hels = if g.nb_edges() == 1 {
        g.el_spine_of_index(1)?
    } else {
        g.el_spine_at(p)?
    };
    let fsp = g
        .as_fil_spine()
        .ok_or_else(|| null("ChFi3d_ExtrSpineCarac: not a fillet spine"))?;
    let r = if fsp.is_constant() {
        fsp.radius()?
    } else {
        fsp.law(&hels)?.value(p)?
    };
    let (_, v) = hels
        .get()
        .ok_or_else(|| null("ChFi3d_ExtrSpineCarac: null elementary spine"))?
        .d1(p)?;
    Ok((r, v))
}

/// The result of `ChFi3d_IsInFront`.
#[derive(Clone, Debug, Default)]
pub struct InFront {
    /// The parameters on the traces.
    pub p: (f64, f64),
    /// The common face.
    pub face: Face,
    /// Whether the fillets are on the same side of the face.
    pub sameside: bool,
    /// The interferences on the common face.
    pub jf: (i32, i32),
    /// Whether the fillets share a face.
    pub visavis: bool,
}

/// The surface data `i` of a stripe.
pub fn surf_data_at(cd: &StripeHandle, i: i32) -> Result<SurfDataHandle, Failure> {
    let seq = lock(cd)
        .set_of_surf_data()
        .cloned()
        .ok_or_else(|| null("no surface data sequence"))?;
    let g = lock(&seq);
    usize::try_from(i - 1)
        .ok()
        .and_then(|k| g.get(k))
        .cloned()
        .ok_or_else(|| Failure::out_of_range("NCollection_Sequence::Value: index out of range"))
}

/// The face of one pair of interferences of `IsInFront`, with its side; Standard_NullObject
/// for a null face or shape.
fn in_front_face(
    dstr: &TopOpeBRepDSDataStructure,
    (cd1, cd2): (&StripeHandle, &StripeHandle),
    (fd1, fd2): (&SurfDataHandle, &SurfDataHandle),
    (jf1, jf2): (i32, i32),
) -> Result<(Face, bool), Failure> {
    let face = topods::face(dstr.shape(lock(fd1).index(jf1), true))?;
    if face.is_null() {
        return Err(null(
            "ChFi3d_IsInFront : Trying to check orientation of NULL face",
        ));
    }
    let shape2 = dstr.shape(lock(fd2).index(jf2), true).clone();
    if shape2.is_null() {
        return Err(null(
            "ChFi3d_IsInFront : Trying to check orientation of NULL shape",
        ));
    }
    let (or_save1, or_save2) = (
        lock(cd1).orientation_on_face(jf1),
        lock(cd2).orientation_on_face(jf2),
    );
    let or = face.orientation();
    Ok((
        face.clone(),
        same_side(or, or_save1, or_save2, or, shape2.orientation()),
    ))
}

/// `ChFi3d_IsInFront(DStr, cd1, cd2, i1, i2, sens1, sens2, p1, p2, face, sameside, jf1,
/// jf2, visavis, Vtx, Check2dDistance, enlarge)`: whether the fillets `i1` and `i2` of the
/// stripes meet on a common face, trying the face pairs in turn and keeping the meeting
/// nearest to the ends; the found (or last tried) values in `res`;
/// `is_in_front(&ds, (&cd1, &cd2), (1, 1), (1, -1), &mut res, &v, (false, false))?`.
#[allow(clippy::too_many_arguments)]
pub fn is_in_front(
    dstr: &TopOpeBRepDSDataStructure,
    (cd1, cd2): (&StripeHandle, &StripeHandle),
    (i1, i2): (i32, i32),
    (sens1, sens2): (i32, i32),
    res: &mut InFront,
    vtx: &Vertex,
    (check_2d, enlarge): (bool, bool),
) -> Result<bool, Failure> {
    let (isf1, isf2) = (sens1 == 1, sens2 == 1);
    let (fd1, fd2) = (surf_data_at(cd1, i1)?, surf_data_at(cd2, i2)?);
    res.visavis = false;
    let mut saved: Option<InFront> = None;
    let (s11, s12) = {
        let g = lock(&fd1);
        (g.index_of_s1(), g.index_of_s2())
    };
    let (s21, s22) = {
        let g = lock(&fd2);
        (g.index_of_s1(), g.index_of_s2())
    };
    for (jf1, jf2, a, b) in [
        (1, 1, s11, s21),
        (2, 1, s12, s21),
        (1, 2, s11, s22),
        (2, 2, s12, s22),
    ] {
        if a != b {
            continue;
        }
        res.jf = (jf1, jf2);
        let (face, sameside) = in_front_face(dstr, (cd1, cd2), (&fd1, &fd2), (jf1, jf2))?;
        res.face = face;
        res.visavis = true;
        res.sameside = sameside;
        let pref1 = lock(&fd1).interference(jf1).parameter(isf1);
        let pref2 = lock(&fd2).interference(jf2).parameter(isf2);
        let p2d = if check_2d {
            BRepTool::parameters(vtx, &res.face)?
        } else {
            Point2::default()
        };
        match int_traces(
            (&fd1, jf1, pref1, sens1),
            (&fd2, jf2, pref2, sens2),
            (p2d, check_2d),
            enlarge,
        )? {
            Some((p1, p2)) => {
                res.p = (p1, p2);
                let restore = saved.as_ref().is_some_and(|s| {
                    (s.jf.0 == jf1 && f64::from(sens1) * (p1 - s.p.0) > 0.)
                        || (s.jf.1 == jf2 && f64::from(sens2) * (p2 - s.p.1) > 0.)
                });
                if restore {
                    let s = saved.clone().expect("checked");
                    (res.p, res.sameside, res.jf, res.face) = (s.p, s.sameside, s.jf, s.face);
                } else {
                    saved = Some(res.clone());
                }
            }
            None => {
                if let Some(s) = saved.clone() {
                    (res.p, res.sameside, res.jf, res.face) = (s.p, s.sameside, s.jf, s.face);
                }
            }
        }
    }
    Ok(saved.is_some())
}

/// `ChFi3d_FilCurveInDS(Icurv, Isurf, Pcurv, Et)`: the surface/curve interference of the
/// curve on the surface with its pcurve; `fil_curve_in_ds(1, 1, &pc, Orientation::Forward)`.
pub fn fil_curve_in_ds(
    icurv: i32,
    isurf: i32,
    pcurv: &Handle<dyn Geom2dCurve>,
    et: Orientation,
) -> Handle<TopOpeBRepDSInterference> {
    Handle::new(TopOpeBRepDSSurfaceCurveInterference::new(
        &TopOpeBRepDSTransition::from_orientation(et),
        TopOpeBRepDSKind::Surface,
        isurf,
        TopOpeBRepDSKind::Curve,
        icurv,
        pcurv,
    ))
    .cast()
}

/// `ChFi3d_SolidIndex(sp, DStr, MapESo, MapESh)`: the index in the structure of the solid
/// (else the shell) of the first edge of the spine; Standard_Failure for an empty spine;
/// `solid_index(Some(&sp), &mut ds, &eso, &esh)?`.
pub fn solid_index(
    sp: Option<&SpineHandle>,
    dstr: &mut TopOpeBRepDSDataStructure,
    map_eso: &Map,
    map_esh: &Map,
) -> Result<i32, Failure> {
    let edref: Shape = match sp {
        Some(s) if lock(s).nb_edges() != 0 => lock(s).edges(1)?.clone().into(),
        _ => return Err(Failure::new("SolidIndex : Spine incomplete")),
    };
    let so = map_eso.find_from_key(&edref)?;
    let shellousolid = match so.first() {
        Some(s) => s.clone(),
        None => map_esh
            .find_from_key(&edref)?
            .first()
            .cloned()
            .ok_or_else(|| {
                Failure::with_message("Standard_NoSuchObject", "SolidIndex: no shell of the edge")
            })?,
    };
    Ok(dstr.add_shape(&shellousolid))
}

/// `ChFi3d_IndexPointInDS(P1, DStr)`: the vertex of the point (its point and tolerance
/// updated) added to the structure, else a new point; `index_point_in_ds(&cp, &mut ds)?`.
pub fn index_point_in_ds(
    p1: &CommonPoint,
    dstr: &mut TopOpeBRepDSDataStructure,
) -> Result<i32, Failure> {
    if p1.is_vertex() {
        let v = p1.vertex()?;
        BRepBuilder::default().update_vertex_point(v, p1.point(), p1.tolerance())?;
        return Ok(dstr.add_shape(v.as_ref()));
    }
    Ok(dstr.add_point(&TopOpeBRepDSPoint::new(p1.point(), p1.tolerance())))
}

/// `ChFi3d_FilPointInDS(Et, Ic, Ip, Par, IsVertex)`: the curve/point (vertex) interference;
/// `fil_point_in_ds(Orientation::Forward, 1, 1, 0.0, false)`.
pub fn fil_point_in_ds(
    et: Orientation,
    ic: i32,
    ip: i32,
    par: f64,
    is_vertex: bool,
) -> Handle<TopOpeBRepDSInterference> {
    let gk = if is_vertex {
        TopOpeBRepDSKind::Vertex
    } else {
        TopOpeBRepDSKind::Point
    };
    Handle::new(TopOpeBRepDSCurvePointInterference::new(
        &TopOpeBRepDSTransition::from_orientation(et),
        TopOpeBRepDSKind::Curve,
        ic,
        gk,
        ip,
        par,
    ))
    .cast()
}

/// `ChFi3d_FilVertexInDS(Et, Ic, Ip, Par)`: the curve/vertex interference;
/// `fil_vertex_in_ds(Orientation::Forward, 1, 1, 0.0)`.
pub fn fil_vertex_in_ds(
    et: Orientation,
    ic: i32,
    ip: i32,
    par: f64,
) -> Handle<TopOpeBRepDSInterference> {
    fil_point_in_ds(et, ic, ip, par, true)
}
