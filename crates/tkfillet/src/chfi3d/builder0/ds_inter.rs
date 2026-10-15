//! The intersections of the stripes at their ends, the bounding of the surfaces and the search of the fillets meeting at a vertex.
use super::bounds::boite4;
use super::ds_tools::{InFront, is_in_front, null};
use super::stripes::{has_common_endpoint, has_transversal_intersection};
use crate::chfi_ds::{StripeHandle, SurfDataHandle, lock};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

/// The static `ChFi3d_HasCompleteCoincidence`: one segment covering both curves; with the
/// curves running in opposite senses.
fn has_complete_coincidence(
    inter: &GInter,
    (c1, c2): (&Geom2dAdaptorCurve, &Geom2dAdaptorCurve),
    tol: f64,
) -> Result<Option<bool>, Failure> {
    if inter.nb_segments()? != 1 {
        return Ok(None);
    }
    let seg = inter.segment(1)?;
    if !seg.has_first_point() || !seg.has_last_point() {
        return Ok(None);
    }
    let (f, l) = (seg.first_point()?, seg.last_point()?);
    let (ff, lf, fs, ls) = (
        f.param_on_first(),
        l.param_on_first(),
        f.param_on_second(),
        l.param_on_second(),
    );
    let complete = (std_min(ff, lf) - c1.first_parameter()?).abs() <= tol
        && (std_max(ff, lf) - c1.last_parameter()?).abs() <= tol
        && (std_min(fs, ls) - c2.first_parameter()?).abs() <= tol
        && (std_max(fs, ls) - c2.last_parameter()?).abs() <= tol;
    Ok(complete.then_some((lf - ff) * (ls - fs) < 0.))
}

/// The face interferences of two surface data on a common face (None when none).
fn common_face_interferences(
    d1: &SurfDataHandle,
    d2: &SurfDataHandle,
) -> Option<(
    crate::chfi_ds::FaceInterference,
    crate::chfi_ds::FaceInterference,
)> {
    let (a, b) = (lock(d1), lock(d2));
    let (i11, i12, i21, i22) = (
        a.index_of_s1(),
        a.index_of_s2(),
        b.index_of_s1(),
        b.index_of_s2(),
    );
    let pick = if i11 == i21 {
        (1, 1)
    } else if i11 == i22 {
        (1, 2)
    } else if i12 == i21 {
        (2, 1)
    } else if i12 == i22 {
        (2, 2)
    } else {
        return None;
    };
    Some((
        a.interference(pick.0).clone(),
        b.interference(pick.1).clone(),
    ))
}

/// Whether two stripes share an end point.
fn stripes_share_point(s1: &StripeHandle, s2: &StripeHandle) -> bool {
    let (a, b) = (lock(s1), lock(s2));
    for i1 in 1..=2 {
        for i2 in 1..=2 {
            let (af, al, bf, bl) = (
                a.index_point(true, i1),
                a.index_point(false, i1),
                b.index_point(true, i2),
                b.index_point(false, i2),
            );
            if al == bl || al == bf || af == bl || af == bf {
                return true;
            }
        }
    }
    false
}

/// The traces of two fillets on a common face merged in the structure when they
/// coincide or share an end; StdFail_NotDone when they cross.
fn merge_traces(
    dstr: &mut TopOpeBRepDSDataStructure,
    (fi1, fi2): (
        &crate::chfi_ds::FaceInterference,
        &crate::chfi_ds::FaceInterference,
    ),
    tol2d: f64,
    inter: &mut GInter,
) -> Result<(), Failure> {
    let c1 = Geom2dAdaptorCurve::with_range(
        fi1.pcurve_on_face(),
        fi1.first_parameter(),
        fi1.last_parameter(),
    )?;
    let c2 = Geom2dAdaptorCurve::with_range(
        fi2.pcurve_on_face(),
        fi2.first_parameter(),
        fi2.last_parameter(),
    )?;
    inter.perform_between(&c1, &c2, tol2d, precision::p_confusion())?;
    if let Some(rev) = has_complete_coincidence(inter, (&c1, &c2), tol2d)? {
        dstr.merge_equivalent_curves(fi1.line_index(), fi2.line_index(), rev);
    }
    if has_transversal_intersection(inter)? {
        return Err(Failure::with_message(
            "StdFail_NotDone",
            "StripeEdgeInter : fillets have too big radiuses",
        ));
    }
    let (mut s1, mut s2) = (false, false);
    if has_common_endpoint(inter, (&c1, &c2), tol2d, (&mut s1, &mut s2))? {
        dstr.merge_equivalent_curve_points(fi1.line_index(), s1, fi2.line_index(), s2);
    }
    Ok(())
}

