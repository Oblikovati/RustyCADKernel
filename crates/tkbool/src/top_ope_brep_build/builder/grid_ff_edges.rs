//! The edges of the wire edge sets of `TopOpeBRepBuild_Builder`
//! (TopOpeBRepBuild_GridFF.cxx): the splits and merges of the edges of the faces to
//! fill, oriented in the reference face.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::globals::GLOBAL_CLASSIFYSPLITEDGE;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildGTopo, TopOpeBRepBuildPaveSet, TopOpeBRepBuildShapeSet,
    TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::export::{
    ORIEINF_FORREVOPPO, ORIEINF_ONOPPOSHA, ORIEINF_ONSAMESHA, fun_ds_ori_e_in_f,
};
use crate::top_ope_brep_ds::{TopOpeBRepDSConfig, TopOpeBRepDSDataStructure};
use crate::top_ope_brep_tool::{fun_tool_bounds, fun_tool_curves_so_at};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;

/// `FUN_samgeomori(BDS, iref, ifil, samgeomori)` (static): the face to fill is of the
/// geometry and orientation of the reference face.
fn fun_samgeomori(bds: &TopOpeBRepDSDataStructure, iref: i32, ifil: i32) -> bool {
    let cfill = bds.same_domain_ori_index(ifil);
    let (oref, ofil) = (
        bds.shape(iref, true).orientation(),
        bds.shape(ifil, true).orientation(),
    );
    let samgeomori = cfill == TopOpeBRepDSConfig::SameOriented;
    if oref == top_abs::complement(ofil) {
        !samgeomori
    } else {
        samgeomori
    }
}

/// `ClassifyEdgeToSolidByOnePoint(E, Ref)` (static): the state in the solid of the
/// point of the edge at a fixed inner parameter (its first vertex without 3d curve;
/// UNKNOWN without vertex).
fn classify_edge_to_solid_by_one_point(e: &Edge, r: &Shape) -> Result<State, Failure> {
    let par_t = 0.43213918;
    let (c3d, f2, l2) = BRepTool::curve(e)?;
    let p3d = match c3d.get() {
        None => {
            let fv = topexp::first_vertex(e, false);
            if fv.is_null() {
                return Ok(State::Unknown);
            }
            BRepTool::pnt(&fv)?
        }
        Some(c) => c.value(f2 * par_t + (1. - par_t) * l2)?,
    };
    let mut sc = BRepClass3dSolidClassifier::with_shape(r)?;
    sc.perform(&p3d, 1e-7)?;
    Ok(sc.state())
}

impl TopOpeBRepBuildBuilder {
    /// The orientation of the edge in the face to fill (from the reference face when the
    /// face has same domain faces) and the ORIEINF case.
    fn edge_in_fill_face(&self, eor: &Edge) -> Result<(i32, Orientation), Failure> {
        let hds = self.hds()?;
        let bds = hds.ds();
        let fill = self.my_face_to_fill.as_ref();
        if !hds.has_same_domain(fill, true) {
            return fun_ds_ori_e_in_f(&bds, eor, fill);
        }
        let (oinref, mut oeinf) = fun_ds_ori_e_in_f(&bds, eor, self.my_face_reference.as_ref())?;
        let (oinfill, oeinfill) = fun_ds_ori_e_in_f(&bds, eor, fill)?;
        if oinref == oinfill {
            let (ifil, iref) = (
                hds.shape_index(fill, true),
                hds.shape_index(self.my_face_reference.as_ref(), true),
            );
            let mut reverse = false;
            if iref != ifil {
                reverse = !fun_samgeomori(&bds, iref, ifil);
                if self.my_face_reference.orientation() == Orientation::Reversed {
                    reverse = !reverse;
                }
            }
            let to_ref = if reverse {
                top_abs::complement(oeinfill)
            } else {
                oeinfill
            };
            if oeinf != to_ref && oeinf != Orientation::Internal && oeinf != Orientation::External {
                oeinf = to_ref;
            }
        }
        Ok((oinref, oeinf))
    }

    /// A split edge oriented in the face (reversed when its curve is opposite to the
    /// edge with same domain edges), or with the new orientation.
    fn oriented_split(
        &self,
        sp: &Shape,
        eor: &Edge,
        new_o: Option<Orientation>,
        hsdm_e: bool,
        neworie: Orientation,
    ) -> Result<Shape, Failure> {
        let Some(oeinf) = new_o else {
            return Ok(sp.oriented(neworie));
        };
        let mut new_e = topods::edge(&sp.oriented(oeinf))?;
        if !BRepTool::degenerated(eor)? && hsdm_e {
            let (f, l) = fun_tool_bounds(&new_e)?;
            let x = 0.45678;
            let mut so = true;
            fun_tool_curves_so_at(&new_e, (1. - x) * f + x * l, eor, &mut so)?;
            if !so {
                new_e = topods::edge(&new_e.reversed())?;
            }
        }
        Ok(new_e.into_shape())
    }

