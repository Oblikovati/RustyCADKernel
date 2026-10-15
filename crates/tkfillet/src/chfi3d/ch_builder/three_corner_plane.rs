// Rust translation of the plane corner of ChFi3d_ChBuilder::PerformThreeCorner and the
// file statics CoPlanar, BoundSurf and ComputeIntersection (ChFi3d_ChBuilder_C3.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The plane corner of three chamfers: a plane through the points where they meet, cut
//! by the three chamfers and by the face of the pivot.
use super::ChFi3dChBuilder;
use super::three_corner::{ChThreeCorner, CornerPoints};
use crate::chfi_ds::{SurfDataHandle, lock};
use crate::chfi3d::{
    IntersectionCurves, boite, bound_surf, compute_curves, parameters, project_pcurv, trim_curve,
};
use crate::chfikpart::{index_curve_in_ds, index_surface_in_ds};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSDataStructure};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3, Vector3};

/// `CoPlanar(PntA, PntB, PntC, PntD)`: the fourth point is in the plane of the three
/// others (or the three others are aligned).
pub(super) fn co_planar(a: Point3, b: Point3, c: Point3, d: Point3) -> bool {
    let (ab, ac, ad) = (
        Vector3::between(a, b),
        Vector3::between(a, c),
        Vector3::between(a, d),
    );
    let (nor2ab, nor2ac) = (ab.square_magnitude(), ac.square_magnitude());
    let proabac = ab.dot(ac);
    let alpha = nor2ab * nor2ac - proabac * proabac;
    if alpha < precision::confusion() {
        return true;
    }
    let (proabad, proacad) = (ab.dot(ad), ac.dot(ad));
    let alpha1 = proabad * nor2ac - proabac * proacad;
    let alpha2 = proacad * nor2ab - proabac * proabad;
    let vec = ab * alpha1 + ac * alpha2 - ad * alpha;
    vec.magnitude() / alpha < precision::confusion()
}

/// `BoundSurf(S, Pdeb, Pfin)`: the surface bounded around the two points, enlarged by a
/// fifth of the box within its own bounds.
fn bound_around(
    s: &Handle<dyn GeomSurface>,
    pdeb: Point2,
    pfin: Point2,
) -> Result<Handle<GeomAdaptorSurface>, Failure> {
    let g = s
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "BoundSurf"))?;
    let (mut uuu1, mut uuu2, mut vvv1, mut vvv2) = g.bounds();
    let (uu1, uu2, vv1, vv2) = boite(pdeb, pfin);
    let step = (uu2 - uu1).max(vv2 - vv1) * 0.2;
    uuu1 = (uu1 - step).max(uuu1);
    uuu2 = (uu2 + step).min(uuu2);
    vvv1 = (vv1 - step).max(vvv1);
    vvv2 = (vv2 + step).min(vvv2);
    let gas = GeomAdaptorSurface::with_surface(s)?;
    gas.load_bounds(s, (uuu1, uuu2, vvv1, vvv2))?;
    Ok(Handle::new(gas))
}

/// What `ComputeIntersection` gives: the curve and pcurves, the derivatives of the
/// chamfer at the start.
struct CornerCut {
    curves: IntersectionCurves,
    derudeb: Vector3,
    dervdeb: Vector3,
}

/// `ComputeIntersection(DStr, SD, SDCoin, pdeb, p2ddeb, pfin, p2dfin, ...)`: the
/// intersection of the chamfer `sd` with the plane of the corner between the two points;
/// None when it fails.
fn compute_intersection(
    dstr: &TopOpeBRepDSDataStructure,
    (sd, coin): (&SurfDataHandle, &SurfDataHandle),
    (pdeb, p2ddeb): (Point3, Point2),
    (pfin, p2dfin): (Point3, Point2),
    (tol3d, tol2d): (f64, f64),
) -> Result<Option<CornerCut>, Failure> {
    let hs1 = bound_surf(dstr, sd, 1, 2)?;
    let gpl = dstr.surface(lock(coin).surf()).surface().clone();
    let gsd = dstr.surface(lock(sd).surf()).surface().clone();
    let d1 = gsd
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "ComputeIntersection"))?
        .d1(p2ddeb.x(), p2ddeb.y())?;
    let (ud, vd) = parameters(&gpl, pdeb)?;
    let (uf, vf) = parameters(&gpl, pfin)?;
    let pardeb = [p2ddeb.x(), p2ddeb.y(), ud, vd];
    let parfin = [p2dfin.x(), p2dfin.y(), uf, vf];
    let hs2 = bound_around(&gpl, Point2::new(ud, vd), Point2::new(uf, vf))?;
    let mut curves = IntersectionCurves::default();
    let (g1, g2): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
        (hs1.cast(), hs2.cast());
    if !compute_curves(
        (&g1, &g2),
        (pardeb, parfin),
        (tol3d, tol2d),
        true,
        &mut curves,
    )? {
        return Ok(None);
    }
    Ok(Some(CornerCut {
        curves,
        derudeb: d1.d1u,
        dervdeb: d1.d1v,
    }))
}

