// Rust translation of ChFi3d_Builder::SplitKPart (ChFi3d_Builder_SpKP.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A particular case cut into the parts of its traces inside the faces, placed on the
//! spine.
use super::ChFi3dBuilder;
use super::spkp_fill::{SideHatch, fill_sd, set_coincident_domain_edge, tri};
use super::trunc::{abscissa_on_edge, adjust_param, cp_sd};
use crate::chfi_ds::{SpineHandle, State, SurfDataHandle, lock};
use crate::chfi3d::{cherche_element, edge_state};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;

/// The context of `SplitKPart`: the spine, the edge, the faces, the ends of the edge and
/// the hatchings of the two traces.
pub(super) struct Split<'a> {
    pub(super) spine: &'a SpineHandle,
    pub(super) iedge: i32,
    pub(super) f: [Face; 2],
    pub(super) bout: [Vertex; 2],
    pub(super) support: Edge,
    pub(super) sides: [SideHatch; 2],
    pub(super) ind: [Vec<i32>; 2],
    pub(super) pitol: f64,
}

impl ChFi3dBuilder {
    /// The surface data of each domain of the single hatched trace `k` (the other trace
    /// being missing or unbounded), and the ends of the contour checked on that face.
    pub(super) fn split_one_side(
        &mut self,
        sp: &mut Split<'_>,
        cd: &mut SurfDataHandle,
        set_data: &mut Vec<SurfDataHandle>,
        k: usize,
        (intf, intl): (&mut bool, &mut bool),
    ) -> Result<bool, Failure> {
        let ons = k as i32 + 1;
        let pitol = sp.pitol;
        let mut ind = Vec::new();
        if !tri(&mut sp.sides[k], &mut ind, (0., 0., pitol))? {
            return Ok(false);
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..sp.sides[k].nb as usize {
            let dom = sp.sides[k].domain(ind[i])?;
            let (pf, pl) = (
                dom.first_point()?.parameter(),
                dom.second_point()?.parameter(),
            );
            fill_sd(
                &self.ds,
                cd,
                &sp.sides[k].m,
                &dom,
                pf,
                true,
                ons,
                pitol,
                &sp.bout[0],
            )?;
            fill_sd(
                &self.ds,
                cd,
                &sp.sides[k].m,
                &dom,
                pl,
                false,
                ons,
                pitol,
                &sp.bout[1],
            )?;
            set_coincident_domain_edge(&mut self.ds, cd, ons, &sp.sides[k].m, &dom, true)?;
            set_data.push(cd.clone());
            *cd = cp_sd(&mut self.ds, cd)?;
        }
        let face = sp.f[k].clone();
        for (first, flag) in [(true, intf), (false, intl)] {
            if !*flag {
                continue;
            }
            let sd = if first {
                set_data.first()
            } else {
                set_data.last()
            }
            .cloned()
            .ok_or_else(|| Failure::out_of_range("NCollection_Sequence::ChangeValue"))?;
            let cp = lock(&sd).vertex(first, ons).clone();
            let status = lock(sp.spine).status(first);
            *flag = if cp.is_on_arc() && status == State::OnSame {
                let mut fbid = Face::default();
                !self.search_face(Some(sp.spine), &cp, &face, &mut fbid)?
            } else {
                false
            };
        }
        Ok(true)
    }

    /// The surface data of each pair of overlapping domains of the two traces; the
    /// indices of the domains of each one.
    #[allow(clippy::type_complexity)]
    pub(super) fn split_both_sides(
        &mut self,
        sp: &mut Split<'_>,
        cd: &mut SurfDataHandle,
        set_data: &mut Vec<SurfDataHandle>,
    ) -> Result<Option<(Vec<i32>, Vec<i32>)>, Failure> {
        let (pitol, wref, tol2d) = (sp.pitol, 0., self.tol2d);
        let (mut period1, mut period2) = (0., 0.);
        let (mut ind1, mut ind2) = (Vec::new(), Vec::new());
        if sp.sides[0].ll.is_periodic()? {
            if !tri(&mut sp.sides[1], &mut ind2, (wref, 0., pitol))? {
                return Ok(None);
            }
            period1 = sp.sides[0].ll.period()?;
            if !tri(&mut sp.sides[0], &mut ind1, (wref, period1, pitol))? {
                return Ok(None);
            }
        } else {
            if !tri(&mut sp.sides[0], &mut ind1, (wref, 0., pitol))? {
                return Ok(None);
            }
            if sp.sides[1].ll.is_periodic()? {
                period2 = sp.sides[1].ll.period()?;
            }
            if !tri(&mut sp.sides[1], &mut ind2, (wref, period2, pitol))? {
                return Ok(None);
            }
        }
        let (mut ion1, mut ion2) = (Vec::new(), Vec::new());
        for i in 1..=sp.sides[0].nb {
            let dom1 = sp.sides[0].domain(ind1[i as usize - 1])?;
            let (acheval1, mut f1, mut l1) = adjust_param(&dom1, wref, period1, pitol)?;
            for _ in 0..if acheval1 { 2 } else { 1 } {
                for j in 1..=sp.sides[1].nb {
                    let dom2 = sp.sides[1].domain(j)?;
                    let (acheval2, mut f2, mut l2) = adjust_param(&dom2, wref, period2, pitol)?;
                    for _ in 0..if acheval2 { 2 } else { 1 } {
                        if f2 <= l1 && f1 <= l2 {
                            let (a_first, a_last) = (f1.max(f2), l1.min(l2));
                            if f1 >= f2 - tol2d {
                                fill_sd(
                                    &self.ds,
                                    cd,
                                    &sp.sides[0].m,
                                    &dom1,
                                    f1,
                                    true,
                                    1,
                                    pitol,
                                    &sp.bout[0],
                                )?;
                            }
                            if f2 >= f1 - tol2d {
                                fill_sd(
                                    &self.ds,
                                    cd,
                                    &sp.sides[1].m,
                                    &dom2,
                                    f2,
                                    true,
                                    2,
                                    pitol,
                                    &sp.bout[0],
                                )?;
                            }
                            if l1 >= l2 - tol2d {
                                fill_sd(
                                    &self.ds,
                                    cd,
                                    &sp.sides[1].m,
                                    &dom2,
                                    l2,
                                    false,
                                    2,
                                    pitol,
                                    &sp.bout[1],
                                )?;
                            }
                            if l2 >= l1 - tol2d {
                                fill_sd(
                                    &self.ds,
                                    cd,
                                    &sp.sides[0].m,
                                    &dom1,
                                    l1,
                                    false,
                                    1,
                                    pitol,
                                    &sp.bout[1],
                                )?;
                            }
                            let whole1 =
                                (a_first - f1).abs() <= tol2d && (a_last - l1).abs() <= tol2d;
                            set_coincident_domain_edge(
                                &mut self.ds,
                                cd,
                                1,
                                &sp.sides[0].m,
                                &dom1,
                                whole1,
                            )?;
                            let whole2 =
                                (a_first - f2).abs() <= tol2d && (a_last - l2).abs() <= tol2d;
                            set_coincident_domain_edge(
                                &mut self.ds,
                                cd,
                                2,
                                &sp.sides[1].m,
                                &dom2,
                                whole2,
                            )?;
                            set_data.push(cd.clone());
                            *cd = cp_sd(&mut self.ds, cd)?;
                            ion1.push(i);
                            ion2.push(j);
                        }
                        f2 += period2;
                        l2 += period2;
                    }
                }
                f1 += period1;
                l1 += period1;
            }
        }
        sp.ind = [ind1, ind2];
        Ok(Some((ion1, ion2)))
    }

    /// The common point of a side at an end leaves the face (no tangent face across its
    /// arc).
    pub(super) fn leaves(
        &self,
        sp: &Split<'_>,
        sd: &SurfDataHandle,
        first: bool,
        ons: i32,
    ) -> Result<bool, Failure> {
        let cp = lock(sd).vertex(first, ons).clone();
        if !cp.is_on_arc() {
            return Ok(false);
        }
        let mut fbid = Face::default();
        Ok(!self.search_face(Some(sp.spine), &cp, &sp.f[ons as usize - 1], &mut fbid)?)
    }

    /// The surface data at an end of the contour nearest to the end of the edge kept as
    /// the first (last) one, the ones before (after) it dropped.
    pub(super) fn keep_nearest(
        &self,
        sp: &Split<'_>,
        set_data: &mut Vec<SurfDataHandle>,
        ions: (&mut Vec<i32>, &mut Vec<i32>),
        first: bool,
    ) -> Result<(), Failure> {
        let (f, l) = {
            let s = lock(sp.spine);
            (
                s.first_parameter_of(sp.iedge)?,
                s.last_parameter_of(sp.iedge)?,
            )
        };
        let (mut best, mut dist) = (0usize, f64::MAX);
        for (k, sd) in set_data.iter().enumerate() {
            for ons in [1, 2] {
                let cp = lock(sd).vertex(first, ons).clone();
                let mut fbid = Face::default();
                if cp.is_on_arc()
                    && !self.search_face(Some(sp.spine), &cp, &sp.f[ons as usize - 1], &mut fbid)?
                {
                    let ptg = lock(sd).interference(ons).parameter(first);
                    let a = abscissa_on_edge(sp.spine, sp.iedge, ptg)?;
                    let dsp = if first { a } else { -a - f + l };
                    if dsp.abs() < dist {
                        best = k + 1;
                        dist = dsp.abs();
                    }
                    break;
                }
            }
        }
        if first {
            if best > 1 {
                set_data.drain(0..best - 1);
                ions.0.drain(0..best - 1);
                ions.1.drain(0..best - 1);
            }
        } else if best < set_data.len() {
            set_data.truncate(best);
            ions.0.truncate(best);
            ions.1.truncate(best);
        }
        Ok(())
    }

    /// The end of the first (last) surface data on the face `ons` moved to the end of the
    /// domain of the other trace when only the face `ons` is left there; whether the end
    /// still leaves both faces.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn adjust_end_side(
        &self,
        sp: &Split<'_>,
        cd: &SurfDataHandle,
        first: bool,
        ons: i32,
        ion_other: i32,
        mut flag: bool,
    ) -> Result<bool, Failure> {
        let opp = 3 - ons;
        let sov = lock(cd).vertex(first, opp).clone();
        if !self.leaves(sp, cd, first, ons)? {
            return Ok(false);
        }
        let k = opp as usize - 1;
        let dom = sp.sides[k].domain(sp.ind[k][ion_other as usize - 1])?;
        let ponh = if first {
            dom.first_point()?.parameter()
        } else {
            dom.second_point()?.parameter()
        };
        fill_sd(
            &self.ds,
            cd,
            &sp.sides[k].m,
            &dom,
            ponh,
            first,
            opp,
            sp.pitol,
            &sp.bout[if first { 0 } else { 1 }],
        )?;
        let cp_other = lock(cd).vertex(first, opp).clone();
        let mut fbid = Face::default();
        if !cp_other.is_on_arc()
            || self.search_face(Some(sp.spine), &cp_other, &sp.f[k], &mut fbid)?
        {
            *lock(cd).change_vertex(first, opp) = sov;
            if lock(sp.spine).status(first) != State::OnSame {
                let mut g = lock(cd);
                let p = g.interference(ons).parameter(first);
                g.change_interference(opp).set_parameter(p, first);
                flag = false;
            }
        }
        Ok(flag)
    }