    /// The splits of the edge on the state kept by the classification of a point in the
    /// other solid (particular case 4).
    fn keep_kpart4_splits(&mut self, eor: &Shape, tb1: State) -> Result<(), Failure> {
        let a_solid = if self.shape_rank(eor) == 1 {
            self.my_shape2.clone()
        } else {
            self.my_shape1.clone()
        };
        let splits = self.change_split(eor, tb1).clone();
        let mut aux = List::default();
        for e in splits.iter() {
            if classify_edge_to_solid_by_one_point(&topods::edge(e)?, &a_solid)? == tb1 {
                aux.append(e.clone());
            }
        }
        *self.change_split(eor, tb1) = aux;
        Ok(())
    }

    /// Whether an unsplit edge is added on the state, and whether it is ON.
    fn unsplit_edge_kept(
        &mut self,
        eor: &Shape,
        lsclass: &List<Shape>,
        tb1: State,
    ) -> Result<(bool, bool), Failure> {
        let hds = self.hds()?;
        let se = hds.ds().is_section_edge(&topods::edge(eor)?, true);
        let (hs, hg) = (hds.has_shape(eor, true), hds.has_geometry(eor));
        if se {
            let first = lsclass.first().ok_or_else(|| {
                Failure::with_type(
                    "Standard_NoSuchObject",
                    "NCollection_List::First",
                    "TopOpeBRepBuild_Builder",
                )
            })?;
            let ftg = first.shape_type()? == ShapeEnum::Face;
            if ftg {
                return Ok((false, false));
            }
            let (keepse, pos) = self.g_keep_shape1(eor, lsclass, tb1)?;
            return Ok((keepse, !keepse && self.my_process_on && pos == State::On));
        }
        if !(hs && !hg) {
            return Ok((true, false));
        }
        let (mut keep, pos) = self.g_keep_shape1(eor, lsclass, tb1)?;
        let mut add_on = false;
        if !keep {
            if lsclass.first().map(|s| s.shape_type()).transpose()? == Some(ShapeEnum::Solid) {
                keep = pos == State::On;
            }
            add_on = self.my_process_on && keep;
        }
        Ok((keep, add_on))
    }

    /// `GSplitEdgeWES(EOR, LSclass, G1, WES)`: the edge split (by the particular case 4
    /// classification too) and its splits on the state, or itself when kept, added to
    /// the wire edge set oriented in the face to fill (not IN of a fuse); the ON parts
    /// remembered when processing ON; `b.g_split_edge_wes(&e, &lc, &g, &mut wes)?`.
    pub fn g_split_edge_wes(
        &mut self,
        eor: &Shape,
        lsclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let (tb1, _) = g1.states_on()?;
        let neworie = Self::orient(eor.orientation(), g1.is_to_reverse1()?);
        let eore = topods::edge(eor)?;
        let hsdm_e = self.hds()?.has_same_domain(eor, true);
        let (oinref, oeinf) = self.edge_in_fill_face(&eore)?;
        let mut new_o = oinref == ORIEINF_ONSAMESHA || oinref == ORIEINF_ONOPPOSHA;
        if self.my_is_k_part == 3 {
            new_o = false;
        }
        let new_o = new_o.then_some(oeinf);
        if self.g_to_split(eor, tb1)? {
            self.g_split_edge(eor, g1, lsclass)?;
            if self.my_is_k_part == 4 {
                self.keep_kpart4_splits(eor, tb1)?;
            }
        }
        if self.opefus() && tb1 == State::In {
            return Ok(());
        }
        if self.is_split(eor, tb1) {
            for sp in self.splits(eor, tb1).clone().iter() {
                wes.add_start_element(&self.oriented_split(sp, &eore, new_o, hsdm_e, neworie)?)?;
            }
        } else {
            let isstart = self.hds()?.has_shape(eor, true);
            let (add, add_on) = self.unsplit_edge_kept(eor, lsclass, tb1)?;
            if add {
                let o = match new_o {
                    Some(o) => o,
                    None if oinref == ORIEINF_FORREVOPPO => Orientation::Internal,
                    None => neworie,
                };
                if isstart {
                    wes.add_start_element(&eor.oriented(o))?
                } else {
                    wes.add_element(&eor.oriented(o))?
                }
            }
            if add_on {
                self.my_on_elem_map.add(eor.oriented(neworie));
            }
        }
        if self.my_process_on && self.is_split(eor, State::On) {
            for sp in self.splits(eor, State::On).clone().iter() {
                let e = self.oriented_split(sp, &eore, new_o, hsdm_e, neworie)?;
                self.my_on_elem_map.add(e);
            }
        }
        Ok(())
    }

