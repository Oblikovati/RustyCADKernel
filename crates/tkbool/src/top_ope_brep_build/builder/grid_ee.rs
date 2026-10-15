//! The edge merges of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_GridEE.cxx): the pave
//! sets of the edges filled from their point and vertex interferences.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::globals::{GLOBAL_FACES2D, GLOBAL_ISSP, THE_IN, THE_ON, THE_OUT};
use crate::top_ope_brep_build::{
    TopOpeBRepBuildEdgeBuilder, TopOpeBRepBuildGTopo, TopOpeBRepBuildPaveClassifier,
    TopOpeBRepBuildPaveSet,
};
use crate::top_ope_brep_ds::export::{
    fds_assign, fds_idata, fds_parameter, fun_ds_getoov_hds, fun_ds_has_i2d,
};
use crate::top_ope_brep_ds::process_interferences_tool::{
    fun_select_sk_interference, fun_select_trasha_interference, make_epv_interference,
};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSEdgeVertexInterference, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSPointIterator, TopOpeBRepDSTKI,
    TopOpeBRepDSTransition, fun_ds_get_tr,
};
use crate::top_ope_brep_tool::{TopOpeBRepToolShapeTool, TopOpeBRepToolTOOL};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// `TopOpeBRepBuild_FUN_aresamegeom(S1, S2)` (TopOpeBRepBuild_GridFF.cxx): the forward
/// shapes are same oriented.
pub(crate) fn fun_aresamegeom(s1: &Shape, s2: &Shape) -> Result<bool, Failure> {
    TopOpeBRepToolShapeTool::shapes_same_oriented(
        &s1.oriented(Orientation::Forward),
        &s2.oriented(Orientation::Forward),
    )
}

