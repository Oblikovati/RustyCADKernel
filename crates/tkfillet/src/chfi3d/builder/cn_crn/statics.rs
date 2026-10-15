// Rust translation of the file statics Calcul_Param, Calcul_P2dOnSurf, Calcul_C2dOnFace,
// Calcul_Orientation, RemoveSD, OrientationIcNonVive, OrientationIcplusNonVive,
// OrientationAreteViveConsecutive, ParametrePlate, SummarizeNormal, SurfIndex and
// PlateOrientation (ChFi3d_Builder_CnCrn.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The small tools of the corner of n edges: the ends of the contours, the orientations
//! of the connecting curves and of the plate.
use super::Corner;
use crate::chfi_ds::{StripeHandle, lock};
use crate::chfi3d::surf_data_at;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_plate::GeomPlateBuildPlateSurface;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

/// The `i`-th pcurve of the plate (`PSurf.Curves2d()->Value(i)`).
pub(super) fn plate_pcurve_at(
    curves: &Handle<HArray1<Handle<dyn Geom2dCurve>>>,
    i: i32,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let arr = curves.get().ok_or_else(|| null("Curves2d"))?;
    let a = arr.array1();
    a.value(i)
        .cloned()
        .map_err(|_| Failure::out_of_range("GeomPlate_BuildPlateSurface::Curves2d"))
}

/// The upper bound of the pcurves of the plate.
fn plate_pcurves_upper(curves: &Handle<HArray1<Handle<dyn Geom2dCurve>>>) -> Result<i32, Failure> {
    Ok(curves
        .get()
        .ok_or_else(|| null("Curves2d"))?
        .array1()
        .upper())
}

/// The null pcurve failure of a contour.
fn null(what: &str) -> Failure {
    Failure::with_message("Standard_NullObject", what)
}

/// `Calcul_Param(stripe, jfposit, indice, isfirst, param)`: the parameter of the end of
/// the interference on the side `jfposit` (on S1 unless 2).
pub(super) fn calcul_param(
    stripe: &StripeHandle,
    jfposit: i32,
    indice: i32,
    isfirst: bool,
) -> Result<f64, Failure> {
    let fd = surf_data_at(stripe, indice)?;
    let g = lock(&fd);
    let on_s = if jfposit == 2 { 2 } else { 1 };
    Ok(g.interference(on_s).parameter(isfirst))
}

/// `Calcul_P2dOnSurf(stripe, jfposit, indice, param, p2)`: the point of the pcurve on the
/// fillet of the side `jfposit` (on S2 unless 1) at `param`.
pub(super) fn calcul_p2d_on_surf(
    stripe: &StripeHandle,
    jfposit: i32,
    indice: i32,
    param: f64,
) -> Result<Point2, Failure> {
    let fd = surf_data_at(stripe, indice)?;
    let on_s = if jfposit == 1 { 1 } else { 2 };
    let pc = lock(&fd).interference(on_s).pcurve_on_surf().clone();
    pc.get()
        .ok_or_else(|| null("Calcul_P2dOnSurf"))?
        .value(param)
}

/// `Calcul_C2dOnFace(stripe, jfposit, indice, c2d)`: the pcurve on the face of the side
/// `jfposit` (on S2 unless 1).
pub(super) fn calcul_c2d_on_face(
    stripe: &StripeHandle,
    jfposit: i32,
    indice: i32,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let fd = surf_data_at(stripe, indice)?;
    let on_s = if jfposit == 1 { 1 } else { 2 };
    Ok(lock(&fd).interference(on_s).pcurve_on_face().clone())
}

/// `Calcul_Orientation(stripe, jfposit, indice, orient)`: the transition of the side
/// `jfposit` (on S2 unless 1).
fn calcul_orientation(
    stripe: &StripeHandle,
    jfposit: i32,
    indice: i32,
) -> Result<Orientation, Failure> {
    let fd = surf_data_at(stripe, indice)?;
    let on_s = if jfposit == 1 { 1 } else { 2 };
    Ok(lock(&fd).interference(on_s).transition())
}

