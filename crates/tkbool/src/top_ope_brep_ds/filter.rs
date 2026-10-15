// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_Filter.hxx/.cxx,
// TopOpeBRepDS_FilterCurveInterferences.cxx, TopOpeBRepDS_FilterEdgeInterferences.cxx,
// TopOpeBRepDS_FilterFaceInterferences.cxx and TopOpeBRepDS_Reducer.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_Filter` (filtering the interferences of the edges, faces and curves)
//! and `TopOpeBRepDS_Reducer` (running the edge and face interference reducers).
use super::export::{fds_copy, href, iref};
use super::process_edge_interferences::{
    fun_order_statetrans, fun_resolve_e_unknown, fun_unkeep_e_interferences,
    order_ff_samedomain_ds, purge_ds_on_se_ds,
};
use super::process_face_interferences::fun_resolve_f_unknown;
use super::process_interferences_tool::select::{to_list, to_vec};
use super::process_interferences_tool::{
    fun_reducedoublons, fun_select_gk_interference, fun_select_sk_interference,
    fun_select_trasha_interference, fun_select_traunk_interference,
};
use super::tool::MapOfShapeOn1State;
use super::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSEIR, TopOpeBRepDSFIR,
    TopOpeBRepDSHDataStructure, TopOpeBRepDSKind,
};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeClassifier;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

/// The interference filter; `let mut f = TopOpeBRepDSFilter::new(&hds, None);`.
pub struct TopOpeBRepDSFilter<'a> {
    my_hds: Handle<TopOpeBRepDSHDataStructure>,
    my_pshape_classif: Option<&'a mut TopOpeBRepToolShapeClassifier>,
}

/// `FUN_FilterEdge(LI, HDS, SIX)` (static): the interferences ordered (same domain
/// faces first, then by state per geometry), the unkept and the doubles removed.
fn fun_filter_edge(
    li: &mut ListOfInterference,
    bds: &TopOpeBRepDSDataStructure,
    six: i32,
) -> Result<(), Failure> {
    order_ff_samedomain_ds(li, bds)?;
    fun_order_statetrans(li)?;
    fun_unkeep_e_interferences(li, bds, six)?;
    fun_reducedoublons(li, bds, six)
}

/// Whether the shape of index `i` is not null and of type `t`.
fn is_of_type(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    i: i32,
    t: ShapeEnum,
) -> Result<bool, Failure> {
    let bds = href(hds)?.ds();
    let s = bds.shape(i, true);
    Ok(!s.is_null() && s.shape_type()? == t)
}

impl<'a> TopOpeBRepDSFilter<'a> {
    /// `TopOpeBRepDS_Filter(HDS, pClassif)`; `TopOpeBRepDSFilter::new(&hds, None)`.
    pub fn new(
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        p_classif: Option<&'a mut TopOpeBRepToolShapeClassifier>,
    ) -> Self {
        Self {
            my_hds: hds.clone(),
            my_pshape_classif: p_classif,
        }
    }

    /// `ProcessInterferences()`: the edge then the curve interferences filtered;
    /// `f.process_interferences()?`.
    pub fn process_interferences(&mut self) -> Result<(), Failure> {
        self.process_edge_interferences()?;
        self.process_curve_interferences()
    }

    /// `ProcessEdgeInterferences()`; `f.process_edge_interferences()?`.
    pub fn process_edge_interferences(&mut self) -> Result<(), Failure> {
        let nshape = href(&self.my_hds)?.ds().nb_shapes();
        for i in 1..=nshape {
            if is_of_type(&self.my_hds, i, ShapeEnum::Edge)? {
                self.process_edge_interferences_index(i)?;
            }
        }
        Ok(())
    }

    /// `ProcessFaceInterferences(MEsp)`; `f.process_face_interferences(&mesp)?`.
    pub fn process_face_interferences(&mut self, mesp: &MapOfShapeOn1State) -> Result<(), Failure> {
        let nshape = href(&self.my_hds)?.ds().nb_shapes();
        for i in 1..=nshape {
            if is_of_type(&self.my_hds, i, ShapeEnum::Face)? {
                self.process_face_interferences_index(i, mesp)?;
            }
        }
        Ok(())
    }

    /// `ProcessCurveInterferences()`; `f.process_curve_interferences()?`.
    pub fn process_curve_interferences(&mut self) -> Result<(), Failure> {
        let ncurve = href(&self.my_hds)?.ds().nb_curves();
        for i in 1..=ncurve {
            self.process_curve_interferences_index(i)?;
        }
        Ok(())
    }