/// `FUN_MoreSHAINT(EPit, SHA)` (static): the iterator moved to the next interference of
/// transition on `sha` before and after; whether one is found.
fn fun_more_shaint(epit: &mut TopOpeBRepDSPointIterator, sha: ShapeEnum) -> Result<bool, Failure> {
    while epit.more() {
        let i = epit.value()?.clone();
        let t = i.get().ok_or_else(null_interference)?.transition();
        if t.shape_before() != sha || t.shape_after() != sha {
            epit.next()?;
        } else {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Standard_NullObject of a null interference.
pub(super) fn null_interference() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "null interference",
        "TopOpeBRepBuild_Builder",
    )
}

/// The GBound of an edge/vertex interference (false for another interference).
fn evi_gbound(i: &Handle<TopOpeBRepDSInterference>) -> bool {
    let evi: Handle<TopOpeBRepDSEdgeVertexInterference> = i.clone().cast();
    evi.get().is_some_and(|e| e.gbound())
}

/// `FUN_isonbound(HDS, I)` (static): a VERTEX geometry bound of its edge/vertex
/// interference and without same domain vertex.
fn fun_isonbound(hds: &TopOpeBRepDSHDataStructure, i: &Handle<TopOpeBRepDSInterference>) -> bool {
    let Some(ii) = i.get() else {
        return false;
    };
    if ii.geometry_type() != TopOpeBRepDSKind::Vertex {
        return false;
    }
    let vhassd = hds.has_same_domain(hds.ds().shape(ii.geometry(), true), true);
    evi_gbound(i) && !vhassd
}

impl TopOpeBRepBuildBuilder {
    /// `GMergeEdges(LE1, LE2, G1)`: the edges of the paves of the two lists, the merged
    /// edges of all of them; `b.g_merge_edges(&le1, &le2, &g)?`.
    pub fn g_merge_edges(
        &mut self,
        le1: &List<Shape>,
        le2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
    ) -> Result<(), Failure> {
        let Some(e1) = le1.first().cloned() else {
            return Ok(());
        };
        let (tb1, tb2) = g1.states_on()?;
        self.my_edge_reference = topods::edge(&e1)?;
        let mut pvs = TopOpeBRepBuildPaveSet::new(&e1)?;
        self.g_fill_edges_pvs(le1, le2, g1, &mut pvs)?;
        let e1f = e1.oriented(Orientation::Forward);
        let mut vcl = TopOpeBRepBuildPaveClassifier::new(&e1f)?;
        if pvs.has_equal_parameters()? {
            vcl.set_first_parameter(pvs.equal_parameters());
        }
        let mut edbu = TopOpeBRepBuildEdgeBuilder::with_paves(&mut pvs, &mut vcl, false)?;
        let mut lem = List::default();
        self.gedbu_make_edges(&e1f, &mut edbu, &mut lem)?;
        for e11 in le1.iter() {
            *self.change_merged(e11, tb1) = lem.clone();
        }
        for e2 in le2.iter() {
            *self.change_merged(e2, tb2) = lem.clone();
        }
        Ok(())
    }

    /// `GFillEdgesPVS(LE1, LE2, G1, PVS)`: the paves of the edges not merged yet of the
    /// first list (on G1) and of the second (on G1 permuted);
    /// `b.g_fill_edges_pvs(&le1, &le2, &g, &mut pvs)?`.
    pub fn g_fill_edges_pvs(
        &mut self,
        le1: &List<Shape>,
        le2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        pvs: &mut TopOpeBRepBuildPaveSet,
    ) -> Result<(), Failure> {
        let Some(e1) = le1.first() else {
            return Ok(());
        };
        let (tb1, tb2) = g1.states_on()?;
        self.my_edge_reference = topods::edge(e1)?;
        for e11 in le1.iter() {
            if !self.is_merged(e11, tb1) {
                self.g_fill_edge_pvs(e11, le2, g1, pvs)?;
            }
        }
        let g2 = g1.copy_permuted();
        for e2 in le2.iter() {
            if !self.is_merged(e2, tb2) {
                self.g_fill_edge_pvs(e2, le1, &g2, pvs)?;
            }
        }
        Ok(())
    }

    /// `GFillEdgePVS(E, LE2, G, PVS)`: the paves of the forward edge;
    /// `b.g_fill_edge_pvs(&e, &le2, &g, &mut pvs)?`.
    pub fn g_fill_edge_pvs(
        &mut self,
        e: &Shape,
        _le2: &List<Shape>,
        g: &TopOpeBRepBuildGTopo,
        pvs: &mut TopOpeBRepBuildPaveSet,
    ) -> Result<(), Failure> {
        g.states_on()?;
        self.g_fill_point_topology_pvs(&e.oriented(Orientation::Forward), g, pvs)
    }

    /// The interferences of the edge to scan: all for a split of a section edge, else
    /// those of transition on the first type of the operation.
    fn shaint_interferences(
        lie: &ListOfInterference,
        scanall: bool,
        shape_interf: ShapeEnum,
    ) -> Result<ListOfInterference, Failure> {
        let mut loi = ListOfInterference::default();
        if scanall {
            fds_assign(lie, &mut loi);
            return Ok(loi);
        }
        let mut epit = TopOpeBRepDSPointIterator::new(lie)?;
        while fun_more_shaint(&mut epit, shape_interf)? {
            loi.append(epit.value()?.clone());
            epit.next()?;
        }
        Ok(loi)
    }

    /// The vertex geometry of a closed edge on its closing vertex (0 when none or when
    /// it is the only geometry).
    fn closing_vertex_geometry(&self, e: &Edge, tki: &mut TopOpeBRepDSTKI) -> Result<i32, Failure> {
        let Some(vclo) = TopOpeBRepToolTOOL::closed_e(e)? else {
            return Ok(0);
        };
        let (mut kp1, mut ng) = (0, 0);
        tki.init();
        while tki.more() {
            ng += 1;
            let (kcur, gcur) = tki
                .value()
                .map(|(k, g, _)| (k, g))
                .ok_or_else(null_interference)?;
            if kcur != TopOpeBRepDSKind::Point {
                let v = self.hds()?.ds().shape(gcur, true).clone();
                let oov = fun_ds_getoov_hds(&v, &self.my_data_structure)?.unwrap_or_default();
                if v.is_same(vclo.as_ref()) || oov.is_same(vclo.as_ref()) {
                    kp1 = gcur;
                }
            }
            tki.next();
        }
        Ok(if ng == 1 { 0 } else { kp1 })
    }

    /// Whether a FACE interference FORWARD or REVERSED on IN is not on a bound.
    fn has_unbound_face_interference(&self, loi: &ListOfInterference) -> Result<bool, Failure> {
        let hds = self.hds()?;
        for i in loi.iter() {
            let ii = i.get().ok_or_else(null_interference)?;
            if ii.support_type() != TopOpeBRepDSKind::Face {
                continue;
            }
            let o = ii.transition().orientation(State::In, ShapeEnum::Face);
            if (o == Orientation::Forward || o == Orientation::Reversed) && !fun_isonbound(hds, i) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The transition of a section edge split on the state of `GLOBAL_issp` at a
    /// geometry from its states before and after (None to skip the geometry).
    fn section_split_transition(
        isfafa: bool,
        b: (State, i32, i32),
        a: (State, i32, i32),
    ) -> Option<TopOpeBRepDSTransition> {
        let ((stb, isb, bdim), (sta, isa, adim)) = (b, a);
        let issp = GLOBAL_ISSP.get();
        let bin = |d: i32| stb == State::In && bdim == d;
        let ain = |d: i32| sta == State::In && adim == d;
        let (bout, aout) = (
            |d: i32| stb == State::Out && bdim == d,
            |d: i32| sta == State::Out && adim == d,
        );
        let mut new_t = TopOpeBRepDSTransition::default();
        let (inb, ina);
        let both_out = stb == State::Out && sta == State::Out;
        let shape = |t: &mut TopOpeBRepDSTransition, s: ShapeEnum| {
            t.set_shape_before(s);
            t.set_shape_after(s);
        };
        if isfafa {
            match issp {
                THE_ON if both_out => return None,
                THE_ON => {
                    (inb, ina) = (bin(1), ain(1));
                    new_t.set_index(isb);
                    shape(&mut new_t, ShapeEnum::Edge);
                }
                THE_IN => {
                    (inb, ina) = (bin(2), ain(2));
                    shape(&mut new_t, ShapeEnum::Face);
                }
                THE_OUT => {
                    (inb, ina) = (!bout(2), !aout(2));
                    shape(&mut new_t, ShapeEnum::Face);
                }
                _ => (inb, ina) = (false, false),
            }
        } else {
            match issp {
                THE_ON if both_out => return None,
                THE_ON => {
                    (inb, ina) = (bin(1) || bin(2), ain(1) || ain(2));
                    new_t.set_index(isb);
                    shape(&mut new_t, ShapeEnum::Edge);
                }
                THE_IN if both_out => return None,
                THE_IN => {
                    (inb, ina) = (bin(3), ain(3));
                    new_t.set_index(if inb { isb } else { isa });
                    shape(&mut new_t, ShapeEnum::Face);
                }
                THE_OUT if stb == State::In && sta == State::In => return None,
                THE_OUT => {
                    (inb, ina) = (!bout(3), !aout(3));
                    new_t.set_index(if bout(3) { isb } else { isa });
                    shape(&mut new_t, ShapeEnum::Face);
                }
                _ => (inb, ina) = (false, false),
            }
        }
        new_t.set_state_before(if inb { State::In } else { State::Out });
        new_t.set_state_after(if ina { State::In } else { State::Out });
        Some(new_t)
    }

    /// The paves of one geometry of a non-section edge: the kept interferences of the
    /// group (the 3d one with 2d ones in 2d, a FACE support in 3d, all on a degenerated
    /// edge), the first one only on a non-degenerated edge.
    #[allow(clippy::too_many_arguments)]
    fn fill_geometry_paves(
        &self,
        e: &Shape,
        licur: &ListOfInterference,
        l3dfcur: &ListOfInterference,
        n2d: i32,
        n3d: i32,
        flags: (bool, bool, bool),
        g: &TopOpeBRepBuildGTopo,
        pvs: &mut TopOpeBRepBuildPaveSet,
    ) -> Result<(), Failure> {
        let (dg_e, is_se, kp6) = flags;
        let hds = self.hds()?;
        let mut itcur = TopOpeBRepDSPointIterator::new(licur)?;
        while itcur.more() {
            let i1 = itcur.value()?.clone();
            let (_, (_, _, st1, _)) = fds_idata(&i1);
            let keepinterf1 = if dg_e {
                true
            } else if GLOBAL_FACES2D.get() {
                if n2d > 0 && n3d > 0 && !is_se {
                    l3dfcur.first().is_some_and(|i3d| *i3d == i1)
                } else {
                    true
                }
            } else {
                st1 == TopOpeBRepDSKind::Face
            };
            if keepinterf1 {
                if !kp6 || !fun_isonbound(hds, &i1) {
                    self.g_fill_point_topology_pvs_at(e, &itcur, g, pvs)?;
                }
                if !dg_e {
                    break;
                }
            }
            itcur.next()?;
        }
        Ok(())
    }

    /// `GFillPointTopologyPVS(E, G, PVS)`: the paves of the edge from its point and
    /// vertex interferences grouped by geometry: on a section edge being split (by
    /// `GLOBAL_issp`), one interference of the transition of its states, else the kept
    /// interferences (but the closing vertex of a closed edge and, when a FACE
    /// interference is not on a bound, those on a bound);
    /// `b.g_fill_point_topology_pvs(&e, &g, &mut pvs)?`.
    pub fn g_fill_point_topology_pvs(
        &mut self,
        e: &Shape,
        g: &TopOpeBRepBuildGTopo,
        pvs: &mut TopOpeBRepBuildPaveSet,
    ) -> Result<(), Failure> {
        let (t1, _) = g.type_();
        let hds = self.my_data_structure.clone();
        let h = hds.get().ok_or_else(null_interference)?;
        let ee = topods::edge(e)?;
        let (ieds, is_se, isfafa, lie) = {
            let bds = h.ds();
            (
                bds.shape_index(e, true),
                bds.is_section_edge(&ee, true),
                bds.isfafa(),
                bds.shape_interferences(e, true).clone(),
            )
        };
        let dg_e = BRepTool::degenerated(&ee)?;
        let issp = GLOBAL_ISSP.get();
        if is_se && issp == 0 {
            return Ok(());
        }
        let scanall = issp == THE_IN || issp == THE_OUT || issp == THE_ON;
        let loishaint = Self::shaint_interferences(&lie, scanall, t1)?;
        let mut tki = TopOpeBRepDSTKI::new();
        tki.fill_on_geometry(&loishaint)?;
        let kp1 = self.closing_vertex_geometry(&ee, &mut tki)?;
        let kp6 = !is_se && self.has_unbound_face_interference(&loishaint)?;
        tki.init();
        while tki.more() {
            let (kcur, gcur, licur) = tki
                .value()
                .map(|(k, gg, l)| (k, gg, l.clone()))
                .ok_or_else(null_interference)?;
            if kcur == TopOpeBRepDSKind::Vertex && kp1 == gcur {
                tki.next();
                continue;
            }
            let i = licur.first().cloned().ok_or_else(null_interference)?;
            let par_se = fds_parameter(&i)?;
            let (mut copy, mut l3dfcur) =
                (ListOfInterference::default(), ListOfInterference::default());
            fds_assign(&licur, &mut copy);
            let n3d = fun_select_sk_interference(&mut copy, TopOpeBRepDSKind::Face, &mut l3dfcur)?;
            let mut l2dfecur = ListOfInterference::default();
            let n2d = fun_ds_has_i2d(ieds, &licur, &mut l2dfecur)?;
            fds_assign(&licur, &mut copy);
            let mut l1decur = ListOfInterference::default();
            fun_select_trasha_interference(&mut copy, ShapeEnum::Edge, &mut l1decur)?;
            let (b, a) = fun_ds_get_tr(&h.ds(), ieds, gcur, &licur)?;
            if is_se {
                if let Some(new_t) = Self::section_split_transition(isfafa, b, a) {
                    let bb = kcur != TopOpeBRepDSKind::Point && evi_gbound(&i);
                    let new_i = make_epv_interference(&new_t, 0, gcur, par_se, kcur, bb);
                    let mut li = ListOfInterference::default();
                    li.append(new_i);
                    let it_cur = TopOpeBRepDSPointIterator::new(&li)?;
                    self.g_fill_point_topology_pvs_at(e, &it_cur, g, pvs)?;
                }
                tki.next();
                continue;
            }
            self.fill_geometry_paves(e, &licur, &l3dfcur, n2d, n3d, (dg_e, is_se, kp6), g, pvs)?;
            tki.next();
        }
        Ok(())
    }
}
