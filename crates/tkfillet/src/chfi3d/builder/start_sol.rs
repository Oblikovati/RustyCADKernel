// Rust translation of ChFi3d_Builder::StartSol(Stripe, HGuide, HS1, HS2, I1, I2, P1, P2,
// First) (ChFi3d_Builder_2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The first section of a contour: tried along the spine on the faces of its edges, then
//! on the tangent neighbours of the faces.
use super::ChFi3dBuilderOps;
use super::compute::spine_of;
use super::faces_search::{bon_voisin, curve_on_surface, guide_plane, pcurve_value, projection};
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::{ElSpine, ErrorStatus, SpineHandle, StripeHandle, lock};
use crate::chfi3d::next_side;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;
use std::f64::consts::PI;
use std::rc::Rc;

/// The faces, the domains and the guide of a start search.
pub(crate) struct StartFaces<'a> {
    /// `HS1`, `HS2`: the faces (set by the search).
    pub hs: (
        &'a mut Handle<BRepAdaptorSurface>,
        &'a mut Handle<BRepAdaptorSurface>,
    ),
    /// `I1`, `I2`: their domains.
    pub it: (
        &'a Rc<BRepTopAdaptorTopolTool>,
        &'a Rc<BRepTopAdaptorTopolTool>,
    ),
}

/// The state of one try of the start search: the edge, its faces and the side.
struct Try {
    iedge: i32,
    cured: Edge,
    f1: Face,
    f2: Face,
    choix: i32,
    tol_e: f64,
}

/// A surface adaptor handle as a general one.
fn adaptor(h: &Handle<BRepAdaptorSurface>) -> Handle<dyn Adaptor3dSurface> {
    h.clone().cast()
}

/// The start solution `(P1, P2)` in a math vector.
pub(crate) fn soldep_of(p1: Point2, p2: Point2) -> Result<Vector<'static>, Failure> {
    let mut v = Vector::with_bounds(1, 4)?;
    v[1] = p1.x();
    v[2] = p1.y();
    v[3] = p2.x();
    v[4] = p2.y();
    Ok(v)
}

