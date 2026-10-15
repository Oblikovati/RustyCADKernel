// Rust translation of the boundaries of the plate on the contours and of the points of the
// sharp edges in ChFi3d_Builder::PerformMoreThreeCorner (ChFi3d_Builder_CnCrn.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The boundaries of the plate on the fillets: a straight pcurve across the end of each
//! contour, its 3d curve and its points stored, and the end of the contour updated; then
//! the points where the sharp edges are cut.
use super::batten::calcul_droite;
use super::regul::end_point;
use super::statics::{SurfType, calcul_p2d_on_surf, surf_index};
use super::{Corner, indices, slot};
use crate::chfi_ds::{CommonPoint, lock};
use crate::chfi3d::builder::ChFi3dBuilder;
use crate::chfi3d::surf_data_at;
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSPoint};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_plate::{GeomPlateBuildPlateSurface, GeomPlateCurveConstraint};
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;

/// The curve on surface of a pcurve, as a handle.
pub(super) fn curve_on(
    pcurve: &Handle<dyn Geom2dCurve>,
    asurf: &Handle<GeomAdaptorSurface>,
) -> Result<Handle<Adaptor3dCurveOnSurface>, Failure> {
    let acurv: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(pcurve)?).cast();
    let s: Handle<dyn Adaptor3dSurface> = asurf.clone().cast();
    Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(
        &acurv, &s,
    )?))
}

/// The plate constraint of order `order` on the curve on surface, with `nbpnt` points.
pub(super) fn add_constraint(
    psurf: &mut GeomPlateBuildPlateSurface,
    hcons: &Handle<Adaptor3dCurveOnSurface>,
    (order, nbpnt): (i32, i32),
    (tolapp3d, angular): (f64, f64),
) -> Result<(), Failure> {
    let cont =
        GeomPlateCurveConstraint::new(&hcons.clone().cast(), order, nbpnt, tolapp3d, angular, 0.1)?;
    psurf.add_curve(&Handle::new(cont));
    Ok(())
}

/// `GeomLib::BuildCurve3d(tolapp, CurvOnS, ...)` on the whole curve: the 3d curve and
/// the maximal deviation.
pub(super) fn build_curve3d(
    hcons: &Handle<Adaptor3dCurveOnSurface>,
    curv3d: &mut Handle<dyn GeomCurve>,
) -> Result<(f64, f64, f64), Failure> {
    let cos = hcons
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "BuildCurve3d"))?;
    let (f, l) = (cos.first_parameter()?, cos.last_parameter()?);
    let (maxdev, _) = geom_lib::build_curve3d(1.0e-4, cos, (f, l), curv3d, Continuity::C1, 14, 30)?;
    Ok((maxdev, f, l))
}

impl ChFi3dBuilder {
    /// The boundaries of the plate on the contours (G1 constraints) and their storage.
    pub(super) fn stripe_bounds(
        &mut self,
        c: &mut Corner,
        psurf: &mut GeomPlateBuildPlateSurface,
    ) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if c.sharp[ic] {
                continue;
            }
            c.n3d += 1;
            let (icplus, icmoins) = indices(c.nedge, ic);
            let p2d1 =
                calcul_p2d_on_surf(c.cd(ic)?, 3 - c.jf[ic], c.i[ic][icmoins], c.p[ic][icmoins])?;
            let p2d2 = calcul_p2d_on_surf(c.cd(ic)?, c.jf[ic], c.i[ic][icplus], c.p[ic][icplus])?;
            let indice = surf_index(c, ic, c.i[ic][icplus], SurfType::Surface)?;
            let asurf = Handle::new(GeomAdaptorSurface::with_surface(
                self.ds.surface(indice).surface(),
            )?);
            let pcurve = calcul_droite(p2d1, p2d2.x() - p2d1.x(), p2d2.y() - p2d1.y())?;
            let hcons = curve_on(&pcurve, &asurf)?;
            c.order[ic] = 1;
            add_constraint(
                psurf,
                &hcons,
                (c.order[ic], 10),
                (self.tolapp3d, self.angular),
            )?;
            let (maxapp, f, l) = build_curve3d(&hcons, &mut c.curv3d)?;
            c.maxapp = maxapp;
            *slot(&mut c.indcurve3d, c.n3d)? = self
                .ds
                .add_curve(&mut TopOpeBRepDSCurve::new(&c.curv3d, maxapp, false));
            let cos = hcons
                .get()
                .ok_or_else(|| Failure::new("PerformMoreThreeCorner"))?;
            let points = [cos.value(f)?, cos.value(l)?];
            c.errapp[ic] = maxapp;
            self.end_points_in_ds(c, ic, points)?;
            update_stripe(c, ic, (&pcurve, points))?;
        }
        Ok(())
    }

    /// `indpoint(ic, 0)` and `indpoint(ic, 1)`: new points, or the points of a previous
    /// contour where they meet.
    fn end_points_in_ds(
        &mut self,
        c: &mut Corner,
        ic: usize,
        points: [Point3; 2],
    ) -> Result<(), Failure> {
        for (k, point) in points.into_iter().enumerate() {
            let found = if ic == 0 {
                None
            } else {
                self.previous_point(c, ic, point, 1 - k)?
            };
            c.indpoint[ic][k] = match found {
                Some(ip) => ip,
                None => self.ds.add_point(&TopOpeBRepDSPoint::new(point, c.maxapp)),
            };
        }
        Ok(())
    }

    /// The point `indpoint(ii, k)` of the first previous contour at `point`.
    fn previous_point(
        &mut self,
        c: &Corner,
        ic: usize,
        point: Point3,
        k: usize,
    ) -> Result<Option<i32>, Failure> {
        for ii in (0..ic).filter(|&ii| !c.sharp[ii]) {
            let tpt = self.ds.change_point(c.indpoint[ii][k])?.point();
            if point.distance(&tpt) < 1.0e-4 {
                return Ok(Some(c.indpoint[ii][k]));
            }
        }
        Ok(None)
    }
}

