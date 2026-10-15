// Rust translation of ChFi3d_Builder::PerformSetOfKGen, ChFi3d_Builder::PerformSetOfSurf
// and the file statics ChangeTransition and ChFi3d_BoxDiag (ChFi3d_Builder_2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillets of a contour: walked along its elementary spines, the twisted ones
//! rebuilt, the common points of consecutive ones merged, and its ends closed.
use super::ChFi3dBuilderOps;
use super::compute::spine_of;
use super::ends::make_extremities;
use super::faces_search::curve_on_surface;
use crate::chfi_ds::{CommonPoint, ElSpine, SpineHandle, StripeHandle, SurfDataHandle, lock};
use crate::chfi3d::{enlarge_box_ds, enlarge_box_edge, solid_index};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Edge};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs;
use rustycad_tkgeombase::extrema::LocateExtPC;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elclib;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;
use std::rc::Rc;

/// `ChFi3d_BoxDiag(box)`: the diagonal of the box.
fn box_diag(b: &BoundingBox3) -> Result<f64, Failure> {
    let [a, b2, c, d, e, f] = b.get()?;
    Ok(((d - a).powi(2) + (e - b2).powi(2) + (f - c).powi(2)).sqrt())
}

/// `ChangeTransition(Precedant, Courant, FaceIndex, DS)`: the arc of the previous common
/// point given to the current one with the reversed transition, unless the arc is a seam
/// of the face whose tangent is not along the previous vector.
fn change_transition(
    precedant: &CommonPoint,
    courant: &mut CommonPoint,
    face_index: i32,
    ds: &TopOpeBRepDSDataStructure,
) -> Result<(), Failure> {
    let f = topods::face(ds.shape(face_index, true))?;
    let arc: Edge = precedant.arc()?.clone();
    let pcurve1 = curve_on_surface(&arc, &f)?;
    let mut rev = arc.clone();
    rev.reverse();
    let pcurve2 = curve_on_surface(&rev, &f)?;
    let mut tochange = true;
    if pcurve1 != pcurve2 {
        let (_, tgarc) = BRepAdaptorCurve::with_edge(&arc)?.d1(precedant.parameter_on_arc()?)?;
        tochange = tgarc.is_parallel(precedant.vector()?, precision::confusion())?;
    }
    if tochange {
        courant.set_arc(
            precision::confusion(),
            &arc,
            precedant.parameter_on_arc()?,
            top_abs::reverse(precedant.transition_on_arc()?),
        );
    }
    Ok(())
}

/// The face (or the obstacle curve) of a surface data on the side `ons`.
fn support(sd: &crate::chfi_ds::SurfData, ons: i32) -> Result<i32, Failure> {
    if sd.is_on_curve(ons) {
        return sd.index_of_c(ons);
    }
    Ok(sd.index(ons))
}