impl dyn ChFi3dBuilderOps + '_ {
    /// The faces of the `iedge`-th edge and the side of the fillet on them.
    fn start_faces(
        &self,
        stripe: &StripeHandle,
        spine: &SpineHandle,
        iedge: i32,
        faces: &mut StartFaces<'_>,
    ) -> Result<Try, Failure> {
        let cured = lock(spine).edges(iedge)?.clone();
        let tol_e = BRepTool::edge_tolerance(&cured)?;
        self.base()
            .conex_faces(spine, iedge, faces.hs.0, faces.hs.1)?;
        let choix = self.side_choice(stripe, faces.hs.0, faces.hs.1);
        Ok(Try {
            iedge,
            cured,
            f1: faces.hs.0.face(),
            f2: faces.hs.1.face(),
            choix,
            tol_e,
        })
    }

    /// `ChFi3d::NextSide` of the faces for the stripe.
    pub(crate) fn side_choice(
        &self,
        stripe: &StripeHandle,
        hs1: &Handle<BRepAdaptorSurface>,
        hs2: &Handle<BRepAdaptorSurface>,
    ) -> i32 {
        let (mut or1, mut or2) = (hs1.face().orientation(), hs2.face().orientation());
        let s = lock(stripe);
        next_side(
            &mut or1,
            &mut or2,
            s.orientation_on_face1(),
            s.orientation_on_face2(),
            s.choix(),
        )
    }

    /// The start point on the first face moved inside it, off the edge.
    fn inside_start(
        &self,
        tr: &Try,
        woned: f64,
        hs1: &Handle<BRepAdaptorSurface>,
        i1: &Rc<BRepTopAdaptorTopolTool>,
    ) -> Result<Point2, Failure> {
        let mut cured = tr.cured.clone();
        cured.set_orientation(Orientation::Forward);
        let mut f1forward = tr.f1.clone();
        f1forward.set_orientation(Orientation::Forward);
        let pc = curve_on_surface(&cured, &f1forward)?;
        i1.initialize_surface(&adaptor(hs1))?;
        let (mut p1, mut derive) = pc
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "StartSol"))?
            .d1(woned)?;
        if derive.magnitude() > precision::p_confusion() {
            derive.normalize()?;
            derive.rotate(PI / 2.);
            let a_s = BRepAdaptorSurface::with_face(&tr.f1, true)?;
            let (res_u, res_v) = (a_s.u_resolution(tr.tol_e)?, a_s.v_resolution(tr.tol_e)?);
            derive *= 2. * (derive.x().abs() * res_u + derive.y().abs() * res_v);
            let p2d = p1.translated(derive);
            if i1.classify(p2d, res_u.min(res_v), false)? == State::In {
                p1 = p2d;
            } else {
                let p2d = p1.translated(-derive);
                if i1.classify(p2d, res_u.min(res_v), false)? == State::In {
                    p1 = p2d;
                }
            }
        }
        Ok(p1)
    }

    /// The point on the second face of the edge at `woned` (the edge forward, reversed
    /// on a seam, with `seam`; as it is otherwise).
    fn second_start(tr: &Try, woned: f64, seam: bool) -> Result<Point2, Failure> {
        let mut cured = tr.cured.clone();
        if seam {
            cured.set_orientation(Orientation::Forward);
            if tr.f1.is_same(&tr.f2) {
                cured.set_orientation(Orientation::Reversed);
            }
        }
        let mut f2forward = tr.f2.clone();
        f2forward.set_orientation(Orientation::Forward);
        pcurve_value(&curve_on_surface(&cured, &f2forward)?, woned)
    }

    /// The first section from a start solution: `Some((P1, P2))` when found.
    #[allow(clippy::too_many_arguments)]
    fn first_section_at(
        &self,
        spine: &SpineHandle,
        hguide: &Handle<ElSpine>,
        choix: i32,
        faces: &StartFaces<'_>,
        w: f64,
        soldep: &mut Vector<'static>,
    ) -> Result<(bool, State, State), Failure> {
        let (d1, d2): (WalkingDomain, WalkingDomain) = (faces.it.0.clone(), faces.it.1.clone());
        self.perform_first_section(
            spine,
            hguide,
            choix,
            (faces.hs.0, faces.hs.1),
            (&d1, &d2),
            w,
            soldep,
        )
    }

    /// The point of the spine at `woned` on its `iedge`-th edge.
    fn spine_point(
        spine: &SpineHandle,
        iedge: i32,
        woned: f64,
    ) -> Result<rustycad_tkmath::gp::Point3, Failure> {
        lock(spine).current_elementary_spine(iedge)?.value(woned)
    }

    /// `StartSol(Stripe, HGuide, HS1, HS2, I1, I2, P1, P2, First)`: the first section of
    /// the contour, its solution `(P1, P2)` and its guide parameter `First`
    /// (Standard_Failure "StartSol echec" with the status StartsolFailure when none).
    pub(crate) fn start_sol(
        &self,
        stripe: &StripeHandle,
        hguide: &Handle<ElSpine>,
        faces: &mut StartFaces<'_>,
    ) -> Result<(Point2, Point2, f64), Failure> {
        let spine = spine_of(stripe)?;
        let (nbed, wf, wl, pfirst, plast) = {
            let sp = lock(&spine);
            let nbed = sp.nb_edges();
            let (f1, l1) = (sp.first_parameter_of(1)?, sp.last_parameter_of(1)?);
            let (fn_, ln) = (sp.first_parameter_of(nbed)?, sp.last_parameter_of(nbed)?);
            (
                nbed,
                0.9981 * f1 + 0.0019 * l1,
                0.9973 * ln + 0.0027 * fn_,
                f1,
                ln,
            )
        };
        let nbessaimax = (3 * nbed).max(10);
        let unsurnbessaimax = 1. / f64::from(nbessaimax);
        let els: &dyn Adaptor3dCurve = &**hguide;
        let mut pext = ExtPC::new();
        pext.initialize(els, pfirst, plast, precision::confusion())?;
        let tolapp3d = self.base().tolapp3d;
        let mut tr: Option<Try> = None;
        for nbessai in 0..=nbessaimax {
            let t = f64::from(nbessai) * unsurnbessaimax;
            let mut w = wf * (1. - t) + wl * t;
            let ie = lock(&spine).index_at(w, true)?;
            if tr.as_ref().is_none_or(|tr| tr.iedge != ie) {
                tr = Some(self.start_faces(stripe, &spine, ie, faces)?);
            }
            let tr = tr.as_ref().ok_or_else(|| Failure::new("StartSol"))?;
            let woned = lock(&spine).parameter_on_edge(tr.iedge, w, true)?;
            let p1 = self.inside_start(tr, woned, faces.hs.0, faces.it.0)?;
            let p2 = Self::second_start(tr, woned, true)?;
            faces.it.1.initialize_surface(&adaptor(faces.hs.1))?;
            let mut soldep = soldep_of(p1, p2)?;
            let pnt = Self::spine_point(&spine, tr.iedge, woned)?;
            if projection(&mut pext, pnt, els, &mut w, tolapp3d)?
                && self
                    .first_section_at(&spine, hguide, tr.choix, faces, w, &mut soldep)?
                    .0
            {
                return Ok((
                    Point2::new(soldep[1], soldep[2]),
                    Point2::new(soldep[3], soldep[4]),
                    w,
                ));
            }
        }
        for nbessai in 0..=nbessaimax {
            let t = f64::from(nbessai) * unsurnbessaimax;
            let mut w = wf * (1. - t) + wl * t;
            let iedge = lock(&spine).index_at(w, true)?;
            let tr = self.start_faces(stripe, &spine, iedge, faces)?;
            let woned = lock(&spine).parameter_on_edge(iedge, w, true)?;
            let mut f1forward = tr.f1.clone();
            f1forward.set_orientation(Orientation::Forward);
            let p1 = pcurve_value(&curve_on_surface(&tr.cured, &f1forward)?, woned)?;
            let p2 = Self::second_start(&tr, woned, false)?;
            faces.it.0.initialize_surface(&adaptor(faces.hs.0))?;
            faces.it.1.initialize_surface(&adaptor(faces.hs.1))?;
            let mut soldep = soldep_of(p1, p2)?;
            let pnt = Self::spine_point(&spine, iedge, woned)?;
            if projection(&mut pext, pnt, els, &mut w, tolapp3d)?
                && let Some(found) =
                    self.start_on_neighbours(stripe, &spine, hguide, faces, &tr, w, &mut soldep)?
            {
                return Ok(found);
            }
        }
        lock(&spine).set_error_status(ErrorStatus::StartsolFailure);
        Err(Failure::new("StartSol echec"))
    }

    /// The second phase of `StartSol`: the first section at `w` retried on the tangent
    /// neighbours of the faces where its ends fall outside them.
    #[allow(clippy::too_many_arguments)]
    fn start_on_neighbours(
        &self,
        stripe: &StripeHandle,
        spine: &SpineHandle,
        hguide: &Handle<ElSpine>,
        faces: &mut StartFaces<'_>,
        tr: &Try,
        w: f64,
        soldep: &mut Vector<'static>,
    ) -> Result<Option<(Point2, Point2, f64)>, Failure> {
        let (_, mut pos1, mut pos2) =
            self.first_section_at(spine, hguide, tr.choix, faces, w, soldep)?;
        let (p, v) = hguide.d1(w)?;
        let plane = guide_plane(p, v)?;
        let (mut f1, mut f2) = (tr.f1.clone(), tr.f2.clone());
        let ef_map = &self.base().my_ef_map;
        let tolapp3d = self.base().tolapp3d;
        let mut bonvoisin = true;
        let mut nb_changement = 1;
        while bonvoisin && nb_changement < 5 {
            if pos1 != State::In {
                let (mut x, mut y) = (soldep[1], soldep[2]);
                bonvoisin = bon_voisin(
                    p,
                    faces.hs.0,
                    &f1,
                    &plane,
                    &tr.cured,
                    (&mut x, &mut y),
                    ef_map,
                    tolapp3d,
                )?;
                soldep[1] = x;
                soldep[2] = y;
            }
            if pos2 != State::In && bonvoisin {
                let (mut x, mut y) = (soldep[3], soldep[4]);
                bonvoisin = bon_voisin(
                    p,
                    faces.hs.1,
                    &f2,
                    &plane,
                    &tr.cured,
                    (&mut x, &mut y),
                    ef_map,
                    tolapp3d,
                )?;
                soldep[3] = x;
                soldep[4] = y;
            }
            if bonvoisin {
                f1 = faces.hs.0.face();
                f2 = faces.hs.1.face();
                let choix = self.side_choice(stripe, faces.hs.0, faces.hs.1);
                faces.it.0.initialize_surface(&adaptor(faces.hs.0))?;
                faces.it.1.initialize_surface(&adaptor(faces.hs.1))?;
                let (ok, p1s, p2s) =
                    self.first_section_at(spine, hguide, choix, faces, w, soldep)?;
                pos1 = p1s;
                pos2 = p2s;
                if ok {
                    return Ok(Some((
                        Point2::new(soldep[1], soldep[2]),
                        Point2::new(soldep[3], soldep[4]),
                        w,
                    )));
                }
            }
            nb_changement += 1;
        }
        Ok(None)
    }
}
