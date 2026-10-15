//! The purges and section edge fillings of the structure (FUN_ds_PURGEforE9,
//! FUN_ds_FillSDMFaces, FUN_ds_addSEsdm1d, FUN_ds_hasI2d, FUN_ds_hasFEI,
//! FUN_ds_FEIGb1TO0).
use super::{
    fds_data, fds_idata, fds_si_is_gi_of_i_of_sba_of_t_of_i, fds_tdata, fun_ds_get_vsdm,
    has_same_domain, href, iref,
};
use crate::top_ope_brep_ds::process_interferences_tool::{
    fun_select_sk_interference, fun_select_trasha_interference,
};
use crate::top_ope_brep_ds::tool::{MapOfShapeOn1State, TopOpeBRepDSTOOL};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSFaceEdgeInterference,
    TopOpeBRepDSHDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    fdscnx_edge_connexity_same_shape_ds,
};
use crate::top_ope_brep_tool::fun_tool_orient_v_in_e;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};

type H = Handle<TopOpeBRepDSInterference>;

/// The edge of index `i` when it is a non-degenerated edge.
fn non_degenerated_edge(bds: &TopOpeBRepDSDataStructure, i: i32) -> Result<Option<Edge>, Failure> {
    let s = bds.shape(i, true);
    if s.shape_type()? != ShapeEnum::Edge {
        return Ok(None);
    }
    let e = topods::edge(s)?;
    Ok((!BRepTool::degenerated(&e)?).then_some(e))
}

/// Whether the interference of `FUN_ds_PURGEforE9` goes: its edge `ie` splits ON its
/// face, or the face is same domain with a connex face of the edge.
fn purged_e9(
    bds: &TopOpeBRepDSDataStructure,
    ie: i32,
    i: &H,
    mapf: &IndexedMap<Shape, ShapeMapHasher>,
) -> Result<bool, Failure> {
    if fds_si_is_gi_of_i_of_sba_of_t_of_i(bds, ie, i)? {
        return Ok(true);
    }
    let f = bds.shape(fds_data(i).3, true);
    if !has_same_domain(bds, f) {
        return Ok(false);
    }
    Ok(bds.shape_same_domain(f).iter().any(|s| mapf.contains(s)))
}

/// `FUN_ds_PURGEforE9(HDS)`: on each non-degenerated edge, the FACE interferences of a
/// face seeing the edge as a split ON, or same domain with a connex face of the edge,
/// removed; `fun_ds_purge_for_e9(&hds)?`.
pub fn fun_ds_purge_for_e9(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_shapes() {
        let Some(e) = non_degenerated_edge(&bds, i)? else {
            continue;
        };
        let ie = bds.shape_index(&e, true);
        let mut licopy = bds.shape_interferences(&e, true).clone();
        let mut l3d_f = ListOfInterference::default();
        if fun_select_sk_interference(&mut licopy, TopOpeBRepDSKind::Face, &mut l3d_f)? == 0 {
            continue;
        }
        let lfcx_e = fdscnx_edge_connexity_same_shape_ds(&e, &bds)?;
        if lfcx_e.is_empty() {
            continue;
        }
        let mut mapf: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        for f in lfcx_e.iter() {
            mapf.add(f.clone());
        }
        let (mut kept, mut removed) = (ListOfInterference::default(), false);
        for i in l3d_f.iter() {
            if purged_e9(&bds, ie, i, &mapf)? {
                removed = true;
            } else {
                kept.append(i.clone());
            }
        }
        if !removed {
            continue;
        }
        let lii = bds.change_shape_interferences(&e);
        lii.clear();
        super::fds_copy(&licopy, lii);
        super::fds_copy(&kept, lii);
    }
    Ok(())
}

/// `FUN_EisSE2` result: no section edge.
const NONE: i32 = 0;
/// `FUN_EisSE2` result: the edge and the support edge are section edges.
const EREF_AND_ESUP: i32 = 2;
/// `FUN_tool_orientVinE`: the first vertex.
const FIRST: i32 = 1;
/// `FUN_tool_orientVinE`: the last vertex.
const LAST: i32 = 2;

