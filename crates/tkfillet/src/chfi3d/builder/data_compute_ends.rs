// Rust translation of the end processing of ChFi3d_Builder::ComputeData(Data, HGuide,
// Spine, Lin, S1, I1, S2, I2, Func, FInv, ...) and the file statics SearchIndex and
// UpdateLine (ChFi3d_Builder_6.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ends of a walked fillet: their common points on the arcs of the faces, the
//! continuation across the arcs, the cut of the sections beyond them and the extensions.
use super::data_compute::{ComputeCall, WalkRun, is_obst};
use super::{ChFi3dBuilder, SurfOut};
use crate::blend::{BlendFuncInv, BlendFunction};
use crate::brep_blend::{BRepBlendExtremity, BRepBlendLine, BRepBlendLineHandle};
use crate::chfi_ds::{SurfDataHandle, lock};
use crate::chfi3d::fil_common_point;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// `SearchIndex(Value, Lin)`: the index of the first section at or after `value` (the
/// last one at most).
fn search_index(value: f64, lin: &BRepBlendLine) -> Result<i32, Failure> {
    let nb = lin.nb_points();
    let mut ind = 1;
    while ind < nb && lin.point(ind)?.parameter() < value {
        ind += 1;
    }
    Ok(ind)
}

/// The extremity at a section point on one surface (the 4-argument `SetValue`, which
/// is the curve overload: `V` lands in the guide parameter).
fn extremity_at(p: rustycad_tkmath::gp::Point3, u: f64, v: f64) -> BRepBlendExtremity {
    let mut be = BRepBlendExtremity::new();
    be.set_value_on_curve(p, u, v, precision::confusion());
    be
}

/// `UpdateLine(Line, isfirst)`: the extremities of the line before (after) its first
/// (last) section moved onto it.
fn update_line(line: &BRepBlendLineHandle, isfirst: bool) -> Result<(), Failure> {
    let mut l = lock(line);
    let idx = if isfirst { 1 } else { l.nb_points() };
    let bp = *l.point(idx)?;
    let tguide = bp.parameter();
    let (u1, v1) = bp.parameters_on_s1()?;
    let (u2, v2) = bp.parameters_on_s2()?;
    if isfirst {
        if l.start_point_on_first().parameter_on_guide() < tguide {
            let s2 = l.start_point_on_second().clone();
            l.set_start_points(extremity_at(bp.point_on_s1(), u1, v1), s2);
        }
        if l.start_point_on_second().parameter_on_guide() < tguide {
            let s1 = l.start_point_on_first().clone();
            l.set_start_points(s1, extremity_at(bp.point_on_s2(), u2, v2));
        }
    } else {
        if l.end_point_on_first().parameter_on_guide() > tguide {
            let s2 = l.end_point_on_second().clone();
            l.set_end_points(extremity_at(bp.point_on_s1(), u1, v1), s2);
        }
        if l.end_point_on_second().parameter_on_guide() > tguide {
            let s1 = l.end_point_on_first().clone();
            l.set_end_points(s1, extremity_at(bp.point_on_s2(), u2, v2));
        }
    }
    Ok(())
}

/// The state of one end of the fillet on the arcs of its two faces.
#[derive(Default)]
struct EndState {
    arc: [bool; 2],
    cas: [bool; 2],
    obst: [bool; 2],
    continue_failed: bool,
}

impl ChFi3dBuilder {
    /// The common point of an end of the line on the face `ons` (1 or 2) written into the
    /// surface data; true when no face continues the fillet across its arc.
    fn end_common_point(
        &self,
        data: &SurfDataHandle,
        run: &WalkRun,
        call: &ComputeCall<'_>,
        (first, ons): (bool, i32),
        bif: &mut Face,
    ) -> Result<bool, Failure> {
        {
            let l = lock(&run.lin);
            let (ext, trans) = match (first, ons) {
                (true, 1) => (l.start_point_on_first().clone(), l.transition_on_s1()?),
                (true, _) => (l.start_point_on_second().clone(), l.transition_on_s2()?),
                (false, 1) => (l.end_point_on_first().clone(), l.transition_on_s1()?),
                (false, _) => (l.end_point_on_second().clone(), l.transition_on_s2()?),
            };
            drop(l);
            fil_common_point(
                &ext,
                trans,
                first,
                lock(data).change_vertex(first, ons),
                self.tolapp3d,
            )?;
        }
        let cp = lock(data).vertex(first, ons).clone();
        let face = if ons == 1 { &run.faces.0 } else { &run.faces.1 };
        Ok(!self.search_face(call.spine, &cp, face, bif)?)
    }

