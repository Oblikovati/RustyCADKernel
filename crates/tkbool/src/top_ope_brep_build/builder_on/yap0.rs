//! The cases yap00 and yap0 of GFillONPartsWES2: the section edge bounds the face.
use super::context::OnContext;
use super::{TopOpeBRepBuildBuilderON, compute_face_crvt_in_sec, fun_kpart};
use crate::top_ope_brep_tool::{
    fun_tool_curves_so_at, fun_tool_getxx_ng, fun_tool_nggeom_f_on_edge_tol,
    fun_tool_orient_e_in_f_forward, fun_tool_par_e_tol, fun_tool_paron_ef_tol, fun_tool_proj_ponf,
    fun_tool_value_on_face,
};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector3};

/// Whether an oriented edge of the face keeps the part on the state by the transition.
fn add_by_orientation(oeff: Orientation, c: &OnContext) -> bool {
    match oeff {
        Orientation::Forward => c.tfeaft == c.tb1,
        Orientation::Reversed => c.tfebef == c.tb1,
        _ => true,
    }
}

/// The kept part of the tangent case by the sides of the faces: same oriented normals
/// keep the part along the material by the operation.
fn keep_tangent(c: &OnContext, so: bool, dot: f64) -> bool {
    let side = dot > 0.;
    let mut keep = true;
    let flip = |k: bool| if c.tbfor == State::Out { !k } else { k };
    if so {
        if c.ope_fus {
            keep = !side;
        }
        if c.ope_com {
            keep = side;
        }
        if c.ope_cut {
            keep = !flip(!side);
        }
    } else {
        if c.ope_fus {
            keep = side;
        }
        if c.ope_com {
            keep = !side;
        }
        if c.ope_cut {
            keep = flip(!side);
        }
    }
    keep
}