/// `ChFi3d_StripeEdgeInter(theStripe1, theStripe2, DStr, tol2d)`: the traces of the two
/// stripes (sharing no end point) on their common faces merged when they coincide or
/// share an end; StdFail_NotDone when they cross;
/// `stripe_edge_inter(&st1, &st2, &mut ds, 1e-5)?`.
pub fn stripe_edge_inter(
    s1: &StripeHandle,
    s2: &StripeHandle,
    dstr: &mut TopOpeBRepDSDataStructure,
    tol2d: f64,
) -> Result<(), Failure> {
    if stripes_share_point(s1, s2) {
        return Ok(());
    }
    let seq1 = lock(s1)
        .set_of_surf_data()
        .cloned()
        .ok_or_else(|| null("StripeEdgeInter: no surface data"))?;
    let seq2 = lock(s2)
        .set_of_surf_data()
        .cloned()
        .ok_or_else(|| null("StripeEdgeInter: no surface data"))?;
    let (l1, l2) = (lock(&seq1).clone(), lock(&seq2).clone());
    let mut inter = GInter::new();
    for d1 in &l1 {
        for d2 in &l2 {
            let Some((fi1, fi2)) = common_face_interferences(d1, d2) else {
                continue;
            };
            if fi1.first_parameter() == fi1.last_parameter()
                || fi2.first_parameter() == fi2.last_parameter()
                || fi1.pcurve_on_face().is_null()
                || fi2.pcurve_on_face().is_null()
            {
                continue;
            }
            merge_traces(dstr, (&fi1, &fi2), tol2d, &mut inter)?;
        }
    }
    Ok(())
}

/// `ChFi3d_BoundSurf(DStr, Fd1, IFaCo1, IFaArc1)`: the fillet surface bounded around the
/// box of its traces (extended along the cylinder axis, around the torus or cone, along
/// the plane); `bound_surf(&ds, &fd, 1, 2)?`.
pub fn bound_surf(
    dstr: &TopOpeBRepDSDataStructure,
    fd1: &SurfDataHandle,
    ifaco1: i32,
    ifaarc1: i32,
) -> Result<Handle<GeomAdaptorSurface>, Failure> {
    let g = lock(fd1);
    let s = dstr.surface(g.surf()).surface().clone();
    let hs = GeomAdaptorSurface::default();
    hs.load(&s)?;
    if ifaco1 == 0 || ifaarc1 == 0 {
        return Ok(Handle::new(hs));
    }
    let (fico, fiarc) = (g.interference(ifaco1), g.interference(ifaarc1));
    let uv = |fi: &crate::chfi_ds::FaceInterference, first: bool| -> Result<Point2, Failure> {
        fi.pcurve_on_surf()
            .get()
            .ok_or_else(|| null("ChFi3d_BoundSurf: null pcurve"))?
            .value(if first {
                fi.first_parameter()
            } else {
                fi.last_parameter()
            })
    };
    let (uvf1, uvl1, uvf2, uvl2) = (
        uv(fico, true)?,
        uv(fico, false)?,
        uv(fiarc, true)?,
        uv(fiarc, false)?,
    );
    let (du, dv, mu, big_mu, mv, big_mv) = boite4([uvf1, uvf2, uvl1, uvl2]);
    match hs.get_type()? {
        SurfaceType::Cylinder => {
            let dv = std_max(0.5 * dv, 4. * hs.cylinder()?.radius());
            hs.load_bounds(&s, (mu, big_mu, mv - dv, big_mv + dv))?;
        }
        SurfaceType::Torus | SurfaceType::Cone => {
            let du = std_min(PI - 0.5 * du, 0.1 * du);
            hs.load_bounds(&s, (mu - du, big_mu + du, mv, big_mv))?;
        }
        SurfaceType::Plane => {
            let du = std_max(0.5 * du, 4. * dv);
            hs.load_bounds(&s, (mu - du, big_mu + du, mv, big_mv))?;
        }
        _ => {}
    }
    Ok(Handle::new(hs))
}

