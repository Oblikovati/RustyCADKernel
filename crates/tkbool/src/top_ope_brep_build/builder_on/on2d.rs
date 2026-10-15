//! The ON parts of a face in 2d (TopOpeBRepBuild_BuilderON2d.cxx): the face/edge
//! interferences of the 2d structure of the same domain faces.
use super::{TopOpeBRepBuildBuilderON, fun_build_tb, not_performed};
use crate::top_ope_brep_build::globals::GLOBAL_DS2D;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildBuilder, TopOpeBRepBuildGTopo, TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::export::{fds_data, fun_ds_share_g};
use crate::top_ope_brep_ds::{
    TopOpeBRepDSHDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSShapeShapeInterference,
};
use crate::top_ope_brep_tool::{
    fun_tool_bounds, fun_tool_curves_so_at, fun_tool_orient_e_in_f, fun_tool_orient_e_in_f_forward,
};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, State};

impl<'a> TopOpeBRepBuildBuilderON<'a> {
    /// `Perform2d(PB, FOR, PG, PLSclass, PWES)`: each ON split of the edge of each
    /// interference of the face in the 2d structure filled;
    /// `bon.perform2d(&b, &f, &g, &l, &mut wes)?`.
    pub fn perform2d(
        &mut self,
        pb: &'a TopOpeBRepBuildBuilder,
        for_: &Shape,
        pg: &TopOpeBRepBuildGTopo,
        plsclass: &'a List<Shape>,
        pwes: &'a mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        self.my_pb = Some(pb);
        self.my_face = for_.clone();
        self.my_pg = Some(*pg);
        self.my_plsclass = Some(plsclass);
        self.my_pwes = Some(pwes);
        let ds2d = GLOBAL_DS2D.with_borrow_mut(|g| {
            g.get_or_insert_with(|| Handle::new(TopOpeBRepDSHDataStructure::new()))
                .clone()
        });
        let lfei = ds2d
            .get()
            .ok_or_else(not_performed)?
            .ds()
            .shape_interferences(for_, true)
            .clone();
        for i in lfei.iter() {
            let (_, gi, _, _) = fds_data(i);
            let eg = pb.hds()?.shape(gi, true);
            for esp_on in pb.splits(&eg, State::On).clone().iter() {
                self.g_fill_on_parts2d_wes2(i, esp_on)?;
            }
        }
        Ok(())
    }

    /// The orientation of the 2d ON split: the edge in the face (bounding it) or in the
    /// other face, complemented by its curve and the face orientations (None when the
    /// curves fail).
    fn on2d_orientation(
        &self,
        esp_on: &Shape,
        eg: &Edge,
        eg_bound_for: bool,
        ifor: i32,
        ifcx: i32,
        oeg_fcxf: Orientation,
    ) -> Result<Option<Orientation>, Failure> {
        let pb = self.pb()?;
        let mut neworie = oeg_fcxf;
        if eg_bound_for {
            neworie = fun_tool_orient_e_in_f_forward(eg, &topods::face(&self.my_face)?)?.1;
        }
        let ee = topods::edge(esp_on)?;
        let (f, l) = fun_tool_bounds(&ee)?;
        let par_on = (f + l) * 0.4352;
        let mut eso = false;
        if !fun_tool_curves_so_at(&ee, par_on, eg, &mut eso)? {
            return Ok(None);
        }
        if !eso {
            neworie = top_abs::complement(neworie);
        }
        let (ofor, ofcx) = (
            pb.hds()?.shape(ifor, true).orientation(),
            pb.hds()?.shape(ifcx, true).orientation(),
        );
        if ofor != ofcx {
            neworie = top_abs::complement(neworie);
        }
        Ok(Some(neworie))
    }

    /// `GFillONParts2dWES2(I, EspON)`: the ON split of an edge with same domain edges of
    /// the first shape (bounding one face) kept by its shared geometry and the
    /// operation, oriented in its face; `bon.g_fill_on_parts2d_wes2(&i, &esp)?`.
    pub fn g_fill_on_parts2d_wes2(
        &mut self,
        i: &Handle<TopOpeBRepDSInterference>,
        esp_on: &Shape,
    ) -> Result<(), Failure> {
        let pb = self.pb()?;
        let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.clone().cast();
        let s = ssi.get().ok_or_else(not_performed)?;
        self.my_pg.ok_or_else(not_performed)?.states_on()?;
        let (_, gi, _, si) = fds_data(i);
        let eg_bound_for = s.gbound();
        let hds = pb.hds()?;
        let ifor = hds.shape_index(&self.my_face, true);
        let eg = topods::edge(&hds.shape(gi, true))?;
        let fcx = topods::face(&hds.shape(si, true))?;
        let rank_eg = pb.g_shape_rank(eg.as_ref());
        let tbeg = fun_build_tb(pb, rank_eg);
        let eghassd = hds.has_same_domain(eg.as_ref(), true);
        let (eg_bound_fcx, oeg_fcxf) = fun_tool_orient_e_in_f_forward(&eg, &fcx)?;
        fun_tool_orient_e_in_f(&eg, &fcx)?;
        if !((eg_bound_for || eg_bound_fcx) && eghassd) || rank_eg != 1 {
            return Ok(());
        }
        let share_g = if eg_bound_fcx {
            fun_ds_share_g(&pb.my_data_structure, ifor, si, gi, &topods::edge(esp_on)?)?
        } else if eg_bound_for {
            fun_ds_share_g(&pb.my_data_structure, si, ifor, gi, &topods::edge(esp_on)?)?
        } else {
            None
        };
        let Some(share_g) = share_g else {
            return Ok(());
        };
        let b = if pb.opefus() || pb.opecom() {
            share_g
        } else if pb.opec12() || pb.opec21() {
            !share_g && tbeg == State::Out
        } else {
            false
        };
        if !b {
            return Ok(());
        }
        let Some(neworie) = self.on2d_orientation(esp_on, &eg, eg_bound_for, ifor, si, oeg_fcxf)?
        else {
            return Ok(());
        };
        self.add_start(&esp_on.oriented(neworie))
    }
}