/// `FUN_EisSE2(BDS, Fsdm, E, I)` (static): EREFandESUP for a 1d vertex interference of
/// `e` sharing its vertex with the support edge, else NONE.
fn e_is_se2(bds: &TopOpeBRepDSDataStructure, e: &Edge, i: &H) -> Result<i32, Failure> {
    let o = iref(i)?
        .transition()
        .orientation(State::In, ShapeEnum::Face);
    let ((sb, _, _, _), (gt, g, _, _)) = fds_idata(i);
    if gt != TopOpeBRepDSKind::Vertex || o == Orientation::External || sb != ShapeEnum::Edge {
        return Ok(NONE);
    }
    let (rk_e, rk_g) = (bds.ancestor_rank(e), bds.ancestor_rank_index(g));
    let vg = topods::vertex(bds.shape(g, true))?;
    let o_g_in_e = match fun_ds_get_vsdm(bds, g) {
        Some(gsd) => {
            let vgsd = topods::vertex(bds.shape(gsd, true))?;
            fun_tool_orient_v_in_e(if rk_e == rk_g { &vg } else { &vgsd }, e)
        }
        None => fun_tool_orient_v_in_e(&vg, e),
    };
    if o_g_in_e == 0 {
        return Ok(EREF_AND_ESUP);
    }
    let noshare = (o_g_in_e == LAST && o == Orientation::Forward)
        || (o_g_in_e == FIRST && o == Orientation::Reversed);
    Ok(if noshare { NONE } else { EREF_AND_ESUP })
}

/// The section edges added for one face with same domain faces of the other rank.
fn fill_sdm_face(bds: &mut TopOpeBRepDSDataStructure, s: &Shape) -> Result<(), Failure> {
    for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&e)?;
        if !bds.has_shape(&e, true) || bds.is_section_edge(&e, true) {
            continue;
        }
        let li = bds.shape_interferences(&e, true).clone();
        for i in li.iter() {
            let is_se2 = e_is_se2(bds, &e, i)?;
            if is_se2 == NONE {
                continue;
            }
            bds.add_section_edge(&e);
            if is_se2 == EREF_AND_ESUP {
                let es = topods::edge(bds.shape(iref(i)?.support(), true))?;
                bds.add_section_edge(&es);
            }
            break;
        }
    }
    Ok(())
}

/// `FUN_ds_FillSDMFaces(HDS)`: the edges of the faces with same domain faces of the
/// other rank, with a 1d vertex interference sharing the vertex, added as section
/// edges (with their support edges); `fun_ds_fill_sdm_faces(&hds)?`.
pub fn fun_ds_fill_sdm_faces(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_shapes() {
        let s = bds.shape(i, true).clone();
        if s.shape_type()? != ShapeEnum::Face {
            continue;
        }
        let rk_s = bds.ancestor_rank(&s);
        if !bds
            .shape_same_domain(&s)
            .iter()
            .any(|f| bds.ancestor_rank(f) != rk_s)
        {
            continue;
        }
        fill_sdm_face(&mut bds, &s)?;
    }
    Ok(())
}

/// `FUN_ds_addSEsdm1d(HDS)`: the non-degenerated edges of rank 1 sharing geometry with
/// same domain edges added as section edges (with them); `fun_ds_add_se_sdm1d(&hds)?`.
pub fn fun_ds_add_se_sdm1d(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_shapes() {
        let Some(e) = non_degenerated_edge(&bds, i)? else {
            continue;
        };
        if bds.is_section_edge(&e, true) || bds.ancestor_rank(&e) != 1 {
            continue;
        }
        let mut lsd = List::default();
        TopOpeBRepDSTOOL::e_share_g_ds(&bds, &e, &mut lsd)?;
        let share_g = !lsd.is_empty();
        for esd in lsd.iter() {
            bds.add_section_edge(&topods::edge(esd)?);
        }
        if share_g {
            bds.add_section_edge(&e);
        }
    }
    Ok(())
}

/// `FUN_ds_hasI2d(EIX, LLI, LI2d)`: the EDGE interferences of `lli` (not on EDGE
/// transitions) without a FACE interference of same geometry on their transition face
/// appended to `li2d`; the size of `li2d`; `fun_ds_has_i2d(3, &l, &mut l2d)?`.
pub fn fun_ds_has_i2d(
    _eix: i32,
    lli: &ListOfInterference,
    li2d: &mut ListOfInterference,
) -> Result<i32, Failure> {
    let mut li = lli.clone();
    let (mut l1d_e, mut lif, mut lie) =
        <(ListOfInterference, ListOfInterference, ListOfInterference)>::default();
    fun_select_trasha_interference(&mut li, ShapeEnum::Edge, &mut l1d_e)?;
    fun_select_sk_interference(&mut li, TopOpeBRepDSKind::Face, &mut lif)?;
    fun_select_sk_interference(&mut li, TopOpeBRepDSKind::Edge, &mut lie)?;
    for ie in lie.iter() {
        let (_, ge, _, _) = fds_data(ie);
        let (_, isb, _, _) = fds_tdata(ie);
        let is3d = lif.iter().any(|f| {
            let (_, gf, _, sf) = fds_data(f);
            ge == gf && sf == isb
        });
        if !is3d {
            li2d.append(ie.clone());
        }
    }
    Ok(li2d.extent())
}