impl dyn ChFi3dBuilderOps + '_ {
    /// The common points of two consecutive surface data on the side `ons` merged: the
    /// one on an arc given to the other, and the tolerance of both made the largest.
    fn merge_side(
        &self,
        cursd: &SurfDataHandle,
        nextsd: &SurfDataHandle,
        ons: i32,
    ) -> Result<f64, Failure> {
        let ds = &self.base().ds;
        let (mut curp, curs, cur_on_curve) = {
            let g = lock(cursd);
            (
                g.vertex(false, ons).clone(),
                support(&g, ons)?,
                g.is_on_curve(ons),
            )
        };
        let (mut nextp, nexts, next_on_curve) = {
            let g = lock(nextsd);
            (
                g.vertex(true, ons).clone(),
                support(&g, ons)?,
                g.is_on_curve(ons),
            )
        };
        let tol = curp.tolerance().max(nextp.tolerance());
        if !curp.is_on_arc() && nextp.is_on_arc() {
            curp = nextp.clone();
            if curs == nexts && !next_on_curve {
                change_transition(&nextp, &mut curp, nexts, ds)?;
            }
        } else if curp.is_on_arc() && !nextp.is_on_arc() {
            nextp = curp.clone();
            if curs == nexts && !cur_on_curve {
                change_transition(&curp, &mut nextp, curs, ds)?;
            }
        }
        *lock(cursd).change_vertex(false, ons) = curp;
        *lock(nextsd).change_vertex(true, ons) = nextp;
        Ok(tol)
    }

    /// The common points between consecutive surface data merged with the tolerance of
    /// their boxes.
    fn merge_consecutive(
        &self,
        cursd: &SurfDataHandle,
        nextsd: &SurfDataHandle,
    ) -> Result<(), Failure> {
        let tol1 = self.merge_side(cursd, nextsd, 1)?;
        let tol2 = self.merge_side(cursd, nextsd, 2)?;
        let set_tols = |t1: f64, t2: f64| {
            for (k, t) in [(1, t1), (2, t2)] {
                lock(cursd).change_vertex(false, k).set_tolerance(t);
                lock(nextsd).change_vertex(true, k).set_tolerance(t);
            }
        };
        set_tols(tol1, tol2);
        let (mut b1, mut b2) = (BoundingBox3::default(), BoundingBox3::default());
        let (curp1, curp2) = {
            let g = lock(cursd);
            (g.vertex(false, 1).clone(), g.vertex(false, 2).clone())
        };
        let ef = &self.base().my_ef_map;
        for (cp, b) in [(&curp1, &mut b1), (&curp2, &mut b2)] {
            if cp.is_on_arc() {
                enlarge_box_edge(
                    cp.arc()?,
                    ef.find_from_key(cp.arc()?.as_ref())?,
                    cp.parameter_on_arc()?,
                    b,
                )?;
            }
        }
        let ds = &self.base().ds;
        enlarge_box_ds(ds, None, cursd, (&mut b1, &mut b2), false)?;
        enlarge_box_ds(ds, None, nextsd, (&mut b1, &mut b2), true)?;
        set_tols(box_diag(&b1)?, box_diag(&b2)?);
        Ok(())
    }

    /// The surface data of the contour bound to the edge `ej` in the map of the generated
    /// faces.
    fn bind_generated(&mut self, ej: &Edge, sd: &SurfDataHandle) {
        let surf = lock(sd).surf();
        let key = ej.clone().into();
        if !self.base().my_evi_map.is_bound(&key) {
            self.base_mut().my_evi_map.bind(key.clone(), Vec::new());
        }
        if let Ok(l) = self.base_mut().my_evi_map.change_find(&key) {
            l.push(surf);
        }
    }

    /// The parameters on the elementary spine of the vertices between the edges `if_`
    /// to `il`.
    fn vertex_parameters(
        spine: &SpineHandle,
        curhels: &Handle<ElSpine>,
        (if_, il): (i32, i32),
        (periodic, wf, period, nbed): (bool, f64, f64, i32),
    ) -> Result<Vec<f64>, Failure> {
        let mut wv = Vec::new();
        for i in if_..il {
            let iloc = if periodic { (i - 1) % nbed + 1 } else { i };
            let mut wi = lock(spine).last_parameter_of(iloc)?;
            if periodic {
                wi = elclib::in_period(wi, wf, wf + period);
            }
            let pv = lock(spine).value(wi)?;
            let curve: &dyn Adaptor3dCurve = &**curhels;
            let ext = LocateExtPC::with_curve(pv, curve, wi, 1.0e-8)?;
            wv.push(if ext.is_done() {
                ext.point()?.parameter()
            } else {
                wi
            });
        }
        Ok(wv)
    }

    /// The faces generated by the edges of an elementary spine: its surface data bound to
    /// the edges they lie along.
    fn bind_el_spine(
        &mut self,
        spine: &SpineHandle,
        curhels: &Handle<ElSpine>,
        seq: &[SurfDataHandle],
        periodic: bool,
    ) -> Result<(), Failure> {
        let tolesp = self.base().tolesp;
        let (wf, wl) = (curhels.first_parameter()?, curhels.last_parameter()?);
        let nbed = lock(spine).nb_edges();
        let (period, if_, il) = if periodic {
            let period = lock(spine).period()?;
            let nwf = elclib::in_period(wf, -tolesp, period - tolesp);
            let nwl = elclib::in_period(wl, tolesp, period + tolesp);
            let sp = lock(spine);
            let mut il = sp.index_at(nwl, false)?;
            if nwl < nwf + tolesp {
                il += nbed;
            }
            (period, sp.index_at(nwf, true)?, il)
        } else {
            let sp = lock(spine);
            (0., sp.index_at(wf, true)?, sp.index_at(wl, false)?)
        };
        let in_range = |sd: &SurfDataHandle| {
            let g = lock(sd);
            !(g.last_spine_param() < wf + tolesp || g.first_spine_param() > wl - tolesp)
        };
        if if_ == il {
            let ifloc = if periodic { (if_ - 1) % nbed + 1 } else { if_ };
            let ej = lock(spine).edges(ifloc)?.clone();
            for sd in seq.iter().filter(|sd| in_range(sd)) {
                self.bind_generated(&ej, sd);
            }
        } else if if_ < il {
            let wv =
                Self::vertex_parameters(spine, curhels, (if_, il), (periodic, wf, period, nbed))?;
            for sd in seq.iter().filter(|sd| in_range(sd)) {
                let (fp, lp) = {
                    let g = lock(sd);
                    (g.first_spine_param(), g.last_spine_param())
                };
                let (mut jf, mut jl) = (0, 0);
                for j in if_..il {
                    jf = j;
                    if fp < wv[(j - if_) as usize] - tolesp {
                        break;
                    }
                }
                for j in if_..il {
                    jl = j;
                    if lp < wv[(j - if_) as usize] + tolesp {
                        break;
                    }
                }
                for j in jf..=jl {
                    let jloc = if periodic { (j - 1) % nbed + 1 } else { j };
                    let ej = lock(spine).edges(jloc)?.clone();
                    self.bind_generated(&ej, sd);
                }
            }
        }
        Ok(())
    }

    /// `PerformSetOfKGen(Stripe, Simul)`: the fillets walked along the elementary spines;
    /// without simulation the twisted ones rebuilt, the common points merged and the
    /// generated faces mapped.
    pub(crate) fn perform_set_of_kgen(
        &mut self,
        stripe: &StripeHandle,
        simul: bool,
    ) -> Result<(), Failure> {
        let its = (
            Rc::new(BRepTopAdaptorTopolTool::new()),
            Rc::new(BRepTopAdaptorTopolTool::new()),
        );
        let spine = spine_of(stripe)?;
        let ll = lock(&spine).change_el_spines().clone();
        for es in &ll {
            self.perform_set_of_surf_on_el_spine(es, stripe, (&its.0, &its.1), simul)?;
        }
        if simul {
            return Ok(());
        }
        let hdata = lock(stripe)
            .set_of_surf_data()
            .cloned()
            .ok_or_else(|| Failure::new("PerformSetOfKGen"))?;
        let seq: Vec<SurfDataHandle> = lock(&hdata).clone();
        let len = seq.len();
        let periodic = lock(&spine).is_periodic();
        for i in 0..len {
            self.rebuild_twisted(&spine, &seq, i, periodic)?;
        }
        let last = if periodic { len + 1 } else { len };
        for i in 1..last {
            let j = i % len + 1;
            self.merge_consecutive(&seq[i - 1], &seq[j - 1])?;
        }
        for curhels in &ll {
            self.bind_el_spine(&spine, curhels, &seq, periodic)?;
        }
        Ok(())
    }

    /// `PerformSetOfSurf(Stripe, Simul)`: the fillets of the contour (its particular cases
    /// first unless split already), and without simulation its ends.
    pub(crate) fn perform_set_of_surf(
        &mut self,
        stripe: &StripeHandle,
        simul: bool,
    ) -> Result<(), Failure> {
        let sp = spine_of(stripe)?;
        let si = {
            let b = self.base_mut();
            solid_index(Some(&sp), &mut b.ds, &b.my_eso_map, &b.my_esh_map)?
        };
        lock(stripe).set_solid_index(si);
        if !lock(&sp).split_done() {
            self.perform_set_of_kpart(stripe, simul)?;
        }
        self.perform_set_of_kgen(stripe, simul)?;
        if !simul {
            let b = self.base_mut();
            make_extremities(stripe, &mut b.ds, &b.my_ef_map, (b.tolapp3d, b.tol2d))?;
        }
        Ok(())
    }
}