    /// `ProcessCurveInterferences(CIX)`: the later VERTEX interferences of a vertex of
    /// the same orientation removed; `f.process_curve_interferences_index(1)?`.
    pub fn process_curve_interferences_index(&mut self, cix: i32) -> Result<(), Failure> {
        let mut bds = href(&self.my_hds)?.change_ds();
        let li = bds.change_curve_interferences(cix);
        let mut v = to_vec(li);
        let mut a = 0;
        while a < v.len() {
            let i1 = iref(&v[a])?;
            let (g1, gt1) = (i1.geometry(), i1.geometry_type());
            let o1 = i1.transition().orientation(State::In, ShapeEnum::Face);
            if gt1 == TopOpeBRepDSKind::Vertex {
                let mut b = a + 1;
                while b < v.len() {
                    let i2 = iref(&v[b])?;
                    let o2 = i2.transition().orientation(State::In, ShapeEnum::Face);
                    if i2.geometry_type() == gt1 && i2.geometry() == g1 && o1 == o2 {
                        v.remove(b);
                    } else {
                        b += 1;
                    }
                }
            }
            a += 1;
        }
        *li = to_list(v);
        Ok(())
    }

    /// `ProcessEdgeInterferences(SIX)`: the interferences of the edge purged on a
    /// section edge, the unknown ones resolved, then the FACE and EDGE ones filtered
    /// (the others dropped); `f.process_edge_interferences_index(3)?`.
    pub fn process_edge_interferences_index(&mut self, six: i32) -> Result<(), Failure> {
        let mut bds = href(&self.my_hds)?.change_ds();
        let mut lall = bds.change_shape_interferences_index(six)?.clone();
        purge_ds_on_se_ds(&bds, six, &mut lall)?;
        let mut luu = ListOfInterference::default();
        fun_select_traunk_interference(&mut lall, &mut luu)?;
        fun_resolve_e_unknown(&mut luu, &mut bds, six)?;
        let mut lw = lall;
        fds_copy(&luu, &mut lw);
        let (mut lf, mut le) = (ListOfInterference::default(), ListOfInterference::default());
        fun_select_trasha_interference(&mut lw, ShapeEnum::Face, &mut lf)?;
        fun_select_trasha_interference(&mut lw, ShapeEnum::Edge, &mut le)?;
        fun_filter_edge(&mut lf, &bds, six)?;
        fun_filter_edge(&mut le, &bds, six)?;
        let li = bds.change_shape_interferences_index(six)?;
        li.clear();
        fds_copy(&lf, li);
        fds_copy(&le, li);
        Ok(())
    }

    /// `ProcessFaceInterferences(SIX, MEsp)`: the doubles removed, the unknown ones
    /// resolved, then the FACE (with their EDGE and FACE/EDGE parts) and EDGE ones kept;
    /// `f.process_face_interferences_index(1, &mesp)?`.
    pub fn process_face_interferences_index(
        &mut self,
        six: i32,
        mesp: &MapOfShapeOn1State,
    ) -> Result<(), Failure> {
        let mut bds = href(&self.my_hds)?.change_ds();
        let mut li = bds.change_shape_interferences_index(six)?.clone();
        fun_reducedoublons(&mut li, &bds, six)?;
        *bds.change_shape_interferences_index(six)? = li.clone();
        let mut lall = li;
        let mut luu = ListOfInterference::default();
        fun_select_traunk_interference(&mut lall, &mut luu)?;
        let classif = self.my_pshape_classif.as_deref_mut();
        fun_resolve_f_unknown(&mut luu, &mut bds, six, mesp, classif)?;
        let mut lw = lall;
        fds_copy(&luu, &mut lw);
        let mut l = [(); 4].map(|_| ListOfInterference::default());
        let [lf, lfe, lfef, le] = &mut l;
        fun_select_trasha_interference(&mut lw, ShapeEnum::Face, lf)?;
        fun_select_gk_interference(lf, TopOpeBRepDSKind::Edge, lfe)?;
        fun_select_sk_interference(lfe, TopOpeBRepDSKind::Face, lfef)?;
        fun_select_trasha_interference(&mut lw, ShapeEnum::Edge, le)?;
        let li = bds.change_shape_interferences_index(six)?;
        li.clear();
        for part in [&*lf, &*lfe, &*lfef, &*le] {
            fds_copy(part, li);
        }
        Ok(())
    }
}

/// The interference reducer; `let r = TopOpeBRepDSReducer::new(&hds);`.
pub struct TopOpeBRepDSReducer {
    my_hds: Handle<TopOpeBRepDSHDataStructure>,
}

impl TopOpeBRepDSReducer {
    /// `TopOpeBRepDS_Reducer(HDS)`; `TopOpeBRepDSReducer::new(&hds)`.
    pub fn new(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Self {
        Self {
            my_hds: hds.clone(),
        }
    }

    /// `ProcessEdgeInterferences()`: `TopOpeBRepDS_EIR`; `r.process_edge_interferences()?`.
    pub fn process_edge_interferences(&self) -> Result<(), Failure> {
        TopOpeBRepDSEIR::new(&self.my_hds).process_edge_interferences()
    }

    /// `ProcessFaceInterferences(M)`: `TopOpeBRepDS_FIR`; `r.process_face_interferences(&m)?`.
    pub fn process_face_interferences(&self, m: &MapOfShapeOn1State) -> Result<(), Failure> {
        TopOpeBRepDSFIR::new(&self.my_hds).process_face_interferences(m)
    }
}
