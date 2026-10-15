// Rust translation of the file statics recadre, Update (three overloads),
// IntersUpdateOnSame, ChFi3d_ExtendSurface, ComputeCurve2d and ChFi3d_Recale, and of
// ChFi3d_SelectStripe (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ends of a fillet moved onto the face that closes it at a vertex.
use crate::chfi_ds::{CommonPoint, FaceInterference, State, StripeHandle, lock};
use crate::chfi3d::{index_of_surf_data, is_tangent_faces};
use rustycad_tkbool::brep_algo::BRepAlgoNormalProjection;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomBoundedSurface, GeomCurve,
    GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::int_curve_surface::HInter;
use rustycad_tkgeombase::extrema::LocateExtCC;
use rustycad_tkgeombase::geom_lib::extend_surf_by_length;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// `recadre(p, ref, isfirst, first, last)`: `p` moved by a period towards `ref` when it
/// gets nearer.
fn recadre(p: f64, rf: f64, isfirst: bool, first: f64, last: f64) -> f64 {
    let pp = if isfirst {
        p - (last - first)
    } else {
        p + (last - first)
    };
    if (pp - rf).abs() < (p - rf).abs() {
        pp
    } else {
        p
    }
}

/// The end of an interference and its common point moved to the parameter `w` on the
/// fillet, at the point `p`.
fn set_end(
    fi: &mut FaceInterference,
    cp: &mut CommonPoint,
    w: f64,
    isfirst: bool,
    p: rustycad_tkmath::gp::Point3,
) {
    fi.set_parameter(w, isfirst);
    cp.reset();
    cp.set_point(p);
}

/// `Update(fb, pcfb, surf, fi, cp, p2dbout, isfirst, pared, wop, tol)`: the end of the
/// fillet moved to the nearest point of the arc `pcfb` of the face (when nearer than
/// `tol`); `(p2dbout, pared, wop)` updated.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_on_arc(
    fb: &Handle<dyn Adaptor3dSurface>,
    pcfb: &Handle<dyn Adaptor2dCurve2d>,
    surf: &Handle<dyn Adaptor3dSurface>,
    (fi, cp): (&mut FaceInterference, &mut CommonPoint),
    p2dbout: &mut Point2,
    isfirst: bool,
    (pared, wop): (&mut f64, &mut f64),
    tol: f64,
) -> Result<bool, Failure> {
    let c1 = Adaptor3dCurveOnSurface::with_curve(pcfb, fb)?;
    let hpc: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(fi.pcurve_on_surf())?).cast();
    let c2 = Adaptor3dCurveOnSurface::with_curve(&hpc, surf)?;
    let ext = LocateExtCC::new(&c1, &c2, *pared, *wop)?;
    if ext.is_done() && ext.square_distance()? < tol * tol {
        let (ponc1, ponc2) = ext.point()?;
        *p2dbout = pcfb.value(ponc1.parameter())?;
        *pared = ponc1.parameter();
        *wop = ponc2.parameter();
        set_end(fi, cp, *wop, isfirst, *ponc1.value());
        return Ok(true);
    }
    Ok(false)
}