/// `RemoveSD(Stripe, num1, num2)`: the surface data `num1..=num2` removed
/// (Standard_OutOfRange for a range outside the sequence or reversed, as natively).
pub(super) fn remove_sd(stripe: &StripeHandle, num1: i32, num2: i32) -> Result<(), Failure> {
    let Some(seq) = lock(stripe).set_of_surf_data().cloned() else {
        return Ok(());
    };
    let mut seq = lock(&seq);
    let n = seq.len() as i32;
    if n == 0 {
        return Ok(());
    }
    if num1 <= 0 || num2 > n || num1 > num2 {
        return Err(Failure::out_of_range("RemoveSD: bad range of surface data"));
    }
    seq.drain((num1 - 1) as usize..num2 as usize);
    Ok(())
}

/// The orientation `FORWARD` when `same` holds, else `REVERSED`.
fn forward_if(same: bool) -> Orientation {
    if same {
        Orientation::Forward
    } else {
        Orientation::Reversed
    }
}

/// `OrientationIcNonVive(CDic, jfic, icicplus, sensic, orien)`: the orientation of the
/// curve from the contour `ic` (not sharp) to `icplus`.
pub(super) fn orientation_ic_non_vive(
    cdic: &StripeHandle,
    jfic: i32,
    icicplus: i32,
    sensic: i32,
) -> Result<Orientation, Failure> {
    let orinterf = calcul_orientation(cdic, jfic, icicplus)?;
    Ok(forward_if(
        (orinterf == Orientation::Forward) == (sensic != 1),
    ))
}

/// `OrientationIcplusNonVive(CDicplus, jficplus, icplusic, sensicplus, orien)`: the
/// orientation of the curve from `ic` to the contour `icplus` (not sharp).
pub(super) fn orientation_icplus_non_vive(
    cdicplus: &StripeHandle,
    jficplus: i32,
    icplusic: i32,
    sensicplus: i32,
) -> Result<Orientation, Failure> {
    let orinterf = calcul_orientation(cdicplus, 3 - jficplus, icplusic)?;
    Ok(forward_if(
        (orinterf == Orientation::Forward) == (sensicplus == 1),
    ))
}

/// `OrientationAreteViveConsecutive(Fviveicicplus, Eviveic, V1, orien)`: the orientation
/// of the curve between two consecutive sharp edges, that of the edge `ic` in the face
/// when it ends at the vertex.
pub(super) fn orientation_arete_vive_consecutive(
    f: &Face,
    e: &Edge,
    v1: &Vertex,
) -> Result<Orientation, Failure> {
    let mut orinterf = Orientation::Forward;
    let mut ex = Explorer::new(
        &f.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while ex.more() {
        if e.is_same(ex.current()) {
            orinterf = ex.current().orientation();
            break;
        }
        ex.next()?;
    }
    let ends_at_v1 = topexp::last_vertex(e, false).is_same(v1);
    Ok(forward_if((orinterf == Orientation::Forward) == ends_at_v1))
}

/// `ParametrePlate(n3d, PSurf, Surf, point, apperror, uv)`: the parameters on the plate
/// of the end of a boundary at `point` (the last end tried when none is).
pub(super) fn parametre_plate(
    n3d: i32,
    psurf: &GeomPlateBuildPlateSurface,
    surf: &Handle<dyn GeomSurface>,
    point: Point3,
    apperror: f64,
) -> Result<Point2, Failure> {
    let curves = psurf.curves2d()?;
    let s = surf.get().ok_or_else(|| null("ParametrePlate"))?;
    let mut uv = Point2::new(0.0, 0.0);
    for ip in 1..=n3d {
        let pc = plate_pcurve_at(&curves, ip)?;
        let pc = pc.get().ok_or_else(|| null("ParametrePlate"))?;
        for par in [pc.first_parameter(), pc.last_parameter()] {
            uv = pc.value(par)?;
            if s.value(uv.x(), uv.y())?.is_equal(&point, apperror) {
                return Ok(uv);
            }
        }
    }
    Ok(uv)
}

/// `SummarizeNormal(V1, Fcur, Ecur, SumFaceNormalAtV1)`: the normal of the face at the
/// end of the edge at the vertex added to the normalized sum.
pub(super) fn summarize_normal(
    v1: &Vertex,
    fcur: &Face,
    ecur: &Edge,
    sum: &mut Vector3,
) -> Result<(), Failure> {
    let (mut uv1, uv2) = BRepTool::uv_points_on_face(ecur, fcur)?;
    if !v1.is_same(&topexp::first_vertex(ecur, false)) {
        uv1 = uv2;
    }
    let s = BRepTool::surface(fcur)?;
    let d1 = s
        .get()
        .ok_or_else(|| null("SummarizeNormal"))?
        .d1(uv1.x(), uv1.y())?;
    let mut n = d1.d1u.crossed(d1.d1v);
    if fcur.orientation() == Orientation::Reversed {
        n.reverse();
    }
    if n.square_magnitude() <= precision::p_confusion() {
        return Ok(());
    }
    *sum += n.normalized()?;
    sum.normalize()
}

/// The surfaces of a surface data asked by `SurfIndex`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SurfType {
    Surface,
    Face1,
    Face2,
}