    /// The number of points on restrictions of the extremities of an end.
    fn narcs(run: &WalkRun, first: bool) -> [i32; 2] {
        let l = lock(&run.lin);
        if first {
            [
                l.start_point_on_first().nb_point_on_rst(),
                l.start_point_on_second().nb_point_on_rst(),
            ]
        } else {
            [
                l.end_point_on_first().nb_point_on_rst(),
                l.end_point_on_second().nb_point_on_rst(),
            ]
        }
    }

    /// The common points of an end on the arcs reached, whether they are obstacles, and
    /// the walking continued across the arcs when it may go on.
    #[allow(clippy::too_many_arguments)]
    fn examine_end(
        &self,
        data: &SurfDataHandle,
        call: &ComputeCall<'_>,
        run: &mut WalkRun,
        (func, finv): (&mut dyn BlendFunction, &mut dyn BlendFuncInv),
        first: bool,
        noprol: bool,
        bif: &mut Face,
    ) -> Result<EndState, Failure> {
        let mut st = EndState::default();
        let narc = Self::narcs(run, first);
        #[allow(clippy::needless_range_loop)]
        for k in 0..2 {
            if narc[k] == 0 {
                continue;
            }
            let ons = k as i32 + 1;
            let cas = self.end_common_point(data, run, call, (first, ons), bif)?;
            st.arc[k] = true;
            if cas {
                st.cas[k] = true;
                if let Some(sp) = call.spine {
                    let cp = lock(data).vertex(first, ons).clone();
                    st.obst[k] = if first && lock(sp).is_periodic() {
                        true
                    } else {
                        let v = if first {
                            lock(sp).first_vertex()?
                        } else {
                            lock(sp).last_vertex()?
                        };
                        is_obst(&cp, &v, &self.my_ve_map)?
                    };
                }
            }
        }
        let mut oncontinue = !noprol && (narc[0] != 0 || narc[1] != 0);
        if st.obst[0]
            || st.obst[1]
            || (st.cas[0] && st.cas[1])
            || (!st.cas[0] && st.arc[0])
            || (!st.cas[1] && st.arc[1])
        {
            oncontinue = false;
        }
        if oncontinue {
            self.continue_end(data, call, run, (func, finv), first, bif, &mut st)?;
        }
        Ok(st)
    }

    /// The walking continued across the arcs of an end, the new common points examined.
    #[allow(clippy::too_many_arguments)]
    fn continue_end(
        &self,
        data: &SurfDataHandle,
        call: &ComputeCall<'_>,
        run: &mut WalkRun,
        (func, finv): (&mut dyn BlendFunction, &mut dyn BlendFuncInv),
        first: bool,
        bif: &mut Face,
        st: &mut EndState,
    ) -> Result<(), Failure> {
        run.walk.classification_on_s1(!st.arc[0]);
        run.walk.classification_on_s2(!st.arc[1]);
        run.walk.check_2d(true);
        run.walk.continu(func, finv, run.target)?;
        run.walk.classification_on_s1(true);
        run.walk.classification_on_s2(true);
        run.walk.check_2d(false);
        let narc = Self::narcs(run, first);
        #[allow(clippy::needless_range_loop)]
        for k in 0..2 {
            if st.arc[k] {
                continue;
            }
            let ons = k as i32 + 1;
            if narc[k] == 0 {
                let l = lock(&run.lin);
                let ext = match (first, k) {
                    (true, 0) => l.start_point_on_first(),
                    (true, _) => l.start_point_on_second(),
                    (false, 0) => l.end_point_on_first(),
                    (false, _) => l.end_point_on_second(),
                };
                st.continue_failed = if first {
                    ext.parameter_on_guide() > run.target
                } else {
                    ext.parameter_on_guide() < run.target
                };
            } else {
                let cas = self.end_common_point(data, run, call, (first, ons), bif)?;
                st.arc[k] = true;
                if cas {
                    st.cas[k] = true;
                }
            }
        }
        if st.continue_failed {
            let ons = if st.arc[0] { 1 } else { 2 };
            if lock(data).vertex(first, ons).is_on_arc() && bif.is_null() {
                st.continue_failed = false;
            }
        }
        Ok(())
    }