/// The plane corner: the plane, its edges with the three chamfers (the curve on the
/// pivot side being its second interference), and with the face of the pivot unless the
/// corner is a triangle.
pub(super) struct PlaneCorner {
    pub icf: i32,
    pub icl: i32,
    pub debpc1: Handle<dyn Geom2dCurve>,
    pub finpc1: Handle<dyn Geom2dCurve>,
}

impl ChFi3dChBuilder {
    /// The plane corner through the points, oriented as the face of the pivot, cut by
    /// the chamfers (StdFail_NotDone when a cut fails).
    pub(super) fn plane_corner(
        &mut self,
        tc: &ChThreeCorner,
        pts: &CornerPoints,
        coin: &SurfDataHandle,
        c1triangle: bool,
    ) -> Result<PlaneCorner, Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        let p3d = pts.p3d;
        let v1 = Vector3::between(p3d[pivot], p3d[deb]);
        let v2 = Vector3::between(p3d[pivot], p3d[fin]);
        let ndir = Direction3::from_vector(v1.crossed(v2))?;
        let xdir = Direction3::from_vector(Vector3::between(p3d[fin], p3d[deb]))?;
        let mut plan_ax3 = Axis3::new(p3d[pivot], ndir, xdir)?;
        if Vector3::from(plan_ax3.y_direction()).dot(v1) <= 0. {
            plan_ax3.y_reverse();
        }
        let gpl = Handle::new(GeomPlane::new(plan_ax3));
        let gpl_s: Handle<dyn GeomSurface> = gpl.clone().cast();
        let isurf = index_surface_in_ds(&gpl_s, &mut self.base.ds);
        let mut norface = pts.norpl;
        if tc.face[pivot].orientation() == Orientation::Reversed {
            norface.reverse();
        }
        let pos = gpl.pln().position();
        let norcoin = Vector3::from(pos.x_direction()).crossed(Vector3::from(pos.y_direction()));
        {
            let mut c = lock(coin);
            c.change_surf(isurf);
            *c.change_orientation() = if norcoin.dot(norface) <= 0. {
                Orientation::Reversed
            } else {
                Orientation::Forward
            };
        }
        let tols = (self.base.tolapp3d, self.base.tol2d);
        let not_done = |what: &str| Failure::with_message("StdFail_NotDone", what);
        let fdpiv = tc.sd(pivot, deb)?;
        let piv = compute_intersection(
            &self.base.ds,
            (&fdpiv, coin),
            (p3d[fin], pts.p2d[fin]),
            (p3d[deb], pts.p2d[deb]),
            tols,
        )?
        .ok_or_else(|| not_done("echec calcul intersection coin-pivot"))?;
        let norpiv = piv.derudeb.crossed(piv.dervdeb);
        let fddeb = tc.sd(deb, pivot)?;
        let fi = |fd: &SurfDataHandle, on_s: i32, u: f64| -> Result<Point2, Failure> {
            let pc = lock(fd).interference(on_s).pcurve_on_surf().clone();
            pc.value(u)
        };
        let u1 = if c1triangle {
            tc.p[deb][fin]
        } else {
            tc.p[deb][pivot]
        };
        let p2d1 = fi(&fddeb, tc.jf[deb][fin], u1)?;
        let p2d2 = fi(&fddeb, tc.jf[deb][pivot], tc.p[deb][pivot])?;
        let cdeb = compute_intersection(
            &self.base.ds,
            (&fddeb, coin),
            (p3d[pivot], p2d1),
            (p3d[fin], p2d2),
            tols,
        )?
        .ok_or_else(|| not_done("echec calcul intersection coin-deb"))?;
        let icf = self.base.ds.add_curve(&mut TopOpeBRepDSCurve::new(
            &cdeb.curves.c3d,
            cdeb.curves.tolreached,
            false,
        ));
        let fdfin = tc.sd(fin, pivot)?;
        let (p3dface, u1) = if c1triangle {
            (p3d[pivot], tc.p[fin][deb])
        } else {
            (p3d[3], tc.p[fin][pivot])
        };
        let p2d1 = fi(&fdfin, tc.jf[fin][deb], u1)?;
        let p2d2 = fi(&fdfin, tc.jf[fin][pivot], tc.p[fin][pivot])?;
        let cfin = compute_intersection(
            &self.base.ds,
            (&fdfin, coin),
            (p3dface, p2d1),
            (p3d[deb], p2d2),
            tols,
        )?
        .ok_or_else(|| not_done("echec calcul intersection coin-face"))?;
        let icl = self.base.ds.add_curve(&mut TopOpeBRepDSCurve::new(
            &cfin.curves.c3d,
            cfin.curves.tolreached,
            false,
        ));
        let face_cut = if c1triangle {
            None
        } else {
            self.face_cut(tc, &gpl_s, (p3d[pivot], p3dface))?
        };
        {
            let mut c = lock(coin);
            c.change_vertex(true, 1).set_point(p3d[pivot]);
            c.change_vertex(true, 2).set_point(p3d[fin]);
            c.change_vertex(false, 1)
                .set_point(if c1triangle { p3d[pivot] } else { p3d[3] });
            c.change_vertex(false, 2).set_point(p3d[deb]);
        }
        let trans1 = if norcoin.dot(pts.norpl) <= 0. {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        if c1triangle {
            lock(coin).change_interference_on_s1().set_interference(
                0,
                trans1,
                &Handle::default(),
                &Handle::default(),
            );
        } else if let Some((gcface, facepc1, facepc2)) = face_cut {
            let gc: Handle<dyn GeomCurve> = gcface.clone().cast();
            let igcface = index_curve_in_ds(&gc, &mut self.base.ds);
            let c = gcface
                .get()
                .ok_or_else(|| Failure::new("PerformThreeCorner: null curve"))?;
            let mut g = lock(coin);
            let fi1 = g.change_interference_on_s1();
            fi1.set_interference(igcface, trans1, &facepc1, &facepc2);
            fi1.set_first_parameter(c.first_parameter());
            fi1.set_last_parameter(c.last_parameter());
        } else {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "PerformThreeCorner: no cut of the face",
            ));
        }
        let trans2 = if norcoin.dot(norpiv) <= 0. {
            Orientation::Reversed
        } else {
            Orientation::Forward
        };
        let igcpiv = index_curve_in_ds(&piv.curves.c3d, &mut self.base.ds);
        let c = piv
            .curves
            .c3d
            .get()
            .ok_or_else(|| Failure::new("PerformThreeCorner: null curve"))?;
        let mut g = lock(coin);
        let fi2 = g.change_interference_on_s2();
        fi2.set_interference(igcpiv, trans2, &piv.curves.pc1, &piv.curves.pc2);
        fi2.set_first_parameter(c.first_parameter());
        fi2.set_last_parameter(c.last_parameter());
        Ok(PlaneCorner {
            icf,
            icl,
            debpc1: cdeb.curves.pc1.clone(),
            finpc1: cfin.curves.pc1.clone(),
        })
    }

    /// The cut of the plane by the face of the pivot between the two points, and its
    /// pcurves on the face and on the plane (None unless there is exactly one line).
    #[allow(clippy::type_complexity)]
    fn face_cut(
        &self,
        tc: &ChThreeCorner,
        gpl: &Handle<dyn GeomSurface>,
        (p1, p2): (Point3, Point3),
    ) -> Result<
        Option<(
            Handle<rustycad_tkg3d::geom::GeomTrimmedCurve>,
            Handle<dyn Geom2dCurve>,
            Handle<dyn Geom2dCurve>,
        )>,
        Failure,
    > {
        let facebid = BRepAdaptorSurface::with_face(&tc.face[tc.pivot], true)?;
        let surfbid = facebid.transformed_surface().geom_surface_transformed()?;
        let mut inter = GeomIntIntSS::new();
        inter.perform(
            gpl,
            &surfbid,
            precision::intersection(),
            GeomIntIntSSApprox::default(),
        )?;
        if !inter.is_done() || inter.nb_lines() != 1 {
            return Ok(None);
        }
        let gcface = trim_curve(&inter.line(1)?, p1, p2)?;
        let gac: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_curve(&gcface.clone().cast())?).cast();
        let gas: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(gpl)?).cast();
        let gaf: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&tc.face[tc.pivot], true)?).cast();
        let (mut facepc1, mut facepc2): (Handle<dyn Geom2dCurve>, Handle<dyn Geom2dCurve>) =
            (Handle::default(), Handle::default());
        let mut tolr = 0.;
        project_pcurv(&gac, &gaf, &mut facepc1, self.base.tolapp3d, &mut tolr)?;
        project_pcurv(&gac, &gas, &mut facepc2, self.base.tolapp3d, &mut tolr)?;
        Ok(Some((gcface, facepc1, facepc2)))
    }
}