/// `FUN_ds_hasFEI(pDS2d, F, GI, ITRA)`: an interference of the face of geometry `gi`
/// on the transition shape `itra`; `fun_ds_has_fei(&ds, &f, 2, 3)?`.
pub fn fun_ds_has_fei(
    pds2d: &TopOpeBRepDSDataStructure,
    f: &Shape,
    gi: i32,
    itra: i32,
) -> Result<bool, Failure> {
    if !pds2d.has_shape(f, true) {
        return Ok(false);
    }
    for i in pds2d.shape_interferences(f, true).iter() {
        let t = iref(i)?.transition();
        if fds_data(i).1 == gi && t.index()? == itra {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The face/edge interferences of GBound true split from the others (`FUN_ds_FEIGb1TO0`).
fn split_gb1(li: &ListOfInterference) -> (Vec<H>, Vec<H>) {
    let (mut lii, mut lgb1) = (Vec::new(), Vec::new());
    for i in li.iter() {
        let fei: Handle<TopOpeBRepDSFaceEdgeInterference> = i.cast();
        if fei.get().is_some_and(|f| f.gbound()) {
            lgb1.push(i.clone());
        } else {
            lii.push(i.clone());
        }
    }
    (lii, lgb1)
}

/// The rewrite of one GBound face/edge interference on the same domain edge
/// (`FUN_ds_FEIGb1TO0`): `Some(true)` rewritten, `Some(false)` already stored (dropped),
/// `None` kept as is.
fn feigb1_to0(
    bds: &TopOpeBRepDSDataStructure,
    mesp_on: &MapOfShapeOn1State,
    igb1: &H,
    lii: &[H],
) -> Result<Option<bool>, Failure> {
    let ii = iref(igb1)?;
    let (s, g) = (ii.support(), ii.geometry());
    let fs = topods::face(bds.shape(s, true))?;
    let Some(gsd) = TopOpeBRepDSTOOL::get_esd_ds(bds, &fs, g)? else {
        return Ok(None);
    };
    for x in lii {
        let x = iref(x)?;
        if x.support() == s && x.geometry() == gsd {
            return Ok(Some(false));
        }
    }
    let mut new_t = ii.transition();
    let mut new_o = new_t.orientation(State::In, ShapeEnum::Face);
    if new_o != Orientation::Internal && new_o != Orientation::External {
        let Some(conf) = TopOpeBRepDSTOOL::get_config_ds(bds, mesp_on, g, gsd)? else {
            return Ok(None);
        };
        if conf == 2 {
            new_o = top_abs::complement(new_o);
        }
    }
    new_t.set_orientation(new_o);
    ii.set_geometry(gsd);
    let fei: Handle<TopOpeBRepDSFaceEdgeInterference> = igb1.cast();
    if let Some(fei) = fei.get() {
        fei.set_gbound(false);
    }
    ii.set_transition(&new_t);
    Ok(Some(true))
}

/// `FUN_ds_FEIGb1TO0(HDS, MEspON)`: on each face, the face/edge interferences of
/// GBound true on an edge with a same domain edge in the support face moved onto that
/// edge (GBound false, orientation complemented for opposite configurations);
/// `fun_ds_feig_b1_to0(&hds, &mesp_on)?`.
pub fn fun_ds_feig_b1_to0(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    mesp_on: &MapOfShapeOn1State,
) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_shapes() {
        let f = bds.shape(i, true).clone();
        if f.shape_type()? != ShapeEnum::Face {
            continue;
        }
        let (lii, lgb1) = split_gb1(bds.shape_interferences(&f, true));
        if lgb1.is_empty() {
            continue;
        }
        let (mut left, mut lgb0) = (Vec::new(), Vec::new());
        for igb1 in lgb1 {
            match feigb1_to0(&bds, mesp_on, &igb1, &lii)? {
                Some(true) => lgb0.push(igb1),
                Some(false) => {}
                None => left.push(igb1),
            }
        }
        if lgb0.is_empty() {
            continue;
        }
        let new_li = bds.change_shape_interferences(&f);
        *new_li = lii.into_iter().chain(left).chain(lgb0).collect();
    }
    Ok(())
}