    /// The sections of the line beyond the exit of a single arc (or beyond a third of the
    /// line at a forced end) removed.
    fn cut_start(
        &self,
        data: &SurfDataHandle,
        run: &WalkRun,
        st: &EndState,
        (intf, unseulsuffitdeb): (i32, bool),
        out: &mut SurfOut,
    ) -> Result<(), Failure> {
        let first = out.first;
        if (st.arc[0] ^ st.arc[1]) && !unseulsuffitdeb && first != run.sp_first {
            let sortie = lock(data)
                .vertex(true, if st.arc[0] { 1 } else { 2 })
                .parameter();
            if sortie - first > self.tolesp {
                self.remove_before(run, sortie, None)?;
                out.first = lock(&run.lin).point(1)?.parameter();
            }
        } else if intf >= 5 && !st.arc[0] && !st.arc[1] && first != run.sp_first {
            let sortie = (2. * first + out.last) / 3.;
            if sortie - first > self.tolesp {
                let nbpnt = lock(&run.lin).nb_points();
                self.remove_before(run, sortie, Some(nbpnt))?;
                out.first = lock(&run.lin).point(1)?.parameter();
            }
        }
        Ok(())
    }

    /// The sections before `sortie` removed (keeping three at least with `keep`).
    fn remove_before(&self, run: &WalkRun, sortie: f64, keep: Option<i32>) -> Result<(), Failure> {
        let mut ind = {
            let l = lock(&run.lin);
            let mut ind = search_index(sortie, &l)?;
            if l.point(ind)?.parameter() == sortie {
                ind -= 1;
            }
            ind
        };
        if let Some(nbpnt) = keep
            && nbpnt - ind < 3
        {
            ind = nbpnt - 3;
        }
        if ind >= 1 {
            lock(&run.lin).remove(1, ind)?;
            update_line(&run.lin, true)?;
        }
        Ok(())
    }

    /// The sections after `sortie` removed (keeping three at least with `keep`).
    fn remove_after(&self, run: &WalkRun, sortie: f64, keep: bool) -> Result<(), Failure> {
        let (mut ind, nbpnt) = {
            let l = lock(&run.lin);
            let mut ind = search_index(sortie, &l)?;
            if l.point(ind)?.parameter() == sortie {
                ind += 1;
            }
            (ind, l.nb_points())
        };
        if keep && ind < 3 {
            ind = 3;
        }
        if ind <= nbpnt {
            lock(&run.lin).remove(ind, nbpnt)?;
            update_line(&run.lin, false)?;
        }
        Ok(())
    }

    /// The sections beyond the exit at the last end removed.
    fn cut_end(
        &self,
        data: &SurfDataHandle,
        run: &WalkRun,
        st: &EndState,
        (intl, unseulsuffitfin): (i32, bool),
        out: &mut SurfOut,
    ) -> Result<(), Failure> {
        let last = out.last;
        if (st.arc[0] ^ st.arc[1]) && !unseulsuffitfin && last != run.sp_last {
            let sortie = lock(data)
                .vertex(false, if st.arc[0] { 1 } else { 2 })
                .parameter();
            if last - sortie > self.tolesp {
                self.remove_after(run, sortie, false)?;
                let l = lock(&run.lin);
                out.last = l.point(l.nb_points())?.parameter();
            }
        } else if intl >= 5 && !st.arc[0] && !st.arc[1] && last != run.sp_last {
            let sortie = (out.first + 2. * last) / 3.;
            if last - sortie > self.tolesp {
                self.remove_after(run, sortie, true)?;
                let l = lock(&run.lin);
                out.last = l.point(l.nb_points())?.parameter();
            }
        }
        Ok(())
    }