/// The solution of `Update` on the intersection of the curve with the face nearest to
/// `wop` (recentred on a periodic curve).
fn nearest_solution(
    inter: &HInter,
    (uf, ul): (f64, f64),
    wop: f64,
    isfirst: bool,
    isperiodic: bool,
) -> Result<Option<(usize, bool, f64, f64)>, Failure> {
    let nbp = inter.nb_points()?;
    let (mut isol, mut dist) = (0, precision::infinite());
    for i in 1..=nbp {
        let mut w = inter.point(i)?.w();
        if isperiodic {
            w = recadre(w, wop, isfirst, uf, ul);
        }
        if uf <= w && ul >= w && (w - wop).abs() < dist {
            isol = i;
            dist = (w - wop).abs();
        }
    }
    let (mut isolbis, mut distbis, mut wbis) = (0, precision::infinite(), 0.);
    let mut recadrebis = false;
    if isperiodic {
        for i in 1..=nbp {
            let w = inter.point(i)?.w();
            if uf <= w
                && ul >= w
                && (w - wop).abs() < distbis
                && ((w - ul).abs() <= 0.01 || (w - uf).abs() <= 0.01)
            {
                isolbis = i;
                wbis = recadre(w, wop, isfirst, uf, ul);
                distbis = (wbis - wop).abs();
                recadrebis = true;
            }
        }
    }
    if isol == 0 && isolbis == 0 {
        return Ok(None);
    }
    if !recadrebis {
        let mut w = inter.point(isol)?.w();
        if isperiodic {
            w = elclib::in_period(w, uf, ul);
        }
        return Ok(Some((isol as usize, false, w, dist)));
    }
    if dist > distbis {
        return Ok(Some((isolbis as usize, true, wbis, distbis)));
    }
    let w = elclib::in_period(inter.point(isol)?.w(), uf, ul);
    Ok(Some((isol as usize, false, w, dist)))
}

/// `Update(fb, ct, fi, cp, p2dbout, isfirst, wop)`: the end of the fillet moved to the
/// intersection of its curve `ct` with the face nearest to `wop`.
pub(super) fn update_on_face(
    fb: &Handle<dyn Adaptor3dSurface>,
    ct: &Handle<dyn Adaptor3dCurve>,
    (fi, cp): (&mut FaceInterference, &mut CommonPoint),
    p2dbout: &mut Point2,
    isfirst: bool,
    wop: &mut f64,
) -> Result<bool, Failure> {
    let c = ct
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "Update"))?;
    let (uf, ul) = (c.first_parameter()?, c.last_parameter()?);
    let isperiodic = c.is_periodic()?;
    let mut inter = HInter::new();
    inter.perform(ct, fb)?;
    if !inter.is_done() {
        return Ok(false);
    }
    let Some((i, _, w, _)) = nearest_solution(&inter, (uf, ul), *wop, isfirst, isperiodic)? else {
        return Ok(false);
    };
    let pint = inter.point(i as i32)?;
    *p2dbout = Point2::new(pint.u(), pint.v());
    set_end(fi, cp, w, isfirst, c.value(w)?);
    *wop = w;
    Ok(true)
}

/// `IntersUpdateOnSame(HGs, HBs, c3dFI, Fop, Fprol, Eprol, Vtx, isFirst, Tol, FIop, CPop,
/// FprolUV, c3dU)`: the end of the fillet on the opposite face moved onto the face
/// prolonged: by the intersection of its curve, else along the prolonged edge when the
/// faces are tangent.
#[allow(clippy::too_many_arguments)]
pub(super) fn inters_update_on_same(
    hgs: &Handle<dyn Adaptor3dSurface>,
    hbs: &Handle<dyn Adaptor3dSurface>,
    c3d_fi: &Handle<dyn GeomCurve>,
    (fop, fprol, eprol, vtx): (&Face, &Face, &Edge, &Vertex),
    is_first: bool,
    tol: f64,
    (fiop, cpop): (&mut FaceInterference, &mut CommonPoint),
    fprol_uv: &mut Point2,
    c3d_u: &mut f64,
) -> Result<bool, Failure> {
    let (uf, ul) = (fiop.first_parameter(), fiop.last_parameter());
    let c = c3d_fi
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "IntersUpdateOnSame"))?;
    let hc3df: Handle<dyn Adaptor3dCurve> = if c.is_periodic() {
        Handle::new(GeomAdaptorCurve::with_curve(c3d_fi)?).cast()
    } else {
        Handle::new(GeomAdaptorCurve::with_range(c3d_fi, uf, ul)?).cast()
    };
    if update_on_face(
        hbs,
        &hc3df,
        (&mut *fiop, &mut *cpop),
        fprol_uv,
        is_first,
        c3d_u,
    )? {
        return Ok(true);
    }
    if !is_tangent_faces(eprol, fprol, fop, Continuity::G1)? {
        return Ok(false);
    }
    let gpcprol = BRepTool::curve_on_face(eprol, fprol)?.pcurve;
    if gpcprol.is_null() {
        return Err(Failure::with_message(
            "Standard_ConstructionError",
            "Failed to get p-curve of edge",
        ));
    }
    let pcprol: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(&gpcprol)?).cast();
    let mut partemp = BRepTool::parameter(vtx, eprol)?;
    update_on_arc(
        hbs,
        &pcprol,
        hgs,
        (fiop, cpop),
        fprol_uv,
        is_first,
        (&mut partemp, c3d_u),
        tol,
    )
}

