// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_EIR.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_EIR`: the edge interference reducer, rewriting the interferences of
//! each edge of the structure into reduced transitions.
mod recl;
mod reducers;

pub use recl::fun_recl_se;

use super::export::complete_se::{evi_gbound, tki_groups};
use super::export::{fds_copy, fds_data, fds_parameter, fun_ds_getoov, href, iref};
use super::process_interferences_tool::{
    fun_select_sk_interference, fun_select_trasha_interference, fun_select1d_i, fun_select2d_i,
    fun_select3d_interference, fun_selectpure2d_i, make_epv_interference_sk,
};
use super::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure, TopOpeBRepDSKind,
    TopOpeBRepDSTKI,
};
use recl::{fun_keepl3d_f, fun_reducepure2d_i, fun_unkeep_evi_on_gb1};
use reducers::{fun_reducer_edge, fun_reducer_edge3d, fun_reducer_sd_edge};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The edge interference reducer; `let eir = TopOpeBRepDSEIR::new(&hds);`.
pub struct TopOpeBRepDSEIR {
    my_hds: Handle<TopOpeBRepDSHDataStructure>,
}

/// An empty list of interferences.
fn empty() -> ListOfInterference {
    ListOfInterference::default()
}

/// The appending of each list in order.
fn concat(lists: &[&ListOfInterference]) -> ListOfInterference {
    let mut out = empty();
    for l in lists {
        fds_copy(l, &mut out);
    }
    out
}

/// The 2d FACE/EDGE interferences of a section edge reduced (`FUN_selectpure2dI`
/// then `FUN_reducepure2dI`, appended back to `lfe`).
fn reduce_pure2d(lf: &ListOfInterference, lfe: &mut ListOfInterference) -> Result<(), Failure> {
    let (mut l_i2d_fe, mut l_ri2d_fe) = (empty(), empty());
    fun_selectpure2d_i(lf, lfe, &mut l_i2d_fe)?;
    fun_reducepure2d_i(&mut l_i2d_fe, &mut l_ri2d_fe)?;
    fds_copy(&l_i2d_fe, lfe);
    fds_copy(&l_ri2d_fe, lfe);
    Ok(())
}

/// `FUN_ProcessEdgeInterferences(EIX, K, G, HDS, LI)` (static): the interferences of
/// one geometry of the edge `eix` selected by dimension and reduced; the reduced ones
/// first, then the 1d and the face ones (the others are dropped).
fn fun_process_edge_interferences(
    eix: i32,
    bds: &mut TopOpeBRepDSDataStructure,
    li: &mut ListOfInterference,
) -> Result<(), Failure> {
    let e = topods::edge(bds.shape(eix, true))?;
    let (mut lf, mut lfe, mut le) = (empty(), empty(), empty());
    fun_select_trasha_interference(li, ShapeEnum::Face, &mut lf)?;
    fun_select_sk_interference(&mut lf, TopOpeBRepDSKind::Edge, &mut lfe)?;
    fun_select_trasha_interference(li, ShapeEnum::Edge, &mut le)?;
    let isse = bds.is_section_edge(&e, true);
    if isse {
        reduce_pure2d(&lf, &mut lfe)?;
    }
    let mut l = [(); 5].map(|_| empty());
    let [lfe_resi, l3d_fe, l3d_f, l3d_fe_resi, l2d_fe] = &mut l;
    fun_select3d_interference(
        eix,
        bds,
        &mut lf,
        l3d_f,
        &mut lfe,
        lfe_resi,
        l3d_fe,
        l3d_fe_resi,
        l2d_fe,
    )?;
    let (mut l1d_e, mut l1d_esd) = (empty(), empty());
    fun_select2d_i(eix, bds, ShapeEnum::Edge, &mut le, &mut l1d_e)?;
    fun_select1d_i(eix, bds, &mut le, &mut l1d_esd)?;
    let mut lr3d_fe = empty();
    fun_reducer_edge3d(eix, bds, l3d_fe, &mut lr3d_fe)?;
    let mut l3d_fkeep = empty();
    fun_keepl3d_f(l3d_f, &lr3d_fe, &mut l3d_fkeep)?;
    fds_copy(&l3d_fkeep, &mut lf);
    let (mut lr2d_fe, mut lr1d_esd, mut lr1d_closed_se, mut lr1d_e) =
        (empty(), empty(), empty(), empty());
    fun_reducer_edge(eix, bds, l2d_fe, &mut lr2d_fe)?;
    fun_reducer_sd_edge(eix, bds, &mut l1d_esd, &mut lr1d_esd)?;
    fun_recl_se(eix, bds, &mut l1d_e, &mut lr1d_closed_se)?;
    fun_reducer_edge(eix, bds, &mut l1d_e, &mut lr1d_e)?;
    let lri = concat(&[&lr1d_esd, &lr1d_e, &lr1d_closed_se, &lr2d_fe, &lr3d_fe]);
    let lf = concat(&[&lf, &lfe, l3d_fe, l2d_fe]);
    let le = concat(&[&le, &l1d_e, &l1d_esd]);
    *li = concat(&[&lri, &le, &lf]);
    if isse {
        fun_unkeep_evi_on_gb1(bds, eix, li)?;
    }
    Ok(())
}

