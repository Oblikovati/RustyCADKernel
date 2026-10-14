//! The completions FUN_ds_completeforE7, FUN_ds_completeforSE8 and
//! FUN_ds_completeforSE9.
use super::complete_se::{evi_gbound, tki_groups};
use super::config::{fds_lo_iinfsup, mkt_states};
use super::{fds_parameter, has_same_domain, href, iref};
use crate::top_ope_brep_ds::process_interferences_tool::{
    fun_select_itrasha_interference, fun_select_si_interference, fun_select_sk_interference,
    fun_select_traori_interference, fun_select_trasha_interference, fun_selectpure2d_i,
    make_epv_interference_sk,
};
use crate::top_ope_brep_ds::redu::fun_ds_redu2d1d;
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSTransition, store_interference_ds,
};
use crate::top_ope_brep_tool::{
    fun_tool_orient_v_in_e, fun_tool_par_e, fun_tool_par_f, fun_tool_paron_ef,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point2;

type H = Handle<TopOpeBRepDSInterference>;

/// The states on the face `fs` of the edge `e` at `par` (`FUN_ds_completeforE7`):
/// through the edge of `fs` interfering at the point when there is one (`ies`).
fn e7_states(
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    range: (f64, f64, f64),
    fs: &Face,
    ies: Option<i32>,
) -> Result<Option<(State, State)>, Failure> {
    let par = range.2;
    let mut uv_fs = Point2::default();
    let Some(ies) = ies else {
        if !fun_tool_par_f(e, par, fs, &mut uv_fs)? {
            return Ok(None);
        }
        return mkt_states(e, range, fs, uv_fs, 1.0e-4, None);
    };
    let es = topods::edge(bds.shape(ies, true))?;
    let mut par_es = 0.;
    if !fun_tool_par_e(e, par, &es, &mut par_es)? {
        return Ok(None);
    }
    if !fun_tool_paron_ef(&es, par_es, fs, &mut uv_fs)? {
        return Ok(None);
    }
    mkt_states(e, range, fs, uv_fs, 1.0e-4, Some((&es, par_es)))
}

/// The new interferences of one FORWARD face interference `ifor` of
/// `FUN_ds_completeforE7` (none when not computable).
#[allow(clippy::too_many_arguments)] // the native locals
fn e7_forward(
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    (k, g): (TopOpeBRepDSKind, i32),
    ifor: &H,
    loicopy: &mut ListOfInterference,
    lrev: &mut ListOfInterference,
    lnew_i: &mut ListOfInterference,
) -> Result<Option<i32>, Failure> {
    let ifs = iref(ifor)?.support();
    let fs = topods::face(bds.shape(ifs, true))?;
    // natively the interferences of support IFS are selected into lREV, whose size is
    // returned (never 0 here)
    if fun_select_si_interference(loicopy, ifs, lrev)? == 0 {
        return Ok(None);
    }
    let mut lfse = ListOfInterference::default();
    let nfse = fun_select_itrasha_interference(loicopy, ifs, &mut lfse)?;
    let par = fds_parameter(ifor)?;
    let (par1, par2, _) = fds_lo_iinfsup(bds, e, par, k, g, bds.shape_interferences(e, true))?;
    let ies = if nfse == 0 {
        None
    } else {
        Some(iref(lfse.first().unwrap_or(&H::default()))?.support())
    };
    let Some((stb, sta)) = e7_states(bds, e, (par1, par2, par), &fs, ies)? else {
        return Ok(None);
    };
    let mut new_t = TopOpeBRepDSTransition::default();
    new_t.set_before(stb, ShapeEnum::Face);
    new_t.set_after(sta, ShapeEnum::Face);
    new_t.set_index(ifs);
    let b = if k == TopOpeBRepDSKind::Vertex {
        evi_gbound(ifor)?
    } else {
        false
    };
    lnew_i.append(make_epv_interference_sk(
        &new_t,
        ifs,
        g,
        par,
        k,
        TopOpeBRepDSKind::Face,
        b,
    )?);
    if let Some(ies) = ies {
        lnew_i.append(make_epv_interference_sk(
            &new_t,
            ies,
            g,
            par,
            k,
            TopOpeBRepDSKind::Edge,
            b,
        )?);
    }
    Ok(Some(ifs))
}

/// One geometry of `FUN_ds_completeforE7`: the new list of the geometry, or `None` to
/// keep it.
fn e7_geometry(
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    (k, g): (TopOpeBRepDSKind, i32),
    loi: &ListOfInterference,
) -> Result<Option<ListOfInterference>, Failure> {
    let mut loicopy = loi.clone();
    let mut l = [(); 3].map(|_| ListOfInterference::default());
    let [l1, lfor, lrev] = &mut l;
    fun_select_sk_interference(&mut loicopy, TopOpeBRepDSKind::Face, l1)?;
    let nfor = fun_select_traori_interference(l1, Orientation::Forward, lfor)?;
    let nrev = fun_select_traori_interference(l1, Orientation::Reversed, lrev)?;
    if nfor == 0 || nrev == 0 {
        return Ok(None);
    }
    let (mut lnew_i, mut i_fs) = (ListOfInterference::default(), 0);
    for ifor in lfor.iter() {
        if let Some(ifs) = e7_forward(bds, e, (k, g), ifor, &mut loicopy, lrev, &mut lnew_i)? {
            i_fs = ifs;
        }
    }
    if i_fs == 0 {
        return Ok(None);
    }
    let (mut loii, mut lefs) = (loi.clone(), ListOfInterference::default());
    fun_select_itrasha_interference(&mut loii, i_fs, &mut lefs)?;
    super::fds_copy(&lnew_i, &mut loii);
    Ok(Some(loii))
}

/// `FUN_ds_completeforE7(HDS)`: on each non-degenerated edge, at a geometry with
/// FORWARD and REVERSED FACE interferences, new transitions on the faces are computed
/// (replacing those on the last face); every edge list is rewritten grouped by
/// geometry; `fun_ds_complete_for_e7(&hds)?`.
pub fn fun_ds_complete_for_e7(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_shapes() {
        let ee = bds.shape(i, true).clone();
        if ee.shape_type()? != ShapeEnum::Edge {
            continue;
        }
        let e = topods::edge(&ee)?;
        if BRepTool::degenerated(&e)? {
            continue;
        }
        let li = bds.shape_interferences(&e, true).clone();
        let mut new_li = ListOfInterference::default();
        for (k, g, loi) in tki_groups(&li)? {
            let l = e7_geometry(&bds, &e, (k, g), &loi)?.unwrap_or(loi);
            super::fds_copy(&l, &mut new_li);
        }
        *bds.change_shape_interferences(&e) = new_li;
    }
    Ok(())
}

/// One geometry of `FUN_ds_completeforSE8`: the 2d interferences reducible with the 1d
/// ones get their reduced transition (changed in place through the handles).
fn se8_geometry(
    bds: &TopOpeBRepDSDataStructure,
    ise: i32,
    loi: &ListOfInterference,
) -> Result<(), Failure> {
    let mut li = loi.clone();
    let mut l = [(); 3].map(|_| ListOfInterference::default());
    let [l1d_e, lfe, l2d_fe] = &mut l;
    let n1d = fun_select_trasha_interference(&mut li, ShapeEnum::Edge, l1d_e)?;
    fun_select_sk_interference(&mut li, TopOpeBRepDSKind::Edge, lfe)?;
    let n2d = fun_selectpure2d_i(&li, lfe, l2d_fe)?;
    if n1d == 0 || n2d == 0 {
        return Ok(());
    }
    for i2d_fe in l2d_fe.iter() {
        let mut new_t2d = TopOpeBRepDSTransition::default();
        if fun_ds_redu2d1d(bds, ise, i2d_fe, l1d_e, &mut new_t2d)? {
            iref(i2d_fe)?.set_transition(&new_t2d);
        }
    }
    Ok(())
}

/// `FUN_ds_completeforSE8(HDS)`: on each section edge, the 2d interferences of a
/// geometry (not a vertex with same domain) reduced with its 1d ones; natively the
/// regrouped list is never stored back (`hasnew` stays false), only the transitions
/// change in place; `fun_ds_complete_for_se8(&hds)?`.
pub fn fun_ds_complete_for_se8(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_section_edges() {
        let se = bds.section_edge(i, true)?;
        let ise = bds.shape_index(&se, true);
        let li = bds.shape_interferences(&se, true).clone();
        for (kk, gg, loi) in tki_groups(&li)? {
            if kk == TopOpeBRepDSKind::Vertex && has_same_domain(&bds, bds.shape(gg, true)) {
                continue;
            }
            se8_geometry(&bds, ise, &loi)?;
        }
    }
    Ok(())
}

/// The 1d interferences of the same domain edge `esd` (of the other rank) of a
/// section edge without interferences, at its vertices of the section edge rank.
fn se9_esd(bds: &mut TopOpeBRepDSDataStructure, se: &Edge, esd: &Edge) -> Result<(), Failure> {
    let (ise, i_esd, rk_se) = (
        bds.shape_index(se, true),
        bds.shape_index(esd, true),
        bds.ancestor_rank(se),
    );
    let mut licopy = bds.shape_interferences(esd, true).clone();
    if licopy.is_empty() {
        return Ok(());
    }
    let mut lise = ListOfInterference::default();
    fun_select_si_interference(&mut licopy, ise, &mut lise)?;
    for (_, g, _) in tki_groups(&lise)? {
        let vg = topods::vertex(bds.shape(g, true))?;
        if has_same_domain(bds, &vg) || bds.ancestor_rank_index(g) != rk_se {
            continue;
        }
        let mut new_t =
            TopOpeBRepDSTransition::new(State::In, State::In, ShapeEnum::Edge, ShapeEnum::Edge);
        new_t.set_index(i_esd);
        if fun_tool_orient_v_in_e(&vg, se) == 0 {
            continue;
        }
        let par = BRepTool::parameter(&vg, se)?;
        let (kv, ke) = (TopOpeBRepDSKind::Vertex, TopOpeBRepDSKind::Edge);
        let new_i = make_epv_interference_sk(&new_t, i_esd, g, par, kv, ke, true)?;
        store_interference_ds(bds, &new_i, se)?;
    }
    Ok(())
}

/// `FUN_ds_completeforSE9(HDS)`: each section edge with same domain edges but no
/// interference gets IN/IN 1d interferences at its vertices met by the same domain
/// edges of the other rank; `fun_ds_complete_for_se9(&hds)?`.
pub fn fun_ds_complete_for_se9(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_section_edges() {
        let se = bds.section_edge(i, true)?;
        let rk_se = bds.ancestor_rank(&se);
        if !has_same_domain(&bds, &se) || !bds.shape_interferences(&se, true).is_empty() {
            continue;
        }
        let esd_se = bds.shape_same_domain(&se).clone();
        for esd in esd_se.iter() {
            let esd = topods::edge(esd)?;
            if bds.ancestor_rank(&esd) == rk_se {
                continue;
            }
            se9_esd(&mut bds, &se, &esd)?;
        }
    }
    Ok(())
}
