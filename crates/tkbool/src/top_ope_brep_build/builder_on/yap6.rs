//! The case yap6 of GFillONPartsWES2: the edge with a (2d) same domain edge on the
//! split, in a face of the same domain as the connected face.
use super::TopOpeBRepBuildBuilderON;
use super::context::{FcxContext, OnContext};
use crate::top_ope_brep_ds::export::fun_ds_share_g;
use crate::top_ope_brep_ds::{TopOpeBRepDSTransition, fdscnx_face_edge_connex_faces};
use crate::top_ope_brep_tool::{
    TopOpeBRepToolTOOL, fun_tool_getxx, fun_tool_nggeom_f, fun_tool_par_e, fun_tool_par_e_tol,
    fun_tool_paron_ef_tol,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Point2, Vector3};
use std::f64::consts::PI;

/// The kept flag of the matters of the two faces around the edges (fuse of same
/// oriented faces): the sum of the matters below PI, or the smaller matter.
#[allow(clippy::if_same_then_else)] // the native branches, kept apart
fn matters_keep(matfs: f64, matfor: f64, tola1: f64) -> bool {
    let sum = matfs + matfor;
    if (sum - PI).abs() < tola1 {
        false
    } else if sum < PI {
        true
    } else if matfs < PI && matfor < PI {
        false
    } else if (matfs - matfor).abs() < tola1 {
        false
    } else {
        matfs < matfor
    }
}