/// `Update(face, edonface, surf, fi, cp, isfirst)`: the end of the fillet on the arc of its
/// common point moved to the nearest point of the arc (within its range extended by a
/// tenth).
pub(super) fn update_on_edge(
    face: &Handle<dyn Adaptor3dSurface>,
    edonface: &Handle<dyn Adaptor2dCurve2d>,
    surf: &Handle<dyn Adaptor3dSurface>,
    (fi, cp): (&mut FaceInterference, &mut CommonPoint),
    isfirst: bool,
) -> Result<bool, Failure> {
    if !cp.is_on_arc() {
        return Ok(false);
    }
    let c1 = Adaptor3dCurveOnSurface::with_curve(edonface, face)?;
    let pc = fi.pcurve_on_surf().clone();
    let pcr = pc
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "Update"))?;
    let (mut f, mut l) = (fi.first_parameter(), fi.last_parameter());
    let delta = 0.1 * (l - f);
    f = (f - delta).max(pcr.first_parameter());
    l = (l + delta).min(pcr.last_parameter());
    let hpc: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&pc, f, l)?).cast();
    let c2 = Adaptor3dCurveOnSurface::with_curve(&hpc, surf)?;
    let ext = LocateExtCC::new(&c1, &c2, cp.parameter_on_arc()?, fi.parameter(isfirst))?;
    if !ext.is_done() {
        return Ok(false);
    }
    let (ponc1, ponc2) = ext.point()?;
    let (pared, mut parltg) = (ponc1.parameter(), ponc2.parameter());
    if !(parltg > f && parltg < l) {
        return Ok(false);
    }
    let mut pconf = fi.pcurve_on_face().clone();
    if !pconf.is_null() {
        let tr: Handle<Geom2dTrimmedCurve> = pconf.clone().cast();
        if let Some(t) = tr.get() {
            pconf = t.basis_curve();
        }
        let p = pconf.get().ok_or_else(|| Failure::new("Update"))?;
        if !p.is_periodic() {
            if isfirst {
                parltg = parltg.max(p.first_parameter());
            } else {
                parltg = parltg.min(p.last_parameter());
            }
        }
    }
    fi.set_parameter(parltg, isfirst);
    let (tol, arc, trans) = (cp.tolerance(), cp.arc()?.clone(), cp.transition_on_arc()?);
    cp.set_arc(tol, &arc, pared, trans);
    Ok(true)
}

/// `ChFi3d_ExtendSurface(S, prol)`: a B-spline or Bezier surface extended on its four
/// sides by its diagonal (`prol` 1 or 2; 0 for the others).
pub(super) fn extend_surface(
    s: &mut Handle<dyn GeomSurface>,
    prol: &mut i32,
) -> Result<(), Failure> {
    if *prol != 0 {
        return Ok(());
    }
    *prol = if !s.clone().cast::<GeomBSplineSurface>().is_null() {
        1
    } else if !s.clone().cast::<GeomBezierSurface>().is_null() {
        2
    } else {
        0
    };
    if *prol == 0 {
        return Ok(());
    }
    let surf = s.get().ok_or_else(|| Failure::new("ExtendSurface"))?;
    let (umin, umax, vmin, vmax) = surf.bounds();
    let length = surf.value(umin, vmin)?.distance(&surf.value(umax, vmax)?);
    let mut a_bs: Handle<dyn GeomBoundedSurface> = s.clone().cast();
    extend_surf_by_length(&mut a_bs, length, 1, false, true)?;
    extend_surf_by_length(&mut a_bs, length, 1, true, true)?;
    extend_surf_by_length(&mut a_bs, length, 1, false, false)?;
    extend_surf_by_length(&mut a_bs, length, 1, true, false)?;
    *s = a_bs.cast();
    Ok(())
}