    /// The processing of an end of the contour after the split on both traces; the face
    /// whose trace places the surface data on the spine (`onS`) and the interference that
    /// limits its length (`cntlFiOnS`).
    pub(super) fn split_end(
        &self,
        sp: &Split<'_>,
        set_data: &mut Vec<SurfDataHandle>,
        ions: (&mut Vec<i32>, &mut Vec<i32>),
        first: bool,
        flag: &mut bool,
        ons_cntl: &mut (i32, i32),
    ) -> Result<bool, Failure> {
        self.keep_nearest(sp, set_data, (&mut *ions.0, &mut *ions.1), first)?;
        let Some(cd) = (if first {
            set_data.first()
        } else {
            set_data.last()
        })
        .cloned() else {
            return Ok(false);
        };
        let (on1, on2) = {
            let g = lock(&cd);
            (
                g.vertex(first, 1).is_on_arc(),
                g.vertex(first, 2).is_on_arc(),
            )
        };
        let pick = |v: &Vec<i32>| {
            if first {
                v.first().copied()
            } else {
                v.last().copied()
            }
        };
        if on1 && on2 {
            *flag = self.leaves(sp, &cd, first, 1)? && self.leaves(sp, &cd, first, 2)?;
        } else if on1 {
            let ion2 = pick(ions.1).ok_or_else(|| Failure::new("SplitKPart"))?;
            *flag = self.adjust_end_side(sp, &cd, first, 1, ion2, *flag)?;
        } else if on2 {
            let ion1 = pick(ions.0).ok_or_else(|| Failure::new("SplitKPart"))?;
            *flag = self.adjust_end_side(sp, &cd, first, 2, ion1, *flag)?;
        }
        if *flag && lock(sp.spine).status(first) == State::OnSame {
            let bout = &sp.bout[if first { 0 } else { 1 }];
            let mut three = [Edge::default(), Edge::default(), sp.support.clone()];
            let mut boutemp = Vertex::default();
            cherche_element(bout, &sp.support, &sp.f[0], &mut three[0], &mut boutemp)?;
            cherche_element(bout, &sp.support, &sp.f[1], &mut three[1], &mut boutemp)?;
            let ons = if edge_state(&three, &self.my_ef_map)? == State::OnSame {
                1
            } else {
                2
            };
            *ons_cntl = (ons, 3 - ons);
        }
        Ok(true)
    }

    /// The surface data whose trace on `ons` lies before the start (after the end) of the
    /// edge removed.
    pub(super) fn drop_outside(
        &self,
        sp: &Split<'_>,
        set_data: &mut Vec<SurfDataHandle>,
        ons: i32,
        at_start: bool,
    ) -> Result<(), Failure> {
        let (f, l) = {
            let s = lock(sp.spine);
            (
                s.first_parameter_of(sp.iedge)?,
                s.last_parameter_of(sp.iedge)?,
            )
        };
        let mut i = 0;
        while i < set_data.len() {
            let tg = if at_start {
                lock(&set_data[i]).interference(ons).last_parameter()
            } else {
                lock(&set_data[i]).interference(ons).first_parameter()
            };
            let nl = abscissa_on_edge(sp.spine, sp.iedge, tg)?;
            let out = if at_start {
                nl < -self.tolesp
            } else {
                nl > l - f + self.tolesp
            };
            if out {
                set_data.remove(i);
            } else {
                i += 1;
            }
        }
        Ok(())
    }
}
