//! The cases yap1 and yap2 of GFillONPartsWES2: the face is of the same domain as the
//! face connected to the edge in the other face, the edge not bounding it.
use super::context::{FcxContext, OnContext};
use super::{TopOpeBRepBuildBuilderON, fun_build_tb};
use crate::top_ope_brep_build::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_ds::export::{fds_data, fun_ds_sdm};
use crate::top_ope_brep_ds::{
    TopOpeBRepDSConfig, TopOpeBRepDSKind, TopOpeBRepDSShapeShapeInterference,
    TopOpeBRepDSTransition,
};
use crate::top_ope_brep_tool::{
    fsc_get_psc, fsc_state_eonface, fun_tool_getdxx, fun_tool_getxx, fun_tool_getxx_ng,
    fun_tool_nggeom_f, fun_tool_orient_e_in_f_forward, fun_tool_par_e_tol, fun_tool_par_f_tol,
};
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Direction3, Point2, Vector3};

/// The 3d and 2d keep flags of an edge oriented `o` in the connected face by the
/// transition on the state of the face and the transition of the orientation.
fn b3d_b2d(o: Orientation, before: State, after: State, sta: State) -> (bool, bool) {
    let t = TopOpeBRepDSTransition::from_orientation(o);
    match o {
        Orientation::Forward => (after == sta, t.after() == State::In),
        Orientation::Reversed => (before == sta, t.before() == State::In),
        _ => (false, false),
    }
}

/// The orientation of the edge in the face, by the reference of the same domain.
pub(super) fn orientation_by_reference(
    c: &OnContext,
    cx: &FcxContext,
    pb: &TopOpeBRepBuildBuilder,
) -> Result<Orientation, Failure> {
    let mut neworie = c.tfe.orientation(c.tb1, ShapeEnum::Face);
    let esd_orientation = |o: &mut Orientation| -> Result<(), Failure> {
        let esd = topods::edge(&pb.hds()?.shape(c.iesd, true))?;
        *o = fun_tool_orient_e_in_f_forward(&esd, &c.for_)?.1;
        if matches!(*o, Orientation::Forward | Orientation::Reversed) && !c.eon_so_esd {
            *o = top_abs::complement(*o);
        }
        Ok(())
    };
    let sta_fcx = cx.sta_fs;
    if cx.fcx_isref && !c.eg_bound_for {
        neworie = fun_tool_orient_e_in_f_forward(&c.eg, &cx.fcx)?.1;
        neworie = TopOpeBRepBuildBuilder::orient(
            neworie,
            TopOpeBRepBuildBuilder::reverse(sta_fcx, cx.sta_for),
        );
    } else if c.for_isref && !c.eg_bound_for {
        esd_orientation(&mut neworie)?;
        neworie = TopOpeBRepBuildBuilder::orient(
            neworie,
            TopOpeBRepBuildBuilder::reverse(cx.sta_for, sta_fcx),
        );
    } else if !c.eg_bound_for {
        let rankref_for = pb.hds()?.ds().ancestor_rank_index(c.iref_for);
        if rankref_for == cx.rank_fcx {
            neworie = fun_tool_orient_e_in_f_forward(&c.eg, &cx.fcx)?.1;
        } else if rankref_for == c.rank_for {
            esd_orientation(&mut neworie)?;
        }
        neworie = TopOpeBRepBuildBuilder::orient(
            neworie,
            TopOpeBRepBuildBuilder::reverse(cx.sta_for, sta_fcx),
        );
    }
    Ok(neworie)
}

