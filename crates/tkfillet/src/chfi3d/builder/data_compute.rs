// Rust translation of ChFi3d_Builder::ComputeData(Data, HGuide, Spine, Lin, S1, I1, S2,
// I2, Func, FInv, ...) and the file statics nbedconnex, IsVois, IsObst and CompBlendPoint
// (ChFi3d_Builder_6.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The walking of a fillet between two faces along its guide, retried with smaller
//! steps until it has enough sections.
use super::{ChFi3dBuilder, SurfOut, WalkSettings};
use crate::blend::{BlendFuncInv, BlendFunction, BlendPoint};
use crate::brep_blend::{BRepBlendLineHandle, BRepBlendWalking, WalkingDomain, WalkingTolerances};
use crate::chfi_ds::{ChamfMode, CommonPoint, ElSpine, Map, SpineHandle, SurfDataHandle, lock};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::math::Vector;

/// `nbedconnex(L)`: the number of distinct edges of the list.
fn nbedconnex(l: &[Shape]) -> i32 {
    let mut nb = 0;
    for (i, curs) in l.iter().enumerate() {
        if !l[..i].iter().any(|s| curs.is_same(s)) {
            nb += 1;
        }
    }
    nb
}

/// `IsVois(E, Vref, VEMap, DONE, prof, profmax)`: `vref` is reached from `e` through at
/// most `profmax` vertices where more than two edges meet.
fn is_vois(
    e: &Edge,
    vref: &Vertex,
    ve_map: &Map,
    done: &mut MapOfShape,
    prof: i32,
    profmax: i32,
) -> Result<bool, Failure> {
    if prof > profmax || done.contains(e.as_ref()) {
        return Ok(false);
    }
    let (v1, v2) = topexp::vertices(e, false);
    if vref.is_same(&v1) || vref.is_same(&v2) {
        return Ok(true);
    }
    done.add(e.clone().into());
    let l1: Vec<Shape> = ve_map.find_from_key(v1.as_ref())?.iter().cloned().collect();
    let i1 = nbedconnex(&l1);
    let l2: Vec<Shape> = ve_map.find_from_key(v2.as_ref())?.iter().cloned().collect();
    for cur in l1.iter().chain(l2.iter()) {
        let cur_e = topods::edge(cur)?;
        let next_prof = if i1 <= 2 { prof } else { prof + 1 };
        if is_vois(&cur_e, vref, ve_map, done, next_prof, profmax)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `IsObst(CP, Vref, VEMap)`: the arc of the common point is not a near neighbour of
/// `vref`: an obstacle.
pub(super) fn is_obst(cp: &CommonPoint, vref: &Vertex, ve_map: &Map) -> Result<bool, Failure> {
    if !cp.is_on_arc() {
        return Ok(false);
    }
    let mut done = MapOfShape::default();
    Ok(!is_vois(cp.arc()?, vref, ve_map, &mut done, 0, 4)?)
}

/// `CompBlendPoint(V, E, W, F1, F2, BP)`: the section degenerated to the vertex at `w`,
/// from the pcurves of the edge on the two faces (None when one is missing).
fn comp_blend_point(
    v: &Vertex,
    e: &Edge,
    w: f64,
    f1: &Face,
    f2: &Face,
) -> Result<Option<BlendPoint>, Failure> {
    let p3d = BRepTool::pnt(v)?;
    let at = |f: &Face| -> Result<Option<rustycad_tkmath::gp::Point2>, Failure> {
        let param = BRepTool::parameter_on_face(v, e, f)?;
        let pc = BRepTool::curve_on_face(e, f)?.pcurve;
        match pc.get() {
            Some(c) => Ok(Some(c.value(param)?)),
            None => Ok(None),
        }
    };
    let Some(p1) = at(f1)? else {
        return Ok(None);
    };
    let Some(p2) = at(f2)? else {
        return Ok(None);
    };
    Ok(Some(BlendPoint::on_surfaces(
        p3d,
        p3d,
        w,
        [p1.x(), p1.y(), p2.x(), p2.y()],
    )))
}

/// A math vector of the values (`math_Vector(1, n)`).
pub(super) fn vec_of(values: &[f64]) -> Result<Vector<'static>, Failure> {
    let mut v = Vector::with_bounds(1, values.len() as i32)?;
    for (i, x) in (1..).zip(values) {
        v[i] = *x;
    }
    Ok(v)
}

/// The face of a surface adaptor when it is a face adaptor (null otherwise).
pub(super) fn face_of(s: &Handle<dyn Adaptor3dSurface>) -> Face {
    let hs: Handle<BRepAdaptorSurface> = s.clone().cast();
    hs.get().map(|h| h.face()).unwrap_or_default()
}

/// The offset elementary spine of `hguide` for the chamfers of constant throat with
/// penetration.
pub(super) fn offset_of(
    spine: Option<&SpineHandle>,
    hguide: &Handle<ElSpine>,
) -> Option<Handle<ElSpine>> {
    let sp = spine?;
    let mut g = lock(sp);
    if g.mode() != ChamfMode::ConstThroatWithPenetrationChamfer {
        return None;
    }
    let ll = g.change_el_spines().clone();
    let off = g.change_offset_el_spines().clone();
    let mut found = None;
    for (es, o) in ll.iter().zip(off.iter()) {
        if es == hguide {
            found = Some(o.clone());
        }
    }
    found
}

/// The arguments of `ComputeData` on two faces.
pub(crate) struct ComputeCall<'a> {
    pub hguide: &'a Handle<ElSpine>,
    pub spine: Option<&'a SpineHandle>,
    pub s1: &'a Handle<dyn Adaptor3dSurface>,
    pub i1: &'a WalkingDomain,
    pub s2: &'a Handle<dyn Adaptor3dSurface>,
    pub i2: &'a WalkingDomain,
    pub pfirst: f64,
    pub walk: WalkSettings,
    pub soldep: &'a Vector<'static>,
    pub rec: (bool, bool),
}

