// Rust translation of ChFi3d_Builder::SplitKPart (ChFi3d_Builder_SpKP.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pieces of a split special-case fillet placed on the spine, and the driver `SplitKPart`.
use super::ChFi3dBuilder;
use super::data_compute::face_of;
use super::spkp::Split;
use super::spkp_fill::SideHatch;
use super::trunc::{param_on_spine, reset_prol};
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::{SpineHandle, SurfDataHandle, lock};
use rustycad_tkbrep::topexp;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, State as TopState};

impl ChFi3dBuilder {
    /// The spine parameters of the surface data, the parameters of the spine at the ends
    /// of the contour and the surface data cut at the ends of the edge otherwise.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn place_on_spine(
        &self,
        sp: &Split<'_>,
        set_data: &mut Vec<SurfDataHandle>,
        (intf, intl): (bool, bool),
        (ons, cntl): (i32, i32),
        faces: (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
    ) -> Result<(), Failure> {
        let tolesp = self.tolesp;
        for cd in set_data.iter() {
            let (ftg, ltg) = {
                let g = lock(cd);
                (
                    g.interference(ons).first_parameter(),
                    g.interference(ons).last_parameter(),
                )
            };
            let (mut fsp, pokdeb) =
                param_on_spine(&self.ds, ftg, cd, sp.spine, sp.iedge, (intf, intl), tolesp)?;
            if !pokdeb {
                fsp = reset_prol(&self.ds, cd, sp.spine, sp.iedge, true)?;
            }
            let (mut lsp, pokfin) =
                param_on_spine(&self.ds, ltg, cd, sp.spine, sp.iedge, (intf, intl), tolesp)?;
            if !pokfin {
                lsp = reset_prol(&self.ds, cd, sp.spine, sp.iedge, false)?;
            }
            let (periodic, nb) = {
                let s = lock(sp.spine);
                (s.is_periodic(), s.nb_edges())
            };
            if periodic && sp.iedge == nb && lsp < fsp {
                lsp += lock(sp.spine).period()?;
            } else if periodic && sp.iedge == 1 && lsp < fsp {
                fsp -= lock(sp.spine).period()?;
            }
            let mut g = lock(cd);
            g.set_first_spine_param(fsp);
            g.set_last_spine_param(lsp);
        }
        let (fe, le) = {
            let s = lock(sp.spine);
            (
                s.first_parameter_of(sp.iedge)?,
                s.last_parameter_of(sp.iedge)?,
            )
        };
        if intf && !set_data.is_empty() {
            let p = lock(&set_data[0]).first_spine_param();
            lock(sp.spine).set_first_parameter(p);
        } else {
            let mut i = set_data.len();
            for (k, cd) in set_data.iter().enumerate() {
                let (fsp, lsp) = {
                    let g = lock(cd);
                    (g.first_spine_param(), g.last_spine_param())
                };
                if lsp > fe {
                    if fsp <= fe {
                        self.trunc(cd, sp.spine, faces, sp.iedge, true, cntl)?;
                    }
                    i = k;
                    break;
                }
            }
            set_data.drain(0..i.min(set_data.len()));
        }
        if intl && !set_data.is_empty() {
            let p = lock(&set_data[set_data.len() - 1]).last_spine_param();
            lock(sp.spine).set_last_parameter(p);
        } else {
            let mut keep = 0;
            for k in (0..set_data.len()).rev() {
                let (fsp, lsp) = {
                    let g = lock(&set_data[k]);
                    (g.first_spine_param(), g.last_spine_param())
                };
                if fsp < le {
                    if lsp >= le {
                        self.trunc(&set_data[k], sp.spine, faces, sp.iedge, false, cntl)?;
                    }
                    keep = k + 1;
                    break;
                }
            }
            set_data.truncate(keep);
        }
        Ok(())
    }

