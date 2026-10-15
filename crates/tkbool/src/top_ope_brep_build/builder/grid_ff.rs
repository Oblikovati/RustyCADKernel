//! The face merges of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_GridFF.cxx): the faces
//! of the same domain merged in 2d through the interferences of the 2d structure, and
//! the wire edge sets of the faces filled with their kept wires and edges.
use super::TopOpeBRepBuildBuilder;
use super::build::null_hds;
use crate::top_ope_brep_build::globals::{GLOBAL_DS2D, GLOBAL_FACES2D};
use crate::top_ope_brep_build::{
    TopOpeBRepBuildFaceBuilder, TopOpeBRepBuildGTopo, TopOpeBRepBuildShapeSet,
    TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::export::{
    fds_idata, fds_parameter, fun_ds_has_fei, fun_ds_mk_ton_fsdm, fun_ds_sdm,
};
use crate::top_ope_brep_ds::process_interferences_tool::fun_reducedoublons;
use crate::top_ope_brep_ds::{
    TopOpeBRepDSConfig, TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSInterferenceTool, TopOpeBRepDSKind,
};
use crate::top_ope_brep_tool::{
    TopOpeBRepToolShapeExplorer, TopOpeBRepToolTOOL, fun_tool_find_ancestor, fun_tool_par_e,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, List, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// The face/edge interference made on the 2d structure when `ok`.
fn add_fei(
    pds2d: &TopOpeBRepDSHDataStructure,
    f: &Shape,
    t: Option<crate::top_ope_brep_ds::TopOpeBRepDSTransition>,
    itra: i32,
    ie: i32,
    bound: bool,
) {
    if let Some(mut new_t) = t {
        new_t.set_index(itra);
        let new_i = TopOpeBRepDSInterferenceTool::make_face_edge_interference(
            &new_t,
            itra,
            ie,
            bound,
            TopOpeBRepDSConfig::SameOriented,
        );
        pds2d.change_ds().add_shape_interference(f, &new_i);
    }
}

/// The faces of the same domain of `f` of the other rank.
fn other_rank_sdm(bds: &TopOpeBRepDSDataStructure, f: &Shape) -> List<Shape> {
    let rk_f = bds.ancestor_rank(f);
    let mut fsdm = List::default();
    for ff in bds.shape_same_domain(f).iter() {
        if bds.ancestor_rank(ff) != rk_f {
            fsdm.append(ff.clone());
        }
    }
    fsdm
}

/// The data of one interference of the edge for FUN_computeLIFfaces2d: the transition
/// face, its index, the parameter, the support edge and its index; None to stop.
type LifData = (Face, i32, f64, Edge, i32);

/// The transition face of an EDGE interference of the edge (the face of the same
/// domain containing its support for an EDGE transition); None when not EDGE.
fn lif_data(
    bds: &TopOpeBRepDSDataStructure,
    i: &Handle<crate::top_ope_brep_ds::TopOpeBRepDSInterference>,
    fsdm: &List<Shape>,
) -> Result<Option<LifData>, Failure> {
    let ((sb, ib, _, _), (_, _, st, s)) = fds_idata(i);
    if st != TopOpeBRepDSKind::Edge {
        return Ok(None);
    }
    let es = topods::edge(bds.shape(s, true))?;
    let (ftra, itra) = match sb {
        ShapeEnum::Face => (topods::face(bds.shape(ib, true))?, ib),
        ShapeEnum::Edge => {
            let Some(f) = fun_tool_find_ancestor(fsdm, &es)? else {
                return Ok(None);
            };
            let itra = bds.shape_index(f.as_ref(), true);
            (f, itra)
        }
        _ => (Face::default(), ib),
    };
    Ok(Some((ftra, itra, fds_parameter(i)?, es, s)))
}

impl TopOpeBRepBuildBuilder {
    /// `FUN_computeLIFfaces2d(BU, F, E, pDS2d)` (static): the face/edge interferences of
    /// the edge on the faces of the same domain of the face made in the 2d structure
    /// from its EDGE interferences; false at an interference not on an EDGE.
    fn compute_lif_faces2d_edge(
        &self,
        f: &Face,
        e: &Edge,
        pds2d: &TopOpeBRepDSHDataStructure,
    ) -> Result<bool, Failure> {
        let hds = self.my_data_structure.clone();
        let h = hds.get().ok_or_else(null_hds)?;
        let (li, ie, i_f, fsdm) = {
            let bds = h.ds();
            (
                bds.shape_interferences(e.as_ref(), true).clone(),
                bds.shape_index(e.as_ref(), true),
                bds.shape_index(f.as_ref(), true),
                other_rank_sdm(&bds, f.as_ref()),
            )
        };
        let mut hassp_e =
            self.is_split(e.as_ref(), State::On) && !self.splits(e.as_ref(), State::On).is_empty();
        let mut ffound: Map<Shape, ShapeMapHasher> = Map::default();
        for i in li.iter() {
            let Some((ftra, itra, par_e, es, s)) = lif_data(&h.ds(), i, &fsdm)? else {
                return Ok(false);
            };
            let found = ffound.contains(ftra.as_ref());
            let hassp_es = self.is_split(es.as_ref(), State::On);
            if hassp_es {
                hassp_e = !self.splits(es.as_ref(), State::On).is_empty();
            }
            let sdm = fun_ds_sdm(&h.ds(), e.as_ref(), es.as_ref());
            let has_f_e = fun_ds_has_fei(&pds2d.ds(), f.as_ref(), ie, itra)?;
            if sdm && hassp_e && !found && !has_f_e {
                ffound.add(ftra.clone().into_shape());
                let t = fun_ds_mk_ton_fsdm(&hds, i_f, itra, s, ie, par_e, &Edge::default(), true)?;
                add_fei(pds2d, f.as_ref(), t, itra, ie, true);
            }
            let has_ftra_e = fun_ds_has_fei(&pds2d.ds(), ftra.as_ref(), ie, i_f)?;
            if sdm && hassp_es && !has_ftra_e {
                let mut par_es = 0.;
                if !fun_tool_par_e(e, par_e, &es, &mut par_es)? {
                    continue;
                }
                let t = fun_ds_mk_ton_fsdm(&hds, itra, i_f, ie, s, par_es, &Edge::default(), true)?;
                add_fei(pds2d, ftra.as_ref(), t, i_f, ie, false);
                let t = fun_ds_mk_ton_fsdm(&hds, itra, i_f, ie, ie, par_e, &Edge::default(), true)?;
                add_fei(pds2d, ftra.as_ref(), t, i_f, s, true);
            }
            if hassp_es && !fun_ds_has_fei(&pds2d.ds(), f.as_ref(), s, itra)? {
                let mut par_es = 0.;
                if !fun_tool_par_e(e, par_e, &es, &mut par_es)? {
                    continue;
                }
                let t = fun_ds_mk_ton_fsdm(&hds, i_f, itra, s, s, par_es, &Edge::default(), true)?;
                add_fei(pds2d, f.as_ref(), t, itra, s, false);
            }
        }
        Ok(true)
    }

    /// `FUN_computeLIFfaces2d(BU, F, pDS2d)` (static): for each edge of the face.
    fn compute_lif_faces2d(
        &self,
        f: &Face,
        pds2d: &TopOpeBRepDSHDataStructure,
    ) -> Result<bool, Failure> {
        for e in Explorer::new(f.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !self.compute_lif_faces2d_edge(f, &topods::edge(&e)?, pds2d)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The 2d structure `GLOBAL_DS2d` (made once), reset with the faces of the two lists
    /// and their face/edge interferences, reduced.
    fn fill_ds2d(
        &self,
        lf1: &List<Shape>,
        lf2: &List<Shape>,
    ) -> Result<Handle<TopOpeBRepDSHDataStructure>, Failure> {
        let pds2d = GLOBAL_DS2D.with_borrow_mut(|g| {
            g.get_or_insert_with(|| Handle::new(TopOpeBRepDSHDataStructure::new()))
                .clone()
        });
        let p = pds2d.get().ok_or_else(null_hds)?;
        p.change_ds().init();
        for (l, rank) in [(lf1, 1), (lf2, 2)] {
            for f in l.iter() {
                p.change_ds().add_shape_with_rank(f, rank);
            }
        }
        for f in lf1.iter().chain(lf2.iter()) {
            self.compute_lif_faces2d(&topods::face(f)?, p)?;
        }
        let n = p.ds().nb_shapes();
        for ii in 1..=n {
            let mut ds = p.change_ds();
            let mut li = std::mem::take(ds.change_shape_interferences_index(ii)?);
            let r = fun_reducedoublons(&mut li, &ds, ii);
            *ds.change_shape_interferences_index(ii)? = li;
            r?;
        }
        Ok(pds2d)
    }

    /// `GMergeFaces(LF1, LF2, G1)`: the faces of the same domain lists built in 2d from
    /// one wire edge set, regularized, as the merged faces of those not merged yet;
    /// `b.g_merge_faces(&lf1, &lf2, &g)?`.
    pub fn g_merge_faces(
        &mut self,
        lf1: &List<Shape>,
        lf2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
    ) -> Result<(), Failure> {
        let Some(f1) = lf1.first().cloned() else {
            return Ok(());
        };
        self.fill_ds2d(lf1, lf2)?;
        let (tb1, tb2) = g1.states_on()?;
        self.my_face_reference = topods::face(&f1)?;
        let mut wes = TopOpeBRepBuildWireEdgeSet::new(&f1)?;
        GLOBAL_FACES2D.set(true);
        let r = self
            .g_fill_faces_wesk(lf1, lf2, g1, &mut wes, 1)
            .and_then(|()| self.g_fill_faces_wesk(lf1, lf2, g1, &mut wes, 3));
        GLOBAL_FACES2D.set(false);
        r?;
        let f1f = f1.oriented(Orientation::Forward);
        let mut fabu = TopOpeBRepBuildFaceBuilder::new();
        fabu.init_face_builder(&mut wes, &f1f, true)?;
        let (mut lfm, mut mwisold) = (List::default(), DataMap::default());
        self.gfabu_make_faces(&f1f, &mut fabu, &mut lfm, &mut mwisold)?;
        let mut new_lfm = List::default();
        self.regularize_faces(&f1f, &lfm, &mut new_lfm)?;
        for (l, tb) in [(lf1, tb1), (lf2, tb2)] {
            for f in l.iter() {
                if !self.is_merged(f, tb) {
                    *self.change_merged(f, tb) = new_lfm.clone();
                }
            }
        }
        Ok(())
    }

    /// `GFillFacesWES(LF1, LF2, G1, WES)`: nothing, as natively;
    /// `b.g_fill_faces_wes(&lf1, &lf2, &g, &mut wes)`.
    pub fn g_fill_faces_wes(
        &mut self,
        _lf1: &List<Shape>,
        _lf2: &List<Shape>,
        _g1: &TopOpeBRepBuildGTopo,
        _wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) {
    }

    /// `FUN_validF1edge(F)` (static): the face has two edges or one closed edge.
    fn fun_valid_f1edge(f: &Shape) -> Result<bool, Failure> {
        let mut m_et: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
            m_et.add(e);
            if m_et.extent() > 2 {
                break;
            }
        }
        match m_et.extent() {
            0 => Ok(false),
            1 => Ok(TopOpeBRepToolTOOL::closed_e(&topods::edge(m_et.find_key(1)?)?)?.is_some()),
            _ => Ok(true),
        }
    }

    /// The faces of the same domain list with the reference face first.
    fn reference_first(&self, llf1: &List<Shape>) -> Result<List<Shape>, Failure> {
        let bds = self.hds()?.ds();
        let mut lf1 = List::default();
        for fcur in llf1.iter() {
            if bds.shape_index(fcur, true) == bds.same_domain_ref(fcur) {
                lf1.prepend(fcur.clone());
            } else {
                lf1.append(fcur.clone());
            }
        }
        Ok(lf1)
    }

    /// `GFillFacesWESMakeFaces(LLF1, LF2, LSO, GM)`: the faces of the same domain lists
    /// (reference first) built from one wire edge set (2d, then the curves, then the
    /// sections), kept valid and on the state, oriented as the reference face, as the
    /// splits of all the faces; `b.g_fill_faces_wes_make_faces(&lf1, &lf2, &lso, &g)?`.
    pub fn g_fill_faces_wes_make_faces(
        &mut self,
        llf1: &List<Shape>,
        lf2: &List<Shape>,
        _lso: &List<Shape>,
        gm: &TopOpeBRepBuildGTopo,
    ) -> Result<(), Failure> {
        let (tb1, tb2) = gm.states_on()?;
        if llf1.is_empty() {
            return Ok(());
        }
        let lf1 = self.reference_first(llf1)?;
        let ff = lf1
            .first()
            .cloned()
            .unwrap_or_default()
            .oriented(Orientation::Forward);
        let iff = self.hds()?.shape_index(&ff, true);
        let mut wes = TopOpeBRepBuildWireEdgeSet::new(&ff)?;
        GLOBAL_FACES2D.set(true);
        let r = self.g_fill_faces_wesk(&lf1, lf2, gm, &mut wes, 1);
        GLOBAL_FACES2D.set(false);
        r?;
        self.g_fill_faces_wesk(&lf1, lf2, gm, &mut wes, 2)?;
        self.g_fill_faces_wesk(&lf1, lf2, gm, &mut wes, 3)?;
        let n = wes.start_elements().len() as i32;
        self.my_edge_avoid.clear();
        Self::g_copy_list_range(
            &wes.start_elements().clone(),
            n + 1,
            n,
            &mut self.my_edge_avoid,
        );
        let mut lof = List::default();
        self.gwes_make_faces(&ff, &mut wes, &mut lof)?;
        let mut valid = List::default();
        for f in lof.iter() {
            if Self::fun_valid_f1edge(f)? {
                valid.append(f.clone());
            }
        }
        let mut lofs = List::default();
        let empty = self.my_empty_shape_list.clone();
        self.g_keep_shapes(&ff, &empty, tb1, &valid, &mut lofs)?;
        let odsff = self.hds()?.shape(iff, true).orientation();
        let lofs: List<Shape> = lofs.iter().map(|f| f.oriented(odsff)).collect();
        for (l, tb) in [(&lf1, tb1), (lf2, tb2)] {
            for s in l.iter() {
                self.mark_split(s, tb, true);
                Self::g_copy_list(&lofs, self.change_split(s, tb));
            }
        }
        Ok(())
    }

    /// `GFillFaceWES(FOR1, LFclass, G1, WES)`: the wires of the forward face out of the
    /// structure added when kept (ON ones remembered when processing ON), the others
    /// filled by their edges; nothing for a same domain face in the common of a cut;
    /// `b.g_fill_face_wes(&f, &lc, &g, &mut wes)?`.
    pub fn g_fill_face_wes(
        &mut self,
        for1: &Shape,
        lfclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let (tb1, tb2) = g1.states_on()?;
        let rev_ori1 = g1.is_to_reverse1()?;
        let com_of_cut = (self.opec12() || self.opec21()) && tb1 == tb2 && tb1 == State::In;
        if self.hds()?.has_same_domain(for1, true) && com_of_cut {
            return Ok(());
        }
        let ff = for1.oriented(Orientation::Forward);
        self.my_face_to_fill = topods::face(&ff)?;
        let mut ex_wire = TopOpeBRepToolShapeExplorer::new(&ff, ShapeEnum::Wire, ShapeEnum::Shape)?;
        while ex_wire.more() {
            let w = ex_wire.current().clone();
            if !self.hds()?.has_shape(&w, true) {
                let (keep, pos) = self.g_keep_shape1(&w, lfclass, tb1)?;
                if keep {
                    wes.add_shape(&w.oriented(Self::orient(w.orientation(), rev_ori1)))?;
                } else if self.my_process_on && pos == State::On {
                    self.my_on_elem_map.add(w);
                }
            } else {
                self.g_fill_wire_wes(&w, lfclass, g1, wes)?;
            }
            ex_wire.next()?;
        }
        Ok(())
    }

    /// `GFillWireWES(W, LSclass, G1, WES)`: each edge of the wire filled;
    /// `b.g_fill_wire_wes(&w, &lc, &g, &mut wes)?`.
    pub fn g_fill_wire_wes(
        &mut self,
        w: &Shape,
        lsclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        g1.states_on()?;
        let mut ex_edge = TopOpeBRepToolShapeExplorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ex_edge.more() {
            let eor = ex_edge.current().clone();
            self.g_fill_edge_wes(&eor, lsclass, g1, wes)?;
            ex_edge.next()?;
        }
        Ok(())
    }

    /// `GFillEdgeWES(EOR, LSclass, G1, WES)`: the merged edge, then the split edge on an
    /// unshared configuration; `b.g_fill_edge_wes(&e, &lc, &g, &mut wes)?`.
    pub fn g_fill_edge_wes(
        &mut self,
        eor: &Shape,
        lsclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let (tb1, _) = g1.states_on()?;
        self.g_to_split(eor, tb1)?;
        self.g_to_merge(eor)?;
        self.g_merge_edge_wes(eor, g1, wes)?;
        let mut gse = *g1;
        gse.change_config(
            TopOpeBRepDSConfig::UnshGeometry,
            TopOpeBRepDSConfig::UnshGeometry,
        );
        self.g_split_edge_wes(eor, lsclass, &gse, wes)
    }
}