/// The end of the contour `ic` at the vertex replaced by the boundary of the plate: its
/// curve, points, parameters and pcurve, and the common points and parameters of its
/// surface data.
fn update_stripe(
    c: &Corner,
    ic: usize,
    (pcurve, points): (&Handle<dyn Geom2dCurve>, [Point3; 2]),
) -> Result<(), Failure> {
    let (icplus, icmoins) = indices(c.nedge, ic);
    let isfirst = c.sens[ic] == 1;
    let (isurf1, isurf2) = (3 - c.jf[ic], c.jf[ic]);
    let pc = pcurve
        .get()
        .ok_or_else(|| Failure::new("PerformMoreThreeCorner"))?;
    {
        let mut s = lock(c.cd(ic)?);
        if isurf1 == 2 {
            s.set_orientation_at_end(Orientation::Reversed, isfirst);
        }
        s.set_curve(c.indcurve3d[c.n3d as usize], isfirst);
        s.set_index_point(c.indpoint[ic][0], isfirst, isurf1);
        s.set_index_point(c.indpoint[ic][1], isfirst, isurf2);
        s.set_parameters(isfirst, pc.first_parameter(), pc.last_parameter());
    }
    let fd = surf_data_at(c.cd(ic)?, c.i[ic][icmoins])?;
    {
        let mut g = lock(&fd);
        for (isurf, point) in [(isurf1, points[0]), (isurf2, points[1])] {
            let mut cp = CommonPoint::new();
            cp.set_point(point);
            *g.change_vertex(isfirst, isurf) = cp;
        }
        g.change_interference(isurf1)
            .set_parameter(c.p[ic][icmoins], isfirst);
        g.change_interference(isurf2)
            .set_parameter(c.p[ic][icplus], isfirst);
    }
    *lock(c.cd(ic)?).change_pcurve(isfirst) = pcurve.clone();
    Ok(())
}

impl ChFi3dBuilder {
    /// The points where the sharp edges are cut: the vertex when two fillets are
    /// tangent, else the common point of a neighbouring contour there, else a new point.
    pub(super) fn sharp_points(&mut self, c: &mut Corner) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if !c.sharp[ic] {
                continue;
            }
            let (icplus, icmoins) = indices(c.nedge, ic);
            let pe = BRepAdaptorCurve::with_edge(&c.evive[ic])?.value(c.p[ic][icplus])?;
            let tpe = TopOpeBRepDSPoint::new(pe, BRepTool::edge_tolerance(&c.evive[ic])?);
            if c.deuxconges {
                c.ivtx = self.ds.add_shape(c.v1.as_ref());
                c.indpoint[ic] = [c.ivtx, c.ivtx];
            }
            let tol = f64::max(1.0e-4, c.tolcp);
            if !c.sharp[icplus]
                && end_point(c, icplus, c.i[icplus][ic], 3 - c.jf[icplus])?
                    .point()
                    .distance(&pe)
                    <= tol
            {
                let ip = c.indpoint[icplus][0];
                c.indpoint[ic] = [ip, ip];
            }
            if !c.sharp[icmoins]
                && end_point(c, icmoins, c.i[icmoins][ic], c.jf[icmoins])?
                    .point()
                    .distance(&pe)
                    <= tol
                && c.indpoint[ic][0] == 0
            {
                let ip = c.indpoint[icmoins][1];
                c.indpoint[ic] = [ip, ip];
            }
            if c.indpoint[ic][0] == 0 {
                let ip = self.ds.add_point(&tpe);
                c.indpoint[ic] = [ip, ip];
            }
        }
        Ok(())
    }
}