impl TopOpeBRepBuildBuilderON<'_> {
    /// The shape/shape EDGE/FACE interference of the connected face on the 3d same domain
    /// edge, with its transition.
    fn e3_interference(
        &self,
        cx: &FcxContext,
        ie3: i32,
    ) -> Result<Option<TopOpeBRepDSTransition>, Failure> {
        let bds = self.pb()?.hds()?.ds();
        for i in bds.shape_interferences(cx.fcx.as_ref(), true).iter() {
            let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.clone().cast();
            let Some(s) = ssi.get() else {
                continue;
            };
            let (gt, g, st, _) = fds_data(i);
            if gt == TopOpeBRepDSKind::Edge && st == TopOpeBRepDSKind::Face && g == ie3 {
                return Ok(Some(s.transition()));
            }
        }
        Ok(None)
    }

    /// The case yap1: the edge with a 3d same domain edge on the split: kept by the
    /// transitions of the edge and of its 3d same domain edge, IN the face, oriented by
    /// the reference of the same domain.
    pub(super) fn yap1(&mut self, c: &OnContext, cx: &FcxContext) -> Result<(), Failure> {
        let pb = self.pb()?;
        let e3 = c.le3.first().cloned().unwrap_or_default();
        let ie3 = pb.hds()?.shape_index(&e3, true);
        let tfe_e3 = self.e3_interference(cx, ie3)?;
        let ssif = tfe_e3.is_some();
        let tfe_e3 = tfe_e3.unwrap_or_default();
        let (tfe_e3bef, tfe_e3aft) = (tfe_e3.before(), TopOpeBRepDSTransition::default().after());
        let mut oe3_forf = Orientation::Forward;
        if ssif {
            oe3_forf = fun_tool_orient_e_in_f_forward(&topods::edge(&e3)?, &c.for_)?.1;
        }
        let mut oeg = cx.oeg_fcxf;
        let (mut b3d, mut b2d, mut b3de3, mut b2de3) = (false, false, false, false);
        for (case, flip) in [
            (cx.ff_in_sdso, c.ofor != cx.ofcx),
            (cx.ff_in_sddo, c.ofor == cx.ofcx),
        ] {
            if !case {
                continue;
            }
            if flip {
                oeg = top_abs::complement(oeg);
                oe3_forf = top_abs::complement(oe3_forf);
            }
            (b3d, b2d) = b3d_b2d(oeg, c.tfebef, c.tfeaft, cx.sta_fs);
            if ssif {
                (b3de3, b2de3) = b3d_b2d(oe3_forf, tfe_e3bef, tfe_e3aft, cx.sta_for);
            }
        }
        if !(b3d && b2d && b3de3 && b2de3) {
            return Ok(());
        }
        let state2d = fsc_get_psc(|psc| fsc_state_eonface(&c.esp_on, 0.345, c.for_.as_ref(), psc))?;
        if state2d != State::In {
            return Ok(());
        }
        let neworie = orientation_by_reference(c, cx, pb)?;
        self.add_start(&c.esp_on.oriented(neworie))
    }

    /// The material direction of the other face at the edge (moved inside the face when
    /// orthogonal to the face normal) dotted with the normal of the face, with the
    /// geometric normal of the other face; None when a parameter fails.
    fn yap2_side(&self, c: &OnContext, par_eg: f64) -> Result<Option<(f64, Vector3)>, Failure> {
        let mut uv_fs = Point2::default();
        if !fun_tool_par_f_tol(&c.eg, par_eg, &c.fs, &mut uv_fs, c.tol_fs)? {
            return Ok(None);
        }
        let ng_fs = fun_tool_nggeom_f(uv_fs, &c.fs)?;
        let Some(mut xx_fs) =
            fun_tool_getxx_ng(&c.fs, &c.eg, par_eg, Direction3::from_vector(ng_fs)?)?
        else {
            return Ok(None);
        };
        let mut uv_for = Point2::default();
        if !fun_tool_par_f_tol(&c.eg, par_eg, &c.for_, &mut uv_for, c.tol_for)? {
            return Ok(None);
        }
        let mut nt_for = Direction3::from_vector(fun_tool_nggeom_f(uv_for, &c.for_)?)?;
        if c.ofor == Orientation::Reversed {
            nt_for.reverse();
        }
        let mut dot = xx_fs.dot(nt_for);
        if dot.abs() < c.tola {
            let Some(dxx) = fun_tool_getdxx(&c.fs, &c.eg, par_eg)? else {
                return Ok(None);
            };
            let newuv_fs = uv_fs.translated(dxx * 0.25);
            let newng_fs = Direction3::from_vector(fun_tool_nggeom_f(newuv_fs, &c.fs)?)?;
            let Some(x) = fun_tool_getxx_ng(&c.fs, &c.eg, par_eg, newng_fs)? else {
                return Ok(None);
            };
            xx_fs = x;
            dot = xx_fs.dot(nt_for);
        }
        Ok(Some((dot, ng_fs)))
    }

    /// Whether the yap2 split is kept by the side of the other face and the operation.
    fn yap2_keep(&self, c: &OnContext, cx: &FcxContext, positive: bool) -> Result<bool, Failure> {
        let pb = self.pb()?;
        let tbfs = fun_build_tb(pb, c.rank_fs);
        let mut keep = true;
        if cx.ff_in_sdso {
            if c.ope_fus {
                keep = positive;
            }
            if c.ope_com {
                keep = true;
            }
            if c.ope_cut {
                keep = if positive {
                    tbfs == State::Out
                } else {
                    tbfs == State::In
                };
            }
        }
        if cx.ff_in_sddo {
            if c.ope_fus {
                keep = positive;
                if keep {
                    let rk_ou = if c.tb1 == State::Out { 1 } else { 2 };
                    let rkfsp_ou = if c.rank_for == rk_ou { 1 } else { 2 };
                    keep = c.rank_for == rkfsp_ou;
                }
            }
            if c.ope_com {
                keep = !positive;
            }
            if c.ope_cut {
                let for_moins_fs = tbfs == State::In;
                keep = if positive == for_moins_fs {
                    true
                } else {
                    c.com_of_cut
                };
            }
        }
        Ok(keep)
    }

    /// The orientation of the yap2 split: the edge in the connected face, complemented
    /// by the operation and the same domain configuration (None when a direction fails).
    fn yap2_orientation(
        &self,
        c: &OnContext,
        cx: &FcxContext,
        par_eg: f64,
        ng_fs: Vector3,
    ) -> Result<Option<Orientation>, Failure> {
        let mut neworie = fun_tool_orient_e_in_f_forward(&c.eg, &cx.fcx)?.1;
        let ok = matches!(neworie, Orientation::Forward | Orientation::Reversed);
        if cx.ff_in_sdso && ok && c.ope_cut {
            neworie = top_abs::complement(neworie);
        }
        if cx.ff_in_sddo && ok {
            if c.ope_cut && !c.com_of_cut {
                neworie = top_abs::complement(neworie);
            }
            if c.ope_fus {
                neworie = top_abs::complement(neworie);
            }
            if c.ope_com {
                let Some(xx_fcx) = fun_tool_getxx(&cx.fcx, &c.eg, par_eg)? else {
                    return Ok(None);
                };
                let mut nt_fs = Direction3::from_vector(ng_fs)?;
                if c.ofs == Orientation::Reversed {
                    nt_fs.reverse();
                }
                if xx_fcx.dot(nt_fs) > 0. {
                    neworie = top_abs::complement(neworie);
                }
            }
        }
        let mut reverse = false;
        if !cx.fcx_isref {
            reverse = cx.cfcx == TopOpeBRepDSConfig::DiffOriented;
            if cx.ofcx != cx.oref_fcx {
                reverse = !reverse;
            }
        }
        Ok(Some(if reverse {
            top_abs::complement(neworie)
        } else {
            neworie
        }))
    }

    /// The case yap2: the edge without same domain on the split: kept by the side of the
    /// other face (by the transitions in the faces on faces case), IN the face, oriented
    /// in the connected face.
    pub(super) fn yap2(&mut self, c: &OnContext, cx: &FcxContext) -> Result<(), Failure> {
        let pb = self.pb()?;
        let mut oeg = cx.oeg_fcxf;
        let (mut b3d, mut b2d) = (false, false);
        let b = b3d && b2d;
        for (case, flip) in [
            (cx.ff_in_sdso, c.ofor != cx.ofcx),
            (cx.ff_in_sddo, c.ofor == cx.ofcx),
        ] {
            if case {
                if flip {
                    oeg = top_abs::complement(oeg);
                }
                (b3d, b2d) = b3d_b2d(oeg, c.tfebef, c.tfeaft, cx.sta_fs);
            }
        }
        let _ = (b3d, b2d);
        let (mut ng_fs, mut par_eg) = (Vector3::default(), 0.);
        let isfafa = fun_ds_sdm(&pb.hds()?.ds(), c.for_.as_ref(), c.fs.as_ref());
        if !isfafa {
            if !fun_tool_par_e_tol(&c.eon, c.par_on, &c.eg, &mut par_eg, c.tol_eg)? {
                return Ok(());
            }
            let Some((dot, ng)) = self.yap2_side(c, par_eg)? else {
                return Ok(());
            };
            ng_fs = ng;
            if !self.yap2_keep(c, cx, dot > 0.)? {
                return Ok(());
            }
        } else if !b {
            return Ok(());
        }
        let state2d = fsc_get_psc(|psc| fsc_state_eonface(&c.esp_on, 0.345, c.for_.as_ref(), psc))?;
        if state2d != State::In {
            return Ok(());
        }
        let Some(neworie) = self.yap2_orientation(c, cx, par_eg, ng_fs)? else {
            return Ok(());
        };
        self.add_start(&c.esp_on.oriented(neworie))
    }
}
