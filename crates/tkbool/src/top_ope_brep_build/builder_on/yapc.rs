//! The cases yapc1, yapc2 and yapc3 of GFillONPartsWES2: the section edge is a closing
//! edge of a face, or its 3d same domain edge is a closing edge of the face.
use super::context::OnContext;
use super::{TopOpeBRepBuildBuilderON, fun_keep_eon};
use crate::top_ope_brep_ds::TopOpeBRepDSConfig;
use crate::top_ope_brep_ds::export::fds_config3d;
use crate::top_ope_brep_tool::{
    fsc_get_psc, fsc_state_eonface, fun_tool_curves_so, fun_tool_curves_so_at, fun_tool_getxx_ng,
    fun_tool_nggeom_f, fun_tool_orient_e_in_f_forward, fun_tool_par_e_tol, fun_tool_par_f_tol,
    fun_tool_paron_ef_tol,
};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Direction3, Point2};

/// The ON split kept by the orientation of a same domain edge (as yap3 and yap5): the
/// orientation of the same domain edge in the face, oriented by the configuration.
pub(super) fn keep_by_sd_edge(
    c: &OnContext,
    esd: &Shape,
    oesd: Orientation,
) -> Result<Option<Orientation>, Failure> {
    let Some(cf) = fds_config3d(&c.esp_on, esd)? else {
        return Ok(None);
    };
    let mut par_eg = 0.;
    if !fun_tool_par_e_tol(&c.eon, c.par_on, &c.eg, &mut par_eg, c.tol_eg)? {
        return Ok(None);
    }
    let mut samegeom = false;
    if !fun_tool_curves_so_at(&c.eg, par_eg, &topods::edge(esd)?, &mut samegeom)? {
        return Ok(None);
    }
    let oesdk = if samegeom {
        oesd
    } else {
        top_abs::complement(oesd)
    };
    let keep = match oesdk {
        Orientation::Forward => c.tfeaft == c.tb1,
        Orientation::Reversed => c.tfebef == c.tb1,
        Orientation::Internal => true,
        Orientation::External => false,
    };
    if !keep {
        return Ok(None);
    }
    Ok(Some(if cf == TopOpeBRepDSConfig::DiffOriented {
        top_abs::complement(oesd)
    } else {
        oesd
    }))
}