/// `SurfIndex(StripeArray1, StripeIndex, SurfDataIndex, SurfType)`: the index of the
/// fillet surface or of a face of the `sd`-th surface data of the contour `ic`.
pub(super) fn surf_index(c: &Corner, ic: usize, sd: i32, kind: SurfType) -> Result<i32, Failure> {
    let fd = surf_data_at(c.cd(ic)?, sd)?;
    let g = lock(&fd);
    Ok(match kind {
        SurfType::Surface => g.surf(),
        SurfType::Face1 => g.index_of_s1(),
        SurfType::Face2 => g.index_of_s2(),
    })
}

/// The point and the normal of the plate at the start of a pcurve.
fn plate_normal(surf: &dyn GeomSurface, uv: Point2) -> Result<(Point3, Vector3), Failure> {
    let d1 = surf.d1(uv.x(), uv.y())?;
    let mut n1 = d1.d1u.crossed(d1.d1v);
    n1.normalize()?;
    Ok((d1.point, n1))
}

/// `PlateOrientation(thePlateSurf, thePCArr, theRefDir)`: FORWARD when the normal of the
/// plate and the turn of its boundary agree with the reference direction.
pub(super) fn plate_orientation(
    surf: &Handle<dyn GeomSurface>,
    pcarr: &Handle<HArray1<Handle<dyn Geom2dCurve>>>,
    refdir: &Vector3,
) -> Result<Orientation, Failure> {
    let s = surf.get().ok_or_else(|| null("PlateOrientation"))?;
    let nb = plate_pcurves_upper(pcarr)?;
    let at = |i: i32, mid: bool| -> Result<Point2, Failure> {
        let pc = plate_pcurve_at(pcarr, i)?;
        let pc = pc.get().ok_or_else(|| null("PlateOrientation"))?;
        let (f, l) = (pc.first_parameter(), pc.last_parameter());
        pc.value(if mid { (f + l) / 2. } else { f })
    };
    let uv = at(nb, true)?;
    let mut pp1 = s.value(uv.x(), uv.y())?;
    let (mut sum1, mut sum2) = (0.0, 0.0);
    for i in 1..=nb {
        let (pp2, n1) = plate_normal(s, at(i, false)?)?;
        let uv = at(i, true)?;
        let pp3 = s.value(uv.x(), uv.y())?;
        let mut n2 = Vector3::between(pp2, pp3).crossed(Vector3::between(pp2, pp1));
        n2.normalize()?;
        sum1 += n1.dot(n2);
        sum2 += n2.dot(*refdir);
        pp1 = pp3;
    }
    Ok(forward_if(sum2 * sum1 > 0.0))
}