    /// The flag of an end after its processing: kept when both sides (or one, when one
    /// suffices) leave the faces.
    fn end_flag(int: i32, unseulsuffit: bool, g: (bool, bool), st: &EndState) -> i32 {
        if int != 0 && !unseulsuffit {
            return i32::from((g.0 && g.1) || st.continue_failed);
        }
        if int != 0 && unseulsuffit && int < 5 {
            let mut r = i32::from(g.0 || g.1);
            if r != 0 && ((!st.cas[0] && st.arc[0]) || (!st.cas[1] && st.arc[1])) {
                r = 0;
            }
            return r;
        }
        if int < 5 {
            return 0;
        }
        int
    }

    /// The ends of the walked fillet: `[Gd1, Gd2, Gf1, Gf2]`, the flags of its ends and its
    /// range updated in `out`, its extensions in the surface data.
    pub(super) fn process_ends(
        &self,
        data: &SurfDataHandle,
        call: &ComputeCall<'_>,
        run: &mut WalkRun,
        (func, finv): (&mut dyn BlendFunction, &mut dyn BlendFuncInv),
        out: &mut SurfOut,
    ) -> Result<[bool; 4], Failure> {
        if run.walk.twist_on_s1() {
            lock(data).set_twist_on_s1(true);
        }
        if run.walk.twist_on_s2() {
            lock(data).set_twist_on_s2(true);
        }
        let (unseulsuffitdeb, unseulsuffitfin) = (out.intf >= 2, out.intl >= 2);
        let (noproldeb, noprolfin) = (out.intf >= 3, out.intl >= 3);
        let rab = 0.03 * (run.sp_last - run.sp_first);
        let mut bif = Face::default();
        let mut deb = EndState::default();
        if run.reverse && out.intf != 0 {
            deb = self.examine_end(data, call, run, (func, finv), true, noproldeb, &mut bif)?;
        }
        let mut fin = EndState::default();
        if call.walk.forward && out.intl != 0 {
            run.target = run.sp_last;
            fin = self.examine_end(data, call, run, (func, finv), false, noprolfin, &mut bif)?;
        }
        {
            let l = lock(&run.lin);
            out.first = l.point(1)?.parameter();
            out.last = l.point(l.nb_points())?.parameter();
        }
        let mut g = [false; 4];
        if run.reverse && out.intf != 0 {
            g[0] = deb.cas[0];
            g[1] = deb.cas[1];
            self.cut_start(data, run, &deb, (out.intf, unseulsuffitdeb), out)?;
            if g[0] && g[1] {
                let p1 = lock(&run.lin).point(1)?.parameter();
                run.target = (p1 - rab).min(out.first).max(run.sp_first);
                lock(data).set_first_extension_value((p1 - run.target).abs());
            }
            out.intf = Self::end_flag(out.intf, unseulsuffitdeb, (g[0], g[1]), &deb);
        }
        if call.walk.forward && out.intl != 0 {
            g[2] = fin.cas[0];
            g[3] = fin.cas[1];
            self.cut_end(data, run, &fin, (out.intl, unseulsuffitfin), out)?;
            if g[2] && g[3] {
                let pl = {
                    let l = lock(&run.lin);
                    l.point(l.nb_points())?.parameter()
                };
                run.target = (pl + rab).max(out.last).min(run.sp_last);
                lock(data).set_last_extension_value((run.target - pl).abs());
            }
            out.intl = Self::end_flag(out.intl, unseulsuffitfin, (g[2], g[3]), &fin);
        }
        Ok(g)
    }
}
