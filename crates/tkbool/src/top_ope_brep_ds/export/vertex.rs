//! The vertex rewritings of the structure (FUN_ds_PointToVertex, FUN_ds_redusamsha,
//! FUN_ds_complete1dForSESDM).
use super::complete_se::{evi_gbound, tki_groups, tki_support_groups};
use super::config::mkt_states;
use super::{
    fds_data, fds_idata, fds_parameter, fds_state_e_with_f2d, has_same_domain, href, iref,
};
use crate::top_ope_brep_ds::process_interferences_tool::{
    fun_select_gk_interference, fun_select_itrasha_interference, fun_select_si_interference,
    fun_select_sk_interference, fun_select_traori_interference, fun_select_trasha_interference,
    fun_select_traunk_interference, make_epv_interference_sk,
};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSConfig, TopOpeBRepDSCurvePointInterference,
    TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure, TopOpeBRepDSInterference,
    TopOpeBRepDSInterferenceTool, TopOpeBRepDSKind, TopOpeBRepDSTransition, store_interference_ds,
};
use crate::top_ope_brep_tool::{
    fun_tool_bounds, fun_tool_curves_so, fun_tool_par_e, fun_tool_par_e_tol, fun_tool_par_v_on_e,
    fun_tool_paron_ef,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point2;

type H = Handle<TopOpeBRepDSInterference>;

/// The edge of index `i` (none for another type).
fn edge_at(bds: &TopOpeBRepDSDataStructure, i: i32) -> Result<Option<Edge>, Failure> {
    let s = bds.shape(i, true);
    if s.shape_type()? != ShapeEnum::Edge {
        return Ok(None);
    }
    topods::edge(s).map(Some)
}

/// The support edge of the EDGE interferences of a point when they are not all the
/// same (`Gfaulty` of `FUN_ds_PointToVertex`); the first support edge index.
fn faulty_support(loi: &ListOfInterference) -> Option<i32> {
    let mut scur = 0;
    for i in loi.iter() {
        let (_, _, st, s) = fds_data(i);
        if st != TopOpeBRepDSKind::Edge {
            continue;
        }
        if scur == 0 {
            scur = s;
        }
        if s != scur {
            return Some(scur);
        }
    }
    None
}

/// The vertex of the edge `scur` nearest to the point `g`, added with the rank of the
/// edge; its index.
fn nearest_vertex(bds: &mut TopOpeBRepDSDataStructure, g: i32, scur: i32) -> Result<i32, Failure> {
    let p_g = bds.point(g)?.point();
    let rk_es = bds.ancestor_rank_index(scur);
    let es = topods::edge(bds.shape(scur, true))?;
    let (vf, vl) = topexp::vertices(&es, false);
    let df = BRepTool::pnt(&vf)?.distance(&p_g);
    let dl = BRepTool::pnt(&vl)?.distance(&p_g);
    let vg: Vertex = if df < dl { vf } else { vl };
    Ok(bds.add_shape_with_rank(&vg, rk_es))
}

/// The curve/point interferences of a faulty point rewritten on its vertex `ivg`.
fn rewrite_on_vertex(
    bds: &TopOpeBRepDSDataStructure,
    s: &Edge,
    ivg: i32,
    loi: &ListOfInterference,
    new_li: &mut ListOfInterference,
) -> Result<(), Failure> {
    let vg = topods::vertex(bds.shape(ivg, true))?;
    let gbound = bds.ancestor_rank_index(ivg) == bds.ancestor_rank(s);
    for i in loi.iter() {
        let cpi: Handle<TopOpeBRepDSCurvePointInterference> = i.cast();
        if cpi.is_null() {
            continue;
        }
        let (_, _, st, si) = fds_data(i);
        let t = iref(i)?.transition();
        let Some(par) = fun_tool_par_v_on_e(&vg, s)? else {
            continue;
        };
        let kv = TopOpeBRepDSKind::Vertex;
        new_li.append(make_epv_interference_sk(&t, si, ivg, par, kv, st, gbound)?);
    }
    Ok(())
}

/// `FUN_ds_PointToVertex(HDS)`: the points of edges whose EDGE interferences have
/// different supports replaced by the nearest vertex of the first support edge (their
/// curve/point interferences rewritten as vertex ones; the points removed);
/// `fun_ds_point_to_vertex(&hds)?`.
pub fn fun_ds_point_to_vertex(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    let ns = bds.nb_shapes();
    let mut ipiv: DataMap<i32, i32> = DataMap::default();
    for i in 1..=ns {
        let Some(s) = edge_at(&bds, i)? else {
            continue;
        };
        let li = bds.shape_interferences(&s, true).clone();
        for (k, g, loi) in tki_groups(&li)? {
            if k == TopOpeBRepDSKind::Vertex {
                continue;
            }
            if let Some(scur) = faulty_support(&loi) {
                let ivg = nearest_vertex(&mut bds, g, scur)?;
                ipiv.bind(g, ivg);
            }
        }
    }
    if ipiv.is_empty() {
        return Ok(());
    }
    for i in 1..=ns {
        let Some(s) = edge_at(&bds, i)? else {
            continue;
        };
        let li = bds.shape_interferences(&s, true).clone();
        if li.is_empty() {
            continue;
        }
        let mut new_li = ListOfInterference::default();
        for (_, g, loi) in tki_groups(&li)? {
            match ipiv.seek(&g) {
                None => super::fds_copy(&loi, &mut new_li),
                Some(&ivg) => rewrite_on_vertex(&bds, &s, ivg, &loi, &mut new_li)?,
            }
        }
        *bds.change_shape_interferences(&s) = new_li;
    }
    let keys: Vec<i32> = ipiv.items().map(|(k, _)| *k).collect();
    for g in keys {
        bds.remove_point(g)?;
    }
    Ok(())
}

/// `FUN_redusamshaonE(BDS, I, EIX, newI)` (static): the EDGE interference of `i` on the
/// edge `eix` rebuilt with the transition computed on its face and corrected by the
/// states of the edge on the face; `None` when not computable.
fn redusamsha_on_e(bds: &TopOpeBRepDSDataStructure, i: &H, eix: i32) -> Result<Option<H>, Failure> {
    let ((_, ib, _, _), (gt, g, _, s)) = fds_idata(i);
    let e = topods::edge(bds.shape(eix, true))?;
    let par_e = fds_parameter(i)?;
    let (f, l) = fun_tool_bounds(&e)?;
    let es = topods::edge(bds.shape(s, true))?;
    let ftra = topods::face(bds.shape(ib, true))?;
    let mut par_es = 0.;
    if !fun_tool_par_e(&e, par_e, &es, &mut par_es)? {
        return Ok(None);
    }
    let mut uv = Point2::default();
    if !fun_tool_paron_ef(&es, par_es, &ftra, &mut uv)? {
        return Ok(None);
    }
    let Some((stb, sta)) = mkt_states(&e, (f, l, par_e), &ftra, uv, 1.0e-2, Some((&es, par_es)))?
    else {
        return Ok(None);
    };
    let mut new_t = TopOpeBRepDSTransition::default();
    new_t.set_index(ib);
    new_t.set_before(stb, ShapeEnum::Face);
    new_t.set_after(sta, ShapeEnum::Face);
    if !fds_state_e_with_f2d(bds, &e, par_e, gt, g, &ftra, &mut new_t)? {
        return Ok(None);
    }
    let b = if gt == TopOpeBRepDSKind::Vertex {
        evi_gbound(i)?
    } else {
        false
    };
    make_epv_interference_sk(&new_t, s, g, par_e, gt, TopOpeBRepDSKind::Edge, b).map(Some)
}

/// Whether an interference after `a` in `li` is the opposite of `li[a]` on the same
/// face and support (`FUN_ds_redusamsha` inner search).
fn has_opposite(li: &[H], a: usize, o1: Orientation, ib1: i32, s1: i32) -> Result<bool, Failure> {
    for i2 in &li[a + 1..] {
        let o2 = iref(i2)?
            .transition()
            .orientation(State::In, ShapeEnum::Face);
        let ((_, ib2, _, ia2), (_, _, _, s2)) = fds_idata(i2);
        if ib2 == ia2 && ib1 == ib2 && s1 == s2 && o1 == top_abs::complement(o2) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The reduction of the interferences of one support edge (`FUN_ds_redusamsha`):
/// the new interference of the first FORWARD/REVERSED one with an opposite one.
fn reduce_support(
    bds: &TopOpeBRepDSDataStructure,
    ie: i32,
    li: &[H],
) -> Result<Option<H>, Failure> {
    for (a, i1) in li.iter().enumerate() {
        let o1 = iref(i1)?
            .transition()
            .orientation(State::In, ShapeEnum::Face);
        if o1 != Orientation::Forward && o1 != Orientation::Reversed {
            continue;
        }
        let ((_, ib1, _, ia1), (_, _, _, s1)) = fds_idata(i1);
        if ib1 != ia1 || !has_opposite(li, a, o1, ib1, s1)? {
            continue;
        }
        if let Some(new_i) = redusamsha_on_e(bds, i1, ie)? {
            return Ok(Some(new_i));
        }
    }
    Ok(None)
}

/// The reduced interferences of one geometry (`FUN_ds_redusamsha`), or `None` to keep.
fn redusamsha_geometry(
    bds: &TopOpeBRepDSDataStructure,
    ie: i32,
    loi: &ListOfInterference,
) -> Result<Option<ListOfInterference>, Failure> {
    let mut loicopy = loi.clone();
    let (mut l0, mut l1, mut l2) =
        <(ListOfInterference, ListOfInterference, ListOfInterference)>::default();
    fun_select_trasha_interference(&mut loicopy, ShapeEnum::Edge, &mut l0)?;
    fun_select_trasha_interference(&mut loicopy, ShapeEnum::Face, &mut l1)?;
    if fun_select_sk_interference(&mut l1, TopOpeBRepDSKind::Edge, &mut l2)? == 0 {
        return Ok(None);
    }
    let mut newloi = ListOfInterference::default();
    for (_, _, li) in tki_support_groups(&l2)? {
        let v: Vec<H> = li.iter().cloned().collect();
        match (v.len() >= 2)
            .then(|| reduce_support(bds, ie, &v))
            .transpose()?
            .flatten()
        {
            Some(new_i) => {
                newloi.append(new_i);
            }
            None => super::fds_copy(&li, &mut newloi),
        }
    }
    super::fds_copy(&l0, &mut newloi);
    super::fds_copy(&l1, &mut newloi);
    Ok(Some(newloi))
}

/// `FUN_ds_redusamsha(HDS)`: on each edge, the opposite FACE/EDGE interferences of a
/// geometry on the same face and support edge reduced to one of computed transition;
/// `fun_ds_redusamsha(&hds)?`.
pub fn fun_ds_redusamsha(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_shapes() {
        let Some(e) = edge_at(&bds, i)? else {
            continue;
        };
        let ie = bds.shape_index(&e, true);
        let li = bds.shape_interferences_index(i, true)?.clone();
        let mut new_li = ListOfInterference::default();
        for (_, _, loi) in tki_groups(&li)? {
            let l = redusamsha_geometry(&bds, ie, &loi)?.unwrap_or(loi);
            super::fds_copy(&l, &mut new_li);
        }
        *bds.change_shape_interferences(&e) = new_li;
    }
    Ok(())
}

/// The vertices of `e` and their same domain vertices.
fn vertex_map(bds: &TopOpeBRepDSDataStructure, vs: [&Vertex; 2]) -> Map<Shape, ShapeMapHasher> {
    let mut m: Map<Shape, ShapeMapHasher> = Map::default();
    for v in vs {
        if v.is_null() {
            continue;
        }
        m.add(v.clone().into_shape());
        for s in bds.shape_same_domain(v).iter() {
            m.add(s.clone());
        }
    }
    m
}

/// The 1d vertex interferences of the section edge `ise` on the same domain edge
/// `iesd` (the selection chain of `FUN_ds_complete1dForSESDM`): (count, list).
fn esd_1d_vertex(
    bds: &TopOpeBRepDSDataStructure,
    ise: i32,
    iesd: i32,
) -> Result<(i32, ListOfInterference), Failure> {
    let mut li = bds.shape_interferences_index(ise, true)?.clone();
    let mut li1 = ListOfInterference::default();
    fun_select_traunk_interference(&mut li, &mut li1)?;
    li1.clear();
    fun_select_trasha_interference(&mut li, ShapeEnum::Edge, &mut li1)?;
    li.clear();
    fun_select_itrasha_interference(&mut li1, iesd, &mut li)?;
    li1.clear();
    fun_select_sk_interference(&mut li, TopOpeBRepDSKind::Edge, &mut li1)?;
    li.clear();
    fun_select_si_interference(&mut li1, iesd, &mut li)?;
    li1.clear();
    let ni = fun_select_gk_interference(&mut li, TopOpeBRepDSKind::Vertex, &mut li1)?;
    Ok((ni, li1))
}

/// Whether a 1d interference of orientation `ori` exists at `av` (or a same domain
/// vertex of it).
fn has_already(
    bds: &TopOpeBRepDSDataStructure,
    li: &ListOfInterference,
    ori: Orientation,
    av: &Vertex,
) -> Result<bool, Failure> {
    let (mut li1, mut li2) = (li.clone(), ListOfInterference::default());
    if fun_select_traori_interference(&mut li1, ori, &mut li2)? == 0 {
        return Ok(false);
    }
    let avg_map = vertex_map(bds, [av, &Vertex::default()]);
    for i1 in li2.iter() {
        if avg_map.contains(bds.shape(iref(i1)?.geometry(), true)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The 1d interferences of the vertices of one same domain edge `esd` added on `se`.
fn complete1d_esd(
    bds: &mut TopOpeBRepDSDataStructure,
    se: &Edge,
    esd: &Edge,
    agb_map: &Map<Shape, ShapeMapHasher>,
) -> Result<(), Failure> {
    let (i_se, i_esd, rk_esd) = (
        bds.shape_index(se, true),
        bds.shape_index(esd, true),
        bds.ancestor_rank(esd),
    );
    let mut is_so = false;
    if !fun_tool_curves_so(esd, se, &mut is_so)? {
        return Ok(());
    }
    let tol_esd = std_max(
        BRepTool::edge_tolerance(esd)?,
        BRepTool::edge_tolerance(se)?,
    );
    let (ni, li) = esd_1d_vertex(bds, i_se, i_esd)?;
    let (v0, v1) = topexp::vertices(esd, false);
    for av in [&v0, &v1] {
        if av.is_null() || agb_map.contains(av.as_ref()) {
            continue;
        }
        let ori = if is_so {
            av.orientation()
        } else {
            top_abs::reverse(av.orientation())
        };
        if ni != 0 && has_already(bds, &li, ori, av)? {
            continue;
        }
        let tol = std_max(BRepTool::vertex_tolerance(av)?, tol_esd);
        let par_esd = BRepTool::parameter(av, esd)?;
        let mut par = 0.;
        if !fun_tool_par_e_tol(esd, par_esd, se, &mut par, tol)? {
            continue;
        }
        let mut at = TopOpeBRepDSTransition::from_orientation(ori);
        at.set_shape_before(ShapeEnum::Edge);
        at.set_shape_after(ShapeEnum::Edge);
        at.set_index(i_esd);
        let iv = bds.add_shape_with_rank(av, rk_esd);
        let config = if is_so {
            TopOpeBRepDSConfig::SameOriented
        } else {
            TopOpeBRepDSConfig::DiffOriented
        };
        let ai = TopOpeBRepDSInterferenceTool::make_edge_vertex_interference(
            &at, i_esd, iv, false, config, par,
        );
        store_interference_ds(bds, &ai, se)?;
    }
    Ok(())
}

/// `FUN_ds_complete1dForSESDM(HDS)`: on each section edge with same domain edges of
/// the other rank, the 1d vertex interferences of their vertices (not vertices of the
/// section edge) added when missing; `fun_ds_complete1d_for_se_sdm(&hds)?`.
pub fn fun_ds_complete1d_for_se_sdm(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for is in 1..=bds.nb_section_edges() {
        let se = bds.section_edge(is, true)?;
        if BRepTool::degenerated(&se)? || !has_same_domain(&bds, &se) {
            continue;
        }
        let lesd = bds.shape_same_domain(&se).clone();
        if lesd.is_empty() {
            continue;
        }
        let rk_se = bds.ancestor_rank(&se);
        let (vse0, vse1) = topexp::vertices(&se, false);
        let agb_map = vertex_map(&bds, [&vse0, &vse1]);
        for esd in lesd.iter() {
            let esd = topods::edge(esd)?;
            if bds.ancestor_rank(&esd) == rk_se || BRepTool::degenerated(&esd)? {
                continue;
            }
            complete1d_esd(&mut bds, &se, &esd, &agb_map)?;
        }
    }
    Ok(())
}