impl TopOpeBRepBuildBuilderON<'_> {
    /// The case yap00: the ON split of an edge bounding the face (without same domain
    /// face) on a same domain edge of the other face, oriented as the edge.
    pub(super) fn yap00(&mut self, c: &OnContext) -> Result<(), Failure> {
        let (_, oeff) = fun_tool_orient_e_in_f_forward(&c.eg, &c.for_)?;
        if !add_by_orientation(oeff, c) {
            return Ok(());
        }
        let esd = self.pb()?.hds()?.shape(c.iesd, true);
        if !Explorer::new(c.fs.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?
            .any(|e| e.is_same(&esd))
        {
            return Ok(());
        }
        let mut par_eg = 0.;
        if !fun_tool_par_e_tol(&c.eon, c.par_on, &c.eg, &mut par_eg, c.tol_eg)? {
            return Ok(());
        }
        let mut samegeom = false;
        if !fun_tool_curves_so_at(&c.eg, par_eg, &c.eon, &mut samegeom)? {
            return Ok(());
        }
        let mut neworie = oeff;
        if !samegeom && matches!(neworie, Orientation::Forward | Orientation::Reversed) {
            neworie = top_abs::complement(neworie);
        }
        self.add_start(&c.esp_on.oriented(neworie))
    }

    /// The geometric normals of the two faces at the edge and the material direction of
    /// the face (None when one fails).
    fn yap0_frames(
        &self,
        c: &OnContext,
        par_eg: f64,
    ) -> Result<Option<(Direction3, Direction3, Direction3)>, Failure> {
        let tol_on = c.tol_fs.max(c.tol_eg) * 1.0e2;
        let Some(ng_fs) = fun_tool_nggeom_f_on_edge_tol(par_eg, &c.eg, &c.fs, tol_on)? else {
            return Ok(None);
        };
        let tol_on = c.tol_for.max(c.tol_eg) * 1.0e2;
        let Some(ng_for) = fun_tool_nggeom_f_on_edge_tol(par_eg, &c.eg, &c.for_, tol_on)? else {
            return Ok(None);
        };
        let Some(xx_for) =
            fun_tool_getxx_ng(&c.for_, &c.eg, par_eg, Direction3::from_vector(ng_for)?)?
        else {
            return Ok(None);
        };
        let mut nt_fs = Direction3::from_vector(ng_fs)?;
        if c.ofs == Orientation::Reversed {
            nt_fs.reverse();
        }
        let mut nt_for = Direction3::from_vector(ng_for)?;
        if c.ofor == Orientation::Reversed {
            nt_for.reverse();
        }
        Ok(Some((nt_fs, nt_for, xx_for)))
    }

    /// Whether the curvatures of the faces (non particular surfaces) differ along the
    /// material direction (None when they cannot be compared, false when equal).
    fn curvatures_differ(
        c: &OnContext,
        uv_for: Point2,
        p_for: Point3,
        xx_for: Direction3,
        ntdot: f64,
        tol_on: f64,
    ) -> Result<bool, Failure> {
        let Some(mut crv_for) = compute_face_crvt_in_sec(&c.for_, uv_for, xx_for)? else {
            return Ok(false);
        };
        let (mut uv_fs, mut d) = (Point2::default(), 0.);
        if !fun_tool_proj_ponf(
            p_for,
            &c.fs,
            &mut uv_fs,
            &mut d,
            ExtFlag::MinMax,
            ExtAlgo::Grad,
        )? {
            return Ok(false);
        }
        let Some(mut crv_fs) = compute_face_crvt_in_sec(&c.fs, uv_fs, xx_for)? else {
            return Ok(false);
        };
        if c.ofor == Orientation::Reversed {
            crv_for = -crv_for;
        }
        if c.ofs == Orientation::Reversed {
            crv_fs = -crv_fs;
        }
        if ntdot < 0. {
            crv_fs = -crv_fs;
        }
        let eps = precision::confusion();
        let (a_for, a_fs) = (crv_for.abs(), crv_fs.abs());
        if a_for <= eps && a_fs <= eps {
            return Ok(false);
        }
        if a_for > eps && a_fs > eps && (1. / crv_for - 1. / crv_fs).abs() <= tol_on {
            return Ok(false);
        }
        Ok(true)
    }

    /// The tangent case of yap0: the faces compared by a point moved inside the face
    /// along the material (with their curvatures when not a particular pair); false
    /// when the split is not kept.
    fn yap0_tangent(
        &self,
        c: &OnContext,
        par_eg: f64,
        frames: (Direction3, Direction3, Direction3),
        ntdot: f64,
    ) -> Result<bool, Failure> {
        let (_, nt_for, xx_for) = frames;
        let mut uv_for = Point2::default();
        if !fun_tool_paron_ef_tol(&c.eg, par_eg, &c.for_, &mut uv_for, c.tol_for)? {
            return Ok(false);
        }
        let mut p_for = fun_tool_value_on_face(uv_for, &c.for_)?;
        let tol_on = c.tol_for.max(c.tol_fs).max(c.tol_eg) * 10.;
        let st_for = BRepAdaptorSurface::with_face(&c.for_, true)?.get_type()?;
        let st_fs = BRepAdaptorSurface::with_face(&c.fs, true)?.get_type()?;
        if fun_kpart(st_for, st_fs) == 0
            && !Self::curvatures_differ(c, uv_for, p_for, xx_for, ntdot, tol_on)?
        {
            return Ok(false);
        }
        let so = ntdot > 0.;
        p_for.translate(Vector3::from(xx_for.xyz() * (tol_on * 10.)));
        let (mut uv_fs, mut d) = (Point2::default(), 0.);
        if !fun_tool_proj_ponf(
            p_for,
            &c.for_,
            &mut uv_for,
            &mut d,
            ExtFlag::MinMax,
            ExtAlgo::Grad,
        )? {
            return Ok(false);
        }
        p_for = fun_tool_value_on_face(uv_for, &c.for_)?;
        if !fun_tool_proj_ponf(
            p_for,
            &c.fs,
            &mut uv_fs,
            &mut d,
            ExtFlag::MinMax,
            ExtAlgo::Grad,
        )? {
            return Ok(false);
        }
        let p_fs = fun_tool_value_on_face(uv_fs, &c.fs)?;
        let dforfs = Direction3::from_vector(Vector3::between(p_for, p_fs))?;
        Ok(keep_tangent(c, so, dforfs.dot(nt_for)))
    }

    /// The case yap0: the ON split of an edge bounding the face, kept by the side of
    /// the other face (or the tangent comparison), oriented as the edge in the face.
    pub(super) fn yap0(&mut self, c: &OnContext) -> Result<(), Failure> {
        let (_, oeff) = fun_tool_orient_e_in_f_forward(&c.eg, &c.for_)?;
        if !add_by_orientation(oeff, c) {
            return Ok(());
        }
        let mut par_eg = 0.;
        if !fun_tool_par_e_tol(&c.eon, c.par_on, &c.eg, &mut par_eg, c.tol_eg)? {
            return Ok(());
        }
        let Some(frames) = self.yap0_frames(c, par_eg)? else {
            return Ok(());
        };
        let (nt_fs, nt_for, xx_for) = frames;
        let ntdot = nt_for.dot(nt_fs);
        let kpart = matches!(c.otfe, Orientation::External | Orientation::Internal)
            && (1. - ntdot.abs()).abs() < c.tola * 1.0e2;
        let keep = if kpart {
            self.yap0_tangent(c, par_eg, frames, ntdot)?
        } else {
            let positive = nt_fs.dot(xx_for) > 0.;
            let mut keep = true;
            if c.ope_fus {
                keep = positive;
            }
            if c.ope_com {
                keep = !positive;
            }
            if c.ope_cut {
                keep = if positive {
                    c.tbfor == State::Out
                } else {
                    c.tbfor != State::Out
                };
            }
            keep
        };
        if !keep {
            return Ok(());
        }
        let (_, mut neworie) = fun_tool_orient_e_in_f_forward(&c.eg, &c.for_)?;
        if c.hsd_for && !c.for_isref {
            let mut reverse = c.c_for == crate::top_ope_brep_ds::TopOpeBRepDSConfig::DiffOriented;
            if c.ofor != c.oref_for {
                reverse = !reverse;
            }
            if reverse {
                neworie = top_abs::complement(neworie);
            }
        }
        self.add_start(&c.esp_on.oriented(neworie))
    }
}