/// The walking state shared by the steps of `ComputeData`.
pub(super) struct WalkRun {
    pub walk: BRepBlendWalking,
    pub lin: BRepBlendLineHandle,
    pub sp_first: f64,
    pub sp_last: f64,
    pub target: f64,
    pub reverse: bool,
    pub tol_guide: f64,
    pub faces: (Face, Face),
}

impl ChFi3dBuilder {
    /// The singular sections at the tangent ends of the spine added to the walking.
    fn add_tangent_ends(
        &self,
        walk: &mut BRepBlendWalking,
        func: &mut dyn BlendFunction,
        spine: &SpineHandle,
        (f1, f2): (&Face, &Face),
    ) -> Result<(), Failure> {
        for first in [true, false] {
            let (v, e, param) = {
                let sp = lock(spine);
                if !sp.is_tangency_extremity(first) {
                    continue;
                }
                if first {
                    (
                        sp.first_vertex()?,
                        sp.edges(1)?.clone(),
                        sp.first_parameter(),
                    )
                } else {
                    let nb = sp.nb_edges();
                    (
                        sp.last_vertex()?,
                        sp.edges(nb)?.clone(),
                        sp.last_parameter()?,
                    )
                }
            };
            if let Some(bp) = comp_blend_point(&v, &e, param, f1, f2)? {
                let (u1, v1) = bp.parameters_on_s1()?;
                let (u2, v2) = bp.parameters_on_s2()?;
                let vec = super::start_sol::soldep_of(
                    rustycad_tkmath::gp::Point2::new(u1, v1),
                    rustycad_tkmath::gp::Point2::new(u2, v2),
                )?;
                func.set_param(param)?;
                if func.is_solution(&vec, self.tolapp3d)? {
                    walk.add_singular_point(&bp);
                }
            }
        }
        Ok(())
    }

    /// The walking of `ComputeData` retried with smaller steps until it has at least
    /// `nbptmin` sections at both ends; false when it fails.
    #[allow(clippy::too_many_arguments)]
    fn walk_sections(
        &self,
        run: &mut WalkRun,
        (func, finv): (&mut dyn BlendFunction, &mut dyn BlendFuncInv),
        call: &ComputeCall<'_>,
        (first, last): (f64, f64),
        new_first: f64,
        par_sol: Vector<'static>,
        offset: &Option<Handle<ElSpine>>,
    ) -> Result<bool, Failure> {
        let nbptmin = 4;
        let mut ms = call.walk.max_step;
        let (forward, inside) = (call.walk.forward, call.walk.inside);
        let (mut again, mut tchernobyl, mut u1sov, mut u2sov) = (0, false, 0., 0.);
        if (last - first).abs() <= ms * 5.
            && (last - first).abs() >= 0.01 * (new_first - run.target).abs()
        {
            ms = (last - first).abs() * 0.2;
        }
        while again < 3 {
            if again == 0 && ms < 5. * run.tol_guide {
                ms = 5. * run.tol_guide;
            } else if 5. * run.tol_guide > ms {
                run.tol_guide = ms / 5.;
            }
            let settings = WalkingTolerances {
                max_step: ms,
                tol3d: self.tolapp3d,
                tol_guide: run.tol_guide,
                fleche: call.walk.fleche,
            };
            run.walk.perform(
                func,
                finv,
                (new_first, run.target),
                settings,
                &par_sol,
                call.walk.appro,
            )?;
            if !run.walk.is_done() {
                return Ok(false);
            }
            run.lin = run.walk.line()?.clone();
            if call.hguide.is_periodic()? && inside {
                run.sp_first = lock(&run.lin).point(1)?.parameter();
                run.sp_last = run.sp_first + call.hguide.period()?;
                for g in std::iter::once(call.hguide).chain(offset.iter()) {
                    g.set_first_parameter(run.sp_first);
                    g.set_last_parameter(run.sp_last);
                    g.set_origin(run.sp_first)?;
                }
            }
            if inside && !run.walk.complete(func, finv, run.sp_last)? {
                return Ok(false);
            }
            let nbpnt = lock(&run.lin).nb_points();
            if nbpnt == 0 {
                return Ok(false);
            }
            let (fpointpar, lpointpar) = {
                let l = lock(&run.lin);
                (l.point(1)?.parameter(), l.point(nbpnt)?.parameter())
            };
            let factor = 1. / f64::from(nbptmin + 1);
            let mut okdeb = forward && !inside;
            let mut okfin = !forward && !inside;
            {
                let l = lock(&run.lin);
                if !okdeb {
                    okdeb = l.start_point_on_first().nb_point_on_rst() > 0
                        || l.start_point_on_second().nb_point_on_rst() > 0
                        || (fpointpar - first) < 10. * run.tol_guide;
                }
                if !okfin {
                    okfin = l.end_point_on_first().nb_point_on_rst() > 0
                        || l.end_point_on_second().nb_point_on_rst() > 0
                        || (last - lpointpar) < 10. * run.tol_guide;
                }
            }
            if !okdeb || !okfin || nbpnt == 1 {
                if tchernobyl {
                    return Ok(false);
                }
                tchernobyl = true;
                run.walk.check(false);
                ms = if nbpnt == 1 {
                    ms * 0.01
                } else {
                    (lpointpar - fpointpar) / f64::from(nbpnt)
                };
            } else if nbpnt < nbptmin {
                if again == 0 {
                    u1sov = fpointpar;
                    u2sov = lpointpar;
                    ms = (lpointpar - fpointpar) * factor;
                } else if again == 1 {
                    if (fpointpar - u1sov).abs() >= run.tol_guide
                        || (lpointpar - u2sov).abs() >= run.tol_guide
                    {
                        ms = (lpointpar - fpointpar) * factor;
                    } else {
                        return Ok(false);
                    }
                }
                again += 1;
            } else {
                again = 3;
            }
        }
        Ok(true)
    }