/// The interferences of `loi` rewritten on the same domain vertex `oog`.
fn rewrite_on_oov(
    k: TopOpeBRepDSKind,
    oog: i32,
    loi: &ListOfInterference,
) -> Result<ListOfInterference, Failure> {
    let mut newloi = empty();
    for i in loi.iter() {
        let (_, _, st, s) = fds_data(i);
        let par = fds_parameter(i)?;
        let t = iref(i)?.transition();
        newloi.append(make_epv_interference_sk(&t, s, oog, par, k, st, false)?);
    }
    Ok(newloi)
}

/// The interferences of the vertices with a same domain vertex merged in `newtki`
/// (`TopOpeBRepDS_EIR::ProcessEdgeInterferences(EIX)` first loop).
fn merge_sd_vertices(
    bds: &TopOpeBRepDSDataStructure,
    li: &ListOfInterference,
    newtki: &mut TopOpeBRepDSTKI,
) -> Result<(), Failure> {
    for (k, g, loi) in tki_groups(li)? {
        if k == TopOpeBRepDSKind::Point {
            continue;
        }
        let Some(oov_g) = fun_ds_getoov(bds.shape(g, true), bds) else {
            continue;
        };
        let oog = bds.shape_index(&oov_g, true);
        if oog == 0 {
            continue;
        }
        let isb = newtki.is_bound(k, oog);
        if evi_gbound(&loi.first().cloned().unwrap_or_default())? {
            let newloi = rewrite_on_oov(k, oog, &loi)?;
            newtki.change_interferences(k, g).clear();
            if !isb {
                newtki.add(k, oog)?;
            }
            fds_copy(&newloi, newtki.change_interferences(k, oog));
            continue;
        }
        if !isb {
            continue;
        }
        let li_oog = std::mem::take(newtki.change_interferences(k, oog));
        fds_copy(&li_oog, newtki.change_interferences(k, g));
    }
    Ok(())
}

impl TopOpeBRepDSEIR {
    /// `TopOpeBRepDS_EIR(HDS)`; `TopOpeBRepDSEIR::new(&hds)`.
    pub fn new(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Self {
        Self {
            my_hds: hds.clone(),
        }
    }

    /// `ProcessEdgeInterferences()`: the interferences of every edge of the structure
    /// reduced; `eir.process_edge_interferences()?`.
    pub fn process_edge_interferences(&self) -> Result<(), Failure> {
        let nshape = href(&self.my_hds)?.ds().nb_shapes();
        for i in 1..=nshape {
            let is_edge = {
                let bds = href(&self.my_hds)?.ds();
                let s = bds.shape(i, true);
                !s.is_null() && s.shape_type()? == ShapeEnum::Edge
            };
            if is_edge {
                self.process_edge_interferences_index(i)?;
            }
        }
        Ok(())
    }

    /// `ProcessEdgeInterferences(EIX)`: the interferences of the non-degenerated edge
    /// `eix` merged on same domain vertices then reduced per geometry;
    /// `eir.process_edge_interferences_index(3)?`.
    pub fn process_edge_interferences_index(&self, eix: i32) -> Result<(), Failure> {
        let mut bds = href(&self.my_hds)?.change_ds();
        let e = topods::edge(bds.shape(eix, true))?;
        if BRepTool::degenerated(&e)? {
            return Ok(());
        }
        let li = bds.change_shape_interferences_index(eix)?.clone();
        let mut newtki = TopOpeBRepDSTKI::new();
        newtki.fill_on_geometry(&li)?;
        merge_sd_vertices(&bds, &li, &mut newtki)?;
        let mut linew = empty();
        newtki.init();
        while let Some((_, _, loi)) = newtki.change_value() {
            let mut loi = std::mem::take(loi);
            fun_process_edge_interferences(eix, &mut bds, &mut loi)?;
            fds_copy(&loi, &mut linew);
            newtki.next();
        }
        // natively the PNC filtering that follows is disabled (performPNC = false)
        *bds.change_shape_interferences_index(eix)? = linew;
        Ok(())
    }
}