/// `ComputeCurve2d(Ct, Face, C2d)`: the pcurve on the face of the normal projection of
/// the curve (unchanged when the projection fails).
pub(super) fn compute_curve_2d(
    ct: &Handle<dyn GeomCurve>,
    face: &Face,
    c2d: &mut Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    let edg = BRepLibMakeEdge::from_curve(ct)?.edge()?;
    let mut ort_proj = BRepAlgoNormalProjection::new();
    ort_proj.init(face.as_ref());
    ort_proj.add(edg.as_ref())?;
    ort_proj.set_params(1.0e-6, 1.0e-6, Continuity::C1, 14, 16);
    ort_proj.set_limit(false);
    ort_proj.compute3d(false);
    ort_proj.build()?;
    if ort_proj.is_done() {
        let mut map_e1 = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(ort_proj.projection(), ShapeEnum::Edge, &mut map_e1)?;
        if map_e1.extent() != 0 {
            let e1 = topods::edge(map_e1.find_key(1)?)?;
            *c2d = BRepTool::curve_on_face(&e1, face)?.pcurve;
        }
    }
    Ok(())
}

/// The coordinate `k` (0 for U, 1 for V) of the points brought within half a period.
fn recale_coord(c1: &mut f64, c2: &mut f64, per: f64, refon1: bool) {
    if (*c2 - *c1).abs() > 0.5 * per {
        if *c2 < *c1 && refon1 {
            *c2 += per;
        } else if *c2 < *c1 && !refon1 {
            *c1 -= per;
        } else if *c1 < *c2 && refon1 {
            *c2 -= per;
        } else if *c1 < *c2 && !refon1 {
            *c1 += per;
        }
    }
}

/// `ChFi3d_Recale(Bs, p1, p2, refon1)`: the points on a periodic surface brought within
/// half a period of each other (the first one fixed with `refon1`).
pub(super) fn recale(
    bs: &BRepAdaptorSurface,
    p1: &mut Point2,
    p2: &mut Point2,
    refon1: bool,
) -> Result<(), Failure> {
    let mut surf = bs.transformed_surface().geom_surface_original();
    let ts: Handle<GeomRectangularTrimmedSurface> = surf.clone().cast();
    if let Some(t) = ts.get() {
        surf = t.basis_surface();
    }
    let s = surf.get().ok_or_else(|| Failure::new("ChFi3d_Recale"))?;
    if s.is_u_periodic() {
        let (mut u1, mut u2) = (p1.x(), p2.x());
        recale_coord(&mut u1, &mut u2, s.u_period()?, refon1);
        p1.set_x(u1);
        p2.set_x(u2);
    }
    if s.is_v_periodic() {
        let (mut v1, mut v2) = (p1.y(), p2.y());
        recale_coord(&mut v1, &mut v2, s.v_period()?, refon1);
        p1.set_y(v1);
        p2.set_y(v2);
    }
    Ok(())
}

/// `ChFi3d_SelectStripe(It, Vtx, thePrepareOnSame)`: the index of the first stripe of the
/// list (the first one ending OnSame at the vertex when preparing OnSame).
pub(super) fn select_stripe(
    list: &[StripeHandle],
    vtx: &Vertex,
    prepare_on_same: bool,
) -> Result<Option<usize>, Failure> {
    if !prepare_on_same {
        return Ok(if list.is_empty() { None } else { Some(0) });
    }
    for (k, stripe) in list.iter().enumerate() {
        let (_, sens) = index_of_surf_data(vtx, stripe)?;
        let sp = super::compute::spine_of(stripe)?;
        let stat = if sens == 1 {
            lock(&sp).first_status()
        } else {
            lock(&sp).last_status()
        };
        if stat == State::OnSame {
            return Ok(Some(k));
        }
    }
    Ok(None)
}
