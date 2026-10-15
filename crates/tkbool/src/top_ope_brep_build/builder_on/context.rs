//! The configuration of an ON split in GFillONPartsWES2: the face, the section edge,
//! the other face, their orientations, ranks and same domains, and the case flags.
use super::{TopOpeBRepBuildBuilder, fun_build_tb, fun_find_esd};
use crate::top_ope_brep_build::TopOpeBRepBuildGTopo;
use crate::top_ope_brep_ds::export::{
    fds_assign, fds_data, fds_has_same_domain3d, fun_ds_one_sd, fun_ds_sdm,
};
use crate::top_ope_brep_ds::process_interferences_tool::fun_select_gi_interference;
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSConfig, TopOpeBRepDSInterference,
    TopOpeBRepDSShapeShapeInterference, TopOpeBRepDSTransition, fdscnx_face_edge_connex_faces,
};
use crate::top_ope_brep_tool::{
    fun_tool_bounds, fun_tool_curves_so, fun_tool_is_closing_e, fun_tool_maxtol,
    fun_tool_orient_e_in_f, fun_tool_orient_e_in_f_forward, fun_tool_proj_ponbounded_f,
    fun_tool_value_on_edge,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point2;

/// The configuration of an ON split of a section edge on a face.
pub(super) struct OnContext {
    pub tb1: State,
    pub tb2: State,
    pub gi: i32,
    pub tfe: TopOpeBRepDSTransition,
    pub eg_bound_for: bool,
    pub for_: Face,
    pub ifor: i32,
    pub eon: Edge,
    pub esp_on: Shape,
    pub eg: Edge,
    pub fs: Face,
    pub tola: f64,
    pub ofor: Orientation,
    pub ofs: Orientation,
    pub isclosed_ff: bool,
    pub isrest: bool,
    pub rank_fs: i32,
    pub rank_eg: i32,
    pub rank_for: i32,
    pub otfe: Orientation,
    pub tfebef: State,
    pub tfeaft: State,
    pub eghassd: bool,
    pub le3: List<Shape>,
    pub eghassd3d: bool,
    pub ie3d: i32,
    pub oe3d: Orientation,
    pub lfcx: List<Shape>,
    pub hsd_for: bool,
    pub c_for: TopOpeBRepDSConfig,
    pub iref_for: i32,
    pub oref_for: Orientation,
    pub for_isref: bool,
    pub ope_fus: bool,
    pub opec12: bool,
    pub ope_cut: bool,
    pub ope_com: bool,
    pub com_of_cut: bool,
    pub tbfor: State,
    pub factor: f64,
    pub tol_eg: f64,
    pub tol_for: f64,
    pub tol_fs: f64,
    pub par_on: f64,
    pub esp_onesd: bool,
    pub iesd: i32,
    pub eon_so_esd: bool,
    pub eon_fs: bool,
    pub eghassd_on: bool,
    pub eghassd3d_on: bool,
    pub yap00: bool,
    pub yap0: bool,
    pub yapc1: bool,
    pub yapc2b: bool,
    pub yapc2: bool,
    pub yapc3: bool,
}

/// The configuration of the face connected to the edge in the other face.
pub(super) struct FcxContext {
    pub fcx: Face,
    pub rank_fcx: i32,
    pub ff_in_sdso: bool,
    pub ff_in_sddo: bool,
    pub ofcx: Orientation,
    pub cfcx: TopOpeBRepDSConfig,
    pub oref_fcx: Orientation,
    pub fcx_isref: bool,
    pub oeg_fcxf: Orientation,
    pub yap1: bool,
    pub yap2: bool,
    pub yap6: bool,
    pub yap5: bool,
    pub yap3: bool,
    pub yap4: bool,
    pub yap7: bool,
    pub sta_for: State,
    pub sta_fs: State,
}

/// The shape/shape interference (None when another interference).
fn ssi_of(
    i: &Handle<TopOpeBRepDSInterference>,
) -> Option<Handle<TopOpeBRepDSShapeShapeInterference>> {
    let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.clone().cast();
    ssi.get().is_some().then_some(ssi)
}

/// Whether the ON split point is on the other face (projection within its tolerance);
/// None when the point or projection fails.
fn eon_on_face(eon: &Edge, par_on: f64, fs: &Face) -> Result<Option<bool>, Failure> {
    let Some(p_on) = fun_tool_value_on_edge(par_on, eon)? else {
        return Ok(None);
    };
    let (mut uv_fs, mut d) = (Point2::default(), 1.);
    if !fun_tool_proj_ponbounded_f(p_on, fs, &mut uv_fs, &mut d)? {
        return Ok(None);
    }
    Ok(Some(d < fun_tool_maxtol(fs.as_ref())?))
}

impl OnContext {
    /// The configuration of the ON split (None when the edge geometry fails, as the
    /// native early returns).
    pub fn new(
        pb: &TopOpeBRepBuildBuilder,
        my_face: &Shape,
        pg: &TopOpeBRepBuildGTopo,
        i: &Handle<TopOpeBRepDSInterference>,
        esp_on: &Shape,
    ) -> Result<Option<Self>, Failure> {
        let hds = pb.hds()?;
        let ssi = ssi_of(i).ok_or_else(super::not_performed)?;
        let s = ssi.get().ok_or_else(super::not_performed)?;
        let (tb1, tb2) = pg.states_on()?;
        let (_, gi, _, si) = fds_data(i);
        let tfe = s.transition();
        let eg_bound_for = s.gbound();
        let for_ = topods::face(my_face)?;
        let (ifor, eg, fs) = {
            let bds = hds.ds();
            (
                bds.shape_index(for_.as_ref(), true),
                topods::edge(bds.shape(gi, true))?,
                topods::face(bds.shape(si, true))?,
            )
        };
        let ifs = hds.shape_index(fs.as_ref(), true);
        let eon = topods::edge(esp_on)?;
        let (ofor, ofs) = (
            hds.shape(ifor, true).orientation(),
            hds.shape(ifs, true).orientation(),
        );
        let isclosed_ff = fun_tool_is_closing_e(&eg, for_.as_ref(), &for_)?;
        let isclosed_fs = fun_tool_is_closing_e(&eg, fs.as_ref(), &fs)?;
        let isclosed = isclosed_ff || isclosed_fs;
        let isrest = hds.ds().is_section_edge(&eg, true);
        let (rank_fs, rank_eg, rank_for) = (
            pb.g_shape_rank(fs.as_ref()),
            pb.g_shape_rank(eg.as_ref()),
            pb.g_shape_rank(for_.as_ref()),
        );
        let eghassd = hds.has_same_domain(eg.as_ref(), true);
        let mut le3 = List::default();
        let eghassd3d = fds_has_same_domain3d(&hds.ds(), eg.as_ref(), Some(&mut le3));
        let lfcx = fdscnx_face_edge_connex_faces(fs.as_ref(), eg.as_ref(), &pb.my_data_structure)?;
        let hsd_for = hds.has_same_domain(for_.as_ref(), true);
        let (mut c_for, mut iref_for, mut oref_for, mut for_isref) = (
            TopOpeBRepDSConfig::UnshGeometry,
            0,
            Orientation::External,
            false,
        );
        if hsd_for {
            let bds = hds.ds();
            c_for = bds.same_domain_ori_index(ifor);
            iref_for = bds.same_domain_ref(for_.as_ref());
            if iref_for != 0 {
                oref_for = bds.shape(iref_for, true).orientation();
            }
            for_isref = iref_for == ifor;
        }
        let (ope_fus, opec12, ope_cut, ope_com) = (
            pb.opefus(),
            pb.opec12(),
            pb.opec12() || pb.opec21(),
            pb.opecom(),
        );
        let com_of_cut = ope_cut && tb1 == tb2 && tb1 == State::In;
        let factor = 1.0e2;
        let (tol_eg, tol_for, tol_fs) = (
            BRepTool::edge_tolerance(&eg)? * factor,
            BRepTool::face_tolerance(&for_)? * factor,
            BRepTool::face_tolerance(&fs)? * factor,
        );
        let (f, l) = fun_tool_bounds(&eon)?;
        let x = 0.456189;
        let par_on = (1. - x) * f + x * l;
        let mut iesd = 0;
        let mut esp_onesd = false;
        if eghassd && let Some(i) = fun_ds_one_sd(&hds.ds(), gi, esp_on)? {
            (esp_onesd, iesd) = (true, i);
        }
        let mut eon_so_esd = false;
        if eghassd && iesd != 0 {
            let esd = topods::edge(&hds.shape(iesd, true))?;
            if !fun_tool_curves_so(&eon, &esd, &mut eon_so_esd)? {
                return Ok(None);
            }
        }
        let mut eon_fs = true;
        if eghassd && !esp_onesd {
            match eon_on_face(&eon, par_on, &fs)? {
                Some(b) => eon_fs = b,
                None => return Ok(None),
            }
        }
        let mut c = Self {
            tb1,
            tb2,
            gi,
            tfe,
            eg_bound_for,
            for_,
            ifor,
            eon,
            esp_on: esp_on.clone(),
            eg,
            fs,
            tola: precision::angular() * 1.0e3,
            ofor,
            ofs,
            isclosed_ff,
            isrest,
            rank_fs,
            rank_eg,
            rank_for,
            otfe: tfe.orientation(State::In, ShapeEnum::Face),
            tfebef: tfe.before(),
            tfeaft: tfe.after(),
            eghassd,
            le3,
            eghassd3d,
            ie3d: 0,
            oe3d: Orientation::External,
            lfcx,
            hsd_for,
            c_for,
            iref_for,
            oref_for,
            for_isref,
            ope_fus,
            opec12,
            ope_cut,
            ope_com,
            com_of_cut,
            tbfor: fun_build_tb(pb, rank_for),
            factor,
            tol_eg,
            tol_for,
            tol_fs,
            par_on,
            esp_onesd,
            iesd,
            eon_so_esd,
            eon_fs,
            eghassd_on: eghassd && esp_onesd,
            eghassd3d_on: eghassd3d && esp_onesd,
            yap00: false,
            yap0: false,
            yapc1: false,
            yapc2b: false,
            yapc2: false,
            yapc3: false,
        };
        c.set_flags(pb, isclosed)?;
        Ok(Some(c))
    }

    /// The flags of the cases on the face (yap00, yap0, yapc1, yapc2, yapc3).
    fn set_flags(&mut self, pb: &TopOpeBRepBuildBuilder, isclosed: bool) -> Result<(), Failure> {
        let hds = pb.hds()?;
        self.yap00 = !isclosed && self.eg_bound_for && self.esp_onesd && !self.hsd_for;
        if self.yap00 {
            let mut copy = ListOfInterference::default();
            fds_assign(
                hds.ds().shape_interferences(self.for_.as_ref(), true),
                &mut copy,
            );
            let mut lgesd = ListOfInterference::default();
            if fun_select_gi_interference(&mut copy, self.iesd, &mut lgesd)? != 0 {
                self.yap00 = false;
            }
        }
        self.yap0 = !isclosed && self.eg_bound_for && !self.eghassd;
        let yap0bis = !isclosed
            && self.eg_bound_for
            && self.eghassd
            && !self.eghassd_on
            && self.eon_fs
            && !self.com_of_cut;
        if yap0bis {
            self.yap0 = true;
        }
        let mut hassd3d_on = false;
        if !self.eg_bound_for && self.eghassd3d {
            (self.ie3d, self.oe3d) = fun_find_esd(&hds.ds(), &self.eon, &self.eg, &self.for_, 3)?;
            hassd3d_on = self.ie3d != 0;
        }
        self.yapc1 = isclosed && !self.eg_bound_for && self.eghassd3d && hassd3d_on;
        let yapc2a = isclosed && !self.eghassd3d;
        self.yapc2b = isclosed && !self.eg_bound_for && self.eghassd3d && !hassd3d_on;
        let yapc2c = isclosed && self.eg_bound_for && self.eghassd3d;
        self.yapc2 = yapc2a || self.yapc2b || yapc2c;
        let mut e3closed_for = false;
        if hassd3d_on {
            let e3d = topods::edge(&hds.shape(self.ie3d, true))?;
            e3closed_for = BRepTool::is_closed_on_face(&e3d, &self.for_)?;
        }
        self.yapc3 = !isclosed && self.eghassd3d && hassd3d_on && e3closed_for;
        Ok(())
    }

    /// The configuration on the face connected to the edge in the other face (None when
    /// there is none or the ON split is not on the face).
    pub fn fcx_context(&self, pb: &TopOpeBRepBuildBuilder) -> Result<Option<FcxContext>, Failure> {
        let Some(fcx) = self.lfcx.first() else {
            return Ok(None);
        };
        let fcx = topods::face(fcx)?;
        let hds = pb.hds()?;
        let ifcx = hds.shape_index(fcx.as_ref(), true);
        let (mut lfso, mut lfdo) = (List::default(), List::default());
        pb.g_find_sam_dom_sodo_of(fcx.as_ref(), &mut lfso, &mut lfdo)?;
        let rank_fcx = pb.g_shape_rank(fcx.as_ref());
        let rank_x = match rank_fcx {
            0 => 0,
            1 => 2,
            _ => 1,
        };
        let (mut lfso2, mut lfdo2) = (List::default(), List::default());
        pb.g_find_same_rank(&lfso, rank_x, &mut lfso2);
        pb.g_find_same_rank(&lfdo, rank_x, &mut lfdo2);
        let ff_in_sdso = lfso2.iter().any(|f| f.is_same(self.for_.as_ref()));
        let ff_in_sddo = lfdo2.iter().any(|f| f.is_same(self.for_.as_ref()));
        let ff_in_sd = ff_in_sdso || ff_in_sddo;
        let (ofcx, cfcx, iref_fcx) = {
            let bds = hds.ds();
            (
                bds.shape(ifcx, true).orientation(),
                bds.same_domain_ori_index(ifcx),
                bds.same_domain_ref(fcx.as_ref()),
            )
        };
        let oref_fcx = hds.shape(iref_fcx, true).orientation();
        let (eg_bound_fcx, oeg_fcxf) = fun_tool_orient_e_in_f_forward(&self.eg, &fcx)?;
        fun_tool_orient_e_in_f(&self.eg, &fcx)?;
        if !self.eg_bound_for && !self.esp_onesd && !self.eon_on_for()? {
            return Ok(None);
        }
        let sdm = fun_ds_sdm(&hds.ds(), self.for_.as_ref(), self.fs.as_ref());
        let mut cx = FcxContext {
            fcx,
            rank_fcx,
            ff_in_sdso,
            ff_in_sddo,
            ofcx,
            cfcx,
            oref_fcx,
            fcx_isref: iref_fcx == ifcx,
            oeg_fcxf,
            yap1: false,
            yap2: false,
            yap6: false,
            yap5: false,
            yap3: false,
            yap4: false,
            yap7: false,
            sta_for: self.tb1,
            sta_fs: if self.rank_fs == self.rank_for {
                self.tb1
            } else {
                self.tb2
            },
        };
        self.set_fcx_flags(&mut cx, ff_in_sd, eg_bound_fcx, sdm);
        Ok(Some(cx))
    }

    /// Whether the point of the ON split projects on the face within the tolerances.
    fn eon_on_for(&self) -> Result<bool, Failure> {
        let Some(pt_on) = fun_tool_value_on_edge(self.par_on, &self.eon)? else {
            return Ok(false);
        };
        let (mut uv_for, mut d) = (Point2::default(), 1.);
        if !fun_tool_proj_ponbounded_f(pt_on, &self.for_, &mut uv_for, &mut d)? {
            return Ok(false);
        }
        Ok(d < self.tol_eg.max(self.tol_for) * 1.0e2)
    }

    /// The flags of the cases on the connected face (yap1 .. yap7).
    fn set_fcx_flags(&self, cx: &mut FcxContext, ff_in_sd: bool, eg_bound_fcx: bool, sdm: bool) {
        let nb = !self.eg_bound_for;
        cx.yap1 = ff_in_sd && nb && eg_bound_fcx && self.eghassd3d_on;
        cx.yap2 = ff_in_sd && nb && eg_bound_fcx && !self.eghassd;
        let yap1b =
            ff_in_sd && nb && eg_bound_fcx && self.eghassd3d && !self.eghassd3d_on && self.eon_fs;
        if yap1b {
            cx.yap2 = true;
        }
        cx.yap6 = ff_in_sd && nb && eg_bound_fcx && self.eghassd_on && !self.eghassd3d;
        let yap6b = ff_in_sd
            && nb
            && eg_bound_fcx
            && self.eghassd
            && !self.eghassd_on
            && self.eon_fs
            && !self.eghassd3d;
        if yap6b {
            cx.yap2 = true;
        }
        let eghassd3fcx = self.eghassd_on && !sdm && !ff_in_sd;
        cx.yap5 = nb && eghassd3fcx;
        cx.yap3 = !ff_in_sd && nb && self.eghassd3d_on;
        cx.yap4 = !ff_in_sd && self.isrest && !cx.yap5 && !self.hsd_for;
        let yap3b = !ff_in_sd && nb && self.eghassd3d && !self.eghassd3d_on && self.eon_fs;
        if yap3b {
            cx.yap4 = true;
        }
        cx.yap7 = !ff_in_sd && self.isrest && !cx.yap5 && self.hsd_for;
    }
}