    /// `GMergeEdgeWES(EOR, G1, WES)`: for a non-closing edge with same domain edges (not
    /// in a fuse, not OUT), its merged edges (made from its splits by the reference
    /// edge) added as start elements; `b.g_merge_edge_wes(&e, &g, &mut wes)?`.
    pub fn g_merge_edge_wes(
        &mut self,
        eor: &Shape,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let eore = topods::edge(eor)?;
        if BRepTool::is_closed_on_face(&eore, &self.my_face_to_fill)? || self.opefus() {
            return Ok(());
        }
        let (tb1, tb2) = g1.states_on()?;
        let neworie = Self::orient(eor.orientation(), g1.is_to_reverse1()?);
        let hds = self.hds()?;
        if !hds.has_same_domain(eor, true) {
            return Ok(());
        }
        let eref = hds.shape(hds.same_domain_reference(eor), true);
        let eisref = eor.is_same(&eref);
        let tbeor = if eisref { tb1 } else { tb2 };
        if tbeor == State::Out {
            return Ok(());
        }
        if self.is_merged(eor, tbeor) {
            if eisref {
                for e in self.merged(eor, tbeor).clone().iter() {
                    wes.add_start_element(&e.oriented(neworie))?;
                }
            }
            return Ok(());
        }
        *self.change_merged(eor, tbeor) = self.my_empty_shape_list.clone();
        if !self.is_split(eor, tbeor) {
            return Ok(());
        }
        let me = self.splits(eor, tbeor).clone();
        *self.change_merged(eor, tbeor) = me.clone();
        for e in me.iter() {
            wes.add_start_element(&e.oriented(neworie))?;
        }
        Ok(())
    }

    /// The shapes to classify the split edges with: the other shape of the operation
    /// (classifying split edges), the shapes to classify unless on solids, or none.
    fn split_edge_classes(
        &self,
        eor: &Shape,
        lsclass: &List<Shape>,
    ) -> Result<List<Shape>, Failure> {
        if GLOBAL_CLASSIFYSPLITEDGE.get() {
            let oos = if self.g_shape_rank(eor) == 1 {
                &self.my_shape2
            } else {
                &self.my_shape1
            };
            let mut loos = List::default();
            if !oos.is_null() {
                loos.append(oos.clone());
            }
            return Ok(loos);
        }
        let onsolid =
            lsclass.first().map(|s| s.shape_type()).transpose()? == Some(ShapeEnum::Solid);
        Ok(if onsolid {
            self.my_empty_shape_list.clone()
        } else {
            lsclass.clone()
        })
    }

    /// `GSplitEdge(EOR, G1, LSclass)`: the forward edge split by its paves, the parts
    /// kept on the state as its splits (the ON parts as its ON splits when processing
    /// ON); `b.g_split_edge(&e, &g, &lc)?`.
    pub fn g_split_edge(
        &mut self,
        eor: &Shape,
        g1: &TopOpeBRepBuildGTopo,
        lsclass: &List<Shape>,
    ) -> Result<(), Failure> {
        let (tb1, _) = g1.states_on()?;
        let ef = eor.oriented(Orientation::Forward);
        let mut loe = List::default();
        let mut pvs = TopOpeBRepBuildPaveSet::new(&ef)?;
        self.my_edge_reference = topods::edge(&ef)?;
        self.g_fill_point_topology_pvs(&ef, g1, &mut pvs)?;
        self.mark_split(&ef, tb1, true);
        self.gpvs_make_edges(&ef, &mut pvs, &mut loe)?;
        if loe.is_empty() {
            return Ok(());
        }
        self.change_split(&ef, tb1).clear();
        let pls = self.split_edge_classes(eor, lsclass)?;
        let mut alon = List::default();
        for ae in loe.iter() {
            let (keep, pos) = self.g_keep_shape1(ae, &pls, tb1)?;
            if keep {
                self.change_split(&ef, tb1).append(ae.clone());
            } else if self.my_process_on && pos == State::On {
                alon.append(ae.clone());
            }
        }
        if !alon.is_empty() {
            self.mark_split(&ef, State::On, true);
            let slon = self.change_split(&ef, State::On);
            slon.clear();
            slon.append_list(&mut alon);
        }
        Ok(())
    }
}