impl TopOpeBRepBuildBuilderON<'_> {
    /// The case yapc1: a closing edge not bounding the face with a 3d same domain edge
    /// on the face: the ON split oriented as that edge.
    pub(super) fn yapc1(&mut self, c: &OnContext) -> Result<(), Failure> {
        let e3d = self.pb()?.hds()?.shape(c.ie3d, true);
        if let Some(o) = keep_by_sd_edge(c, &e3d, c.oe3d)? {
            self.add_start(&c.esp_on.oriented(o))?;
        }
        Ok(())
    }

    /// The ON split added with the orientation, both FORWARD and REVERSED when asked.
    fn add_oriented_or_both(
        &mut self,
        c: &OnContext,
        o: Orientation,
        both: bool,
    ) -> Result<(), Failure> {
        if both {
            self.add_start(&c.esp_on.oriented(Orientation::Forward))?;
            return self.add_start(&c.esp_on.oriented(Orientation::Reversed));
        }
        self.add_start(&c.esp_on.oriented(o))
    }

    /// The orientation of the transition on the state, the edge orientation in the face
    /// for an INTERNAL/EXTERNAL one of an edge bounding the face (not closing).
    fn transition_orientation(&self, c: &OnContext) -> Result<Orientation, Failure> {
        let mut o = c.tfe.orientation(c.tb1, ShapeEnum::Face);
        if c.eg_bound_for
            && !c.isclosed_ff
            && matches!(o, Orientation::External | Orientation::Internal)
        {
            o = fun_tool_orient_e_in_f_forward(&c.eg, &c.for_)?.1;
        }
        Ok(o)
    }

    /// The case yapc2: a closing edge (without 3d same domain on the face): the ON split
    /// oriented by the transition (kept by FUN_keepEON when bounding the face; checked
    /// IN the face without 3d same domain), INTERNAL on a closing edge doubled.
    pub(super) fn yapc2(&mut self, c: &OnContext) -> Result<(), Failure> {
        if c.eg_bound_for && !fun_keep_eon(c.eg.as_ref(), c.for_.as_ref(), &c.tfe, c.tb1)? {
            return Ok(());
        }
        let neworie = self.transition_orientation(c)?;
        if c.yapc2b && neworie != Orientation::External {
            let state2d =
                fsc_get_psc(|psc| fsc_state_eonface(&c.esp_on, 0.345, c.for_.as_ref(), psc))?;
            if state2d != State::In {
                return Ok(());
            }
        }
        let (mut neworiek, mut add_forrev) = (Orientation::External, false);
        if !c.isclosed_ff || matches!(neworie, Orientation::Forward | Orientation::Reversed) {
            neworiek = neworie;
        } else if neworie == Orientation::Internal {
            add_forrev = true;
        }
        self.add_oriented_or_both(c, neworiek, add_forrev)
    }

    /// Whether the faces are approximately same oriented at the edge for yapc3 (by
    /// their normals, or the material direction of the other face when orthogonal);
    /// None when a parameter fails.
    fn yapc3_same_oriented(&self, c: &OnContext) -> Result<Option<bool>, Failure> {
        let mut par_eg = 0.;
        if !fun_tool_par_e_tol(&c.eon, c.par_on, &c.eg, &mut par_eg, c.tol_eg)? {
            return Ok(None);
        }
        let mut uv_fs = Point2::default();
        if !fun_tool_paron_ef_tol(&c.eg, par_eg, &c.fs, &mut uv_fs, c.tol_fs)? {
            return Ok(None);
        }
        let ng_fs = fun_tool_nggeom_f(uv_fs, &c.fs)?;
        let mut nt_fs = Direction3::from_vector(ng_fs)?;
        if c.ofs == Orientation::Reversed {
            nt_fs.reverse();
        }
        let mut uv_for = Point2::default();
        if !fun_tool_par_f_tol(&c.eg, par_eg, &c.for_, &mut uv_for, c.tol_for)? {
            return Ok(None);
        }
        let mut nt_for = Direction3::from_vector(fun_tool_nggeom_f(uv_for, &c.for_)?)?;
        if c.ofor == Orientation::Reversed {
            nt_for.reverse();
        }
        let dot = nt_for.dot(nt_fs);
        if dot.abs() < c.tola {
            let Some(xx_fs) =
                fun_tool_getxx_ng(&c.fs, &c.eg, par_eg, Direction3::from_vector(ng_fs)?)?
            else {
                return Ok(None);
            };
            return Ok(Some(nt_for.dot(xx_fs) < 0.));
        }
        Ok(Some(dot > 0.))
    }

    /// The case yapc3: the 3d same domain edge is a closing edge of the face: the ON
    /// split oriented by the transition and its curve, or doubled for an
    /// INTERNAL/EXTERNAL transition (but the same oriented EXTERNAL of a common and
    /// INTERNAL of a fuse).
    pub(super) fn yapc3(&mut self, c: &OnContext) -> Result<(), Failure> {
        if c.eg_bound_for && !fun_keep_eon(c.eg.as_ref(), c.for_.as_ref(), &c.tfe, c.tb1)? {
            return Ok(());
        }
        let neworie = self.transition_orientation(c)?;
        let mut add_forrev = false;
        if matches!(c.otfe, Orientation::Internal | Orientation::External) {
            let Some(so) = self.yapc3_same_oriented(c)? else {
                return Ok(());
            };
            let unkeepclo = so
                && ((c.otfe == Orientation::External && c.ope_com)
                    || (c.otfe == Orientation::Internal && c.ope_fus));
            add_forrev = !unkeepclo;
        }
        if add_forrev {
            return self.add_oriented_or_both(c, neworie, true);
        }
        let mut samegeom = false;
        if !fun_tool_curves_so(&c.eon, &c.eg, &mut samegeom)? {
            return Ok(());
        }
        let neworiek = if samegeom {
            neworie
        } else {
            top_abs::complement(neworie)
        };
        self.add_start(&c.esp_on.oriented(neworiek))
    }
}