/// The result of `ChFi3d_SearchFD`.
#[derive(Clone, Debug, Default)]
pub struct SearchFd {
    /// The fillets found.
    pub i: (i32, i32),
    /// The meeting on their common face.
    pub front: InFront,
}

/// `ChFi3d_SearchFD(DStr, cd1, cd2, sens1, sens2, i1, i2, p1, p2, ind1, ind2, face,
/// sameside, jf1, jf2)`: the fillets of the two stripes meeting on a common face,
/// searched from `ind1` and `ind2` along the senses alternately (the first pair sharing a
/// face when none meets); `search_fd(&ds, (&cd1, &cd2), (1, -1), (1, 1), &mut res)?`.
pub fn search_fd(
    dstr: &TopOpeBRepDSDataStructure,
    (cd1, cd2): (&StripeHandle, &StripeHandle),
    (sens1, sens2): (i32, i32),
    (ind1, ind2): (i32, i32),
    res: &mut SearchFd,
) -> Result<bool, Failure> {
    let seq_len = |cd: &StripeHandle| -> Result<i32, Failure> {
        let seq = lock(cd)
            .set_of_surf_data()
            .cloned()
            .ok_or_else(|| null("ChFi3d_SearchFD: no surface data"))?;
        Ok(i32::try_from(lock(&seq).len()).unwrap_or(i32::MAX))
    };
    let (l1, l2) = (seq_len(cd1)?, seq_len(cd2)?);
    let (mut if1, mut if2, id1, id2) = (ind1, ind2, ind1, ind2);
    let (mut found, mut fini1, mut fini2, mut visavisok) = (false, false, false, false);
    let vtx = Vertex::default();
    while !found {
        let mut i = id1;
        while i * sens1 <= if1 * sens1 && !found && !fini2 {
            found = try_pair(
                dstr,
                (cd1, cd2),
                (i, if2),
                (sens1, sens2),
                res,
                &vtx,
                &mut visavisok,
            )?;
            i += sens1;
        }
        if !fini1 {
            if1 += sens1;
            if if1 < 1 || if1 > l1 {
                if1 -= sens1;
                fini1 = true;
            }
        }
        let mut i = id2;
        while i * sens2 <= if2 * sens2 && !found && !fini1 {
            found = try_pair(
                dstr,
                (cd1, cd2),
                (if1, i),
                (sens1, sens2),
                res,
                &vtx,
                &mut visavisok,
            )?;
            i += sens2;
        }
        if !fini2 {
            if2 += sens2;
            if if2 < 1 || if2 > l2 {
                if2 -= sens2;
                fini2 = true;
            }
        }
        if fini1 && fini2 {
            break;
        }
    }
    Ok(found)
}

/// One pair of `SearchFD`: true when the fillets meet; the first pair sharing a face
/// noted otherwise.
#[allow(clippy::too_many_arguments)]
fn try_pair(
    dstr: &TopOpeBRepDSDataStructure,
    cds: (&StripeHandle, &StripeHandle),
    (i1, i2): (i32, i32),
    senses: (i32, i32),
    res: &mut SearchFd,
    vtx: &Vertex,
    visavisok: &mut bool,
) -> Result<bool, Failure> {
    if is_in_front(
        dstr,
        cds,
        (i1, i2),
        senses,
        &mut res.front,
        vtx,
        (false, false),
    )? {
        res.i = (i1, i2);
        return Ok(true);
    }
    if res.front.visavis && !*visavisok {
        *visavisok = true;
        res.i = (i1, i2);
    }
    Ok(false)
}