    /// `SplitKPart(Data, SetData, Spine, Iedge, S1, I1, S2, I2, intf, intl)`: the particular
    /// case of the edge cut by the faces into `set_data`, placed on the spine; false when
    /// its traces cannot be hatched or nothing is left.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn split_kpart(
        &mut self,
        data: &SurfDataHandle,
        set_data: &mut Vec<SurfDataHandle>,
        spine: &SpineHandle,
        iedge: i32,
        (s1, i1): (&Handle<dyn Adaptor3dSurface>, &WalkingDomain),
        (s2, i2): (&Handle<dyn Adaptor3dSurface>, &WalkingDomain),
        (intf, intl): (&mut bool, &mut bool),
    ) -> Result<bool, Failure> {
        let pitol = precision::p_intersection();
        let tols = (self.tol2d, self.tolapp3d);
        let (c1, c2) = {
            let g = lock(data);
            (
                g.interference(1).pcurve_on_face().clone(),
                g.interference(2).pcurve_on_face().clone(),
            )
        };
        let Some(side1) = SideHatch::build(&c1, i1, tols)? else {
            return Ok(false);
        };
        let Some(side2) = SideHatch::build(&c2, i2, tols)? else {
            return Ok(false);
        };
        let support = lock(spine).current_elementary_spine(iedge)?.edge();
        let (mut bout1, mut bout2) = topexp::vertices(&support, false);
        if support.orientation() == Orientation::Reversed {
            std::mem::swap(&mut bout1, &mut bout2);
        }
        let f = [face_of(s1), face_of(s2)];
        let mut cd = data.clone();
        {
            let (a, b) = (
                self.ds.add_shape(f[0].as_ref()),
                self.ds.add_shape(f[1].as_ref()),
            );
            let mut g = lock(&cd);
            g.change_index_of_s1(a);
            g.change_index_of_s2(b);
        }
        let mut sp = Split {
            spine,
            iedge,
            f,
            bout: [bout1, bout2],
            support,
            sides: [side1, side2],
            ind: [Vec::new(), Vec::new()],
            pitol,
        };
        let mut ons_cntl = (1, 0);
        if c1.is_null() && c2.is_null() {
            return Ok(false);
        }
        let unbounded = |s: &SideHatch| -> Result<bool, Failure> {
            Ok(s.nb == 1 && !s.domain(1)?.has_first_point())
        };
        if c1.is_null() || unbounded(&sp.sides[0])? {
            if c1.is_null()
                && i1.classify(lock(&cd).get_2d_point(false, 1), 1.0e-8, false)? == TopState::Out
            {
                return Ok(false);
            }
            if !self.split_one_side(&mut sp, &mut cd, set_data, 1, (&mut *intf, &mut *intl))? {
                return Ok(false);
            }
        } else if c2.is_null() || unbounded(&sp.sides[1])? {
            if c2.is_null()
                && i2.classify(lock(&cd).get_2d_point(false, 2), 1.0e-8, false)? == TopState::Out
            {
                return Ok(false);
            }
            if !self.split_one_side(&mut sp, &mut cd, set_data, 0, (&mut *intf, &mut *intl))? {
                return Ok(false);
            }
        } else {
            let Some((mut ion1, mut ion2)) = self.split_both_sides(&mut sp, &mut cd, set_data)?
            else {
                return Ok(false);
            };
            if *intf
                && !self.split_end(
                    &sp,
                    set_data,
                    (&mut ion1, &mut ion2),
                    true,
                    intf,
                    &mut ons_cntl,
                )?
            {
                return Ok(false);
            }
            if *intl
                && !self.split_end(
                    &sp,
                    set_data,
                    (&mut ion1, &mut ion2),
                    false,
                    intl,
                    &mut ons_cntl,
                )?
            {
                return Ok(false);
            }
        }
        if !*intf {
            self.drop_outside(&sp, set_data, ons_cntl.0, true)?;
        }
        if !*intl {
            self.drop_outside(&sp, set_data, ons_cntl.0, false)?;
        }
        self.place_on_spine(&sp, set_data, (*intf, *intl), ons_cntl, (s1, s2))?;
        Ok(true)
    }
}