    /// `ComputeData(Data, HGuide, Spine, Lin, S1, I1, S2, I2, Func, FInv, PFirst, MaxStep,
    /// Fleche, TolGuide, First, Last, Inside, Appro, Forward, Soldep, Intf, Intl, Gd1, Gd2,
    /// Gf1, Gf2, RecOnS1, RecOnS2)`: the walking of the fillet, its ends on the arcs of the
    /// faces and the extensions; `(done, [Gd1, Gd2, Gf1, Gf2])` with the line in `lin`.
    pub(crate) fn compute_data(
        &self,
        data: &SurfDataHandle,
        call: &ComputeCall<'_>,
        lin: &mut Option<BRepBlendLineHandle>,
        (func, finv): (&mut dyn BlendFunction, &mut dyn BlendFuncInv),
        out: &mut SurfOut,
    ) -> Result<(bool, [bool; 4]), Failure> {
        let offset = offset_of(call.spine, call.hguide);
        {
            let mut d = lock(data);
            d.set_first_extension_value(0.);
            d.set_last_extension_value(0.);
        }
        let faces = (face_of(call.s1), face_of(call.s2));
        let mut walk = BRepBlendWalking::new(call.s1, call.s2, call.i1, call.i2, call.hguide);
        walk.check_2d(false);
        let (sp_first, sp_last) = (
            call.hguide.first_parameter()?,
            call.hguide.last_parameter()?,
        );
        let reverse = !call.walk.forward || call.walk.inside;
        let target = if reverse {
            if out.intf == 0 { out.last } else { sp_first }
        } else if out.intl == 0 {
            out.last
        } else {
            sp_last + sp_last.abs()
        };
        if let Some(sp) = call.spine {
            self.add_tangent_ends(&mut walk, func, sp, (&faces.0, &faces.1))?;
        }
        let tol_guide = call.walk.tol_guide;
        let (new_first, par_sol) = if call.rec.0 || call.rec.1 {
            match walk.perform_first_section_on_restrictions(
                func,
                finv,
                (call.pfirst, target),
                call.soldep,
                (self.tolapp3d, tol_guide),
                call.rec,
            )? {
                Some((p, sol)) => (p, vec_of(&sol)?),
                None => return Ok((false, [false; 4])),
            }
        } else {
            (call.pfirst, call.soldep.clone())
        };
        let mut run = WalkRun {
            walk,
            lin: crate::chfi_ds::shared(crate::brep_blend::BRepBlendLine::new()),
            sp_first,
            sp_last,
            target,
            reverse,
            tol_guide,
            faces,
        };
        if !self.walk_sections(
            &mut run,
            (func, finv),
            call,
            (out.first, out.last),
            new_first,
            par_sol,
            &offset,
        )? {
            return Ok((false, [false; 4]));
        }
        *lin = Some(run.lin.clone());
        let ends = self.process_ends(data, call, &mut run, (func, finv), out)?;
        *lin = Some(run.lin.clone());
        Ok((true, ends))
    }
}