impl TopOpeBRepBuildBuilderON<'_> {
    /// The fuse keep of a same oriented split by the matters of the faces around the
    /// edge and its same domain edge (None to drop, Some((b, forcekeep))).
    fn yap6_fuse_matters(
        &self,
        c: &OnContext,
        cx: &FcxContext,
        share_g: bool,
    ) -> Result<Option<(bool, bool)>, Failure> {
        let pb = self.pb()?;
        let esd = topods::edge(&pb.hds()?.shape(c.iesd, true))?;
        let lfor =
            fdscnx_face_edge_connex_faces(c.for_.as_ref(), esd.as_ref(), &pb.my_data_structure)?;
        if lfor.is_empty() {
            return Ok(Some((share_g, false)));
        }
        if lfor.len() > 1 {
            return Ok(None);
        }
        let ff = topods::face(lfor.first().unwrap_or(c.for_.as_ref()))?;
        let tola1 = precision::angular() * 1.0e2;
        let mut par_eg = 0.;
        if !fun_tool_par_e_tol(&c.eon, c.par_on, &c.eg, &mut par_eg, c.tol_eg)? {
            return Ok(None);
        }
        let Some(matfs) = TopOpeBRepToolTOOL::matter_faces(&c.fs, &cx.fcx, &c.eg, par_eg, tola1)?
        else {
            return Ok(None);
        };
        let tol_esd = BRepTool::edge_tolerance(&esd)? * c.factor;
        let mut par_esd = 0.;
        if !fun_tool_par_e_tol(&c.eon, c.par_on, &esd, &mut par_esd, tol_esd)? {
            return Ok(None);
        }
        let Some(matfor) = TopOpeBRepToolTOOL::matter_faces(&c.for_, &ff, &esd, par_esd, tola1)?
        else {
            return Ok(None);
        };
        let b = matters_keep(matfs, matfor, tola1);
        Ok(Some((b, b)))
    }

    /// The same oriented case of yap6 (None to drop).
    fn yap6_same(
        &self,
        c: &OnContext,
        cx: &FcxContext,
        share_g: bool,
    ) -> Result<Option<bool>, Failure> {
        let mut forcekeep = c.ope_fus && share_g && c.rank_eg == 1;
        let mut b = false;
        if c.ope_fus {
            let Some((bb, fk)) = self.yap6_fuse_matters(c, cx, share_g)? else {
                return Ok(None);
            };
            b = bb;
            forcekeep |= fk;
        } else if c.ope_cut {
            b = !share_g;
        } else if c.ope_com {
            b = share_g && c.rank_eg == 1;
        }
        if b {
            let oeg = if c.ofor != cx.ofcx {
                top_abs::complement(cx.oeg_fcxf)
            } else {
                cx.oeg_fcxf
            };
            let t1 = TopOpeBRepDSTransition::from_orientation(oeg);
            let (mut b3d, mut b2d) = (false, false);
            match oeg {
                Orientation::Forward => {
                    (b3d, b2d) = (c.tfeaft == cx.sta_fs, t1.after() == State::In)
                }
                Orientation::Reversed => {
                    (b3d, b2d) = (c.tfebef == cx.sta_fs, t1.before() == State::In)
                }
                _ => {}
            }
            b = b3d && b2d;
        }
        Ok(Some(b || forcekeep))
    }

    /// The OUT part of the first shape in a cut of different orientations: kept unless
    /// the face connected to its same domain edge turns away from the connected face
    /// (None to drop).
    fn yap6_binou(&self, c: &OnContext, cx: &FcxContext) -> Result<Option<bool>, Failure> {
        let pb = self.pb()?;
        let esd: Edge = topods::edge(&pb.hds()?.shape(c.iesd, true))?;
        let lfc =
            fdscnx_face_edge_connex_faces(c.for_.as_ref(), esd.as_ref(), &pb.my_data_structure)?;
        if lfc.len() != 1 {
            return Ok(None);
        }
        let mut par_esd = 0.;
        if !fun_tool_par_e_tol(&c.eon, c.par_on, &esd, &mut par_esd, c.tol_eg)? {
            return Ok(None);
        }
        let oofor: Face = topods::face(lfc.first().unwrap_or(c.for_.as_ref()))?;
        let mut uv = Point2::default();
        fun_tool_paron_ef_tol(&esd, par_esd, &oofor, &mut uv, 0.)?;
        let mut nt_oofor = fun_tool_nggeom_f(uv, &oofor)?;
        if oofor.orientation() == Orientation::Reversed {
            nt_oofor.reverse();
        }
        let mut par_eg = 0.;
        if !fun_tool_par_e(&esd, par_esd, &c.eg, &mut par_eg)? {
            return Ok(None);
        }
        let Some(xx_fcx) = fun_tool_getxx(&cx.fcx, &c.eg, par_eg)? else {
            return Ok(None);
        };
        let dot = nt_oofor.dot(Vector3::from(xx_fcx));
        Ok(Some(if dot.abs() < c.tola { true } else { dot >= 0. }))
    }

    /// The different oriented case of yap6 (None to drop).
    fn yap6_diff(
        &self,
        c: &OnContext,
        cx: &FcxContext,
        share_g: bool,
        b: bool,
    ) -> Result<Option<bool>, Failure> {
        let mut b = b;
        if c.ope_fus {
            b = !share_g;
        }
        if c.ope_cut {
            let rank_ref = if c.opec12 { 1 } else { 2 };
            let (mut binin, mut binou) = (false, false);
            if c.tb1 == c.tb2 {
                binin = c.rank_eg == rank_ref;
            } else {
                binou = c.rank_for == 1 && c.tbfor == State::Out && !share_g;
                if binou {
                    match self.yap6_binou(c, cx)? {
                        Some(x) => binou = x,
                        None => return Ok(None),
                    }
                }
            }
            b = binin || binou;
        }
        Ok(Some(b))
    }

    /// Whether the split is added twice (complemented) in a cut of same orientations
    /// without shared geometry, when an edge of the face split IN has IN splits.
    fn yap6_addtwice(
        &self,
        c: &OnContext,
        cx: &FcxContext,
        share_g: bool,
        neworie: Orientation,
    ) -> Result<bool, Failure> {
        if !(cx.ff_in_sdso
            && !share_g
            && c.ope_cut
            && matches!(neworie, Orientation::Forward | Orientation::Reversed))
        {
            return Ok(false);
        }
        let pb = self.pb()?;
        let ftosp_in = if c.tbfor == State::In {
            c.for_.as_ref()
        } else {
            cx.fcx.as_ref()
        };
        for ee in Explorer::new(ftosp_in, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !pb.hds()?.ds().has_shape(&ee, true) {
                continue;
            }
            if pb.is_split(&ee, State::In) && !pb.splits(&ee, State::In).is_empty() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The case yap6: kept by the shared geometry, the matters (fuse) or the side of the
    /// face connected to the same domain edge (cut), oriented by the reference of the
    /// same domain, doubled in a cut of same orientations.
    pub(super) fn yap6(&mut self, c: &OnContext, cx: &FcxContext) -> Result<(), Failure> {
        let pb = self.pb()?;
        let ifcx = pb.hds()?.shape_index(cx.fcx.as_ref(), true);
        let Some(share_g) = fun_ds_share_g(&pb.my_data_structure, c.ifor, ifcx, c.gi, &c.eon)?
        else {
            return Ok(());
        };
        let mut b = false;
        if cx.ff_in_sdso {
            match self.yap6_same(c, cx, share_g)? {
                Some(x) => b = x,
                None => return Ok(()),
            }
        }
        if cx.ff_in_sddo {
            match self.yap6_diff(c, cx, share_g, b)? {
                Some(x) => b = x,
                None => return Ok(()),
            }
        }
        if !b {
            return Ok(());
        }
        let neworie = super::yap12::orientation_by_reference(c, cx, pb)?;
        self.add_start(&c.esp_on.oriented(neworie))?;
        if self.yap6_addtwice(c, cx, share_g, neworie)? {
            self.add_start(&c.esp_on.oriented(top_abs::complement(neworie)))?;
        }
        Ok(())
    }
}
