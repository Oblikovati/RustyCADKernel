//! The completions of the section edge interferences FUN_ds_completeforSE4 to SE6.
use super::complete_se::tki_groups;
use super::config::dir;
use super::{
    fds_data, fds_has_same_domain3d, fds_parameter, fds_tdata, fun_ds_getoov, fun_ds_has_i2d,
    has_same_domain, href, iref,
};
use crate::top_ope_brep_ds::process_interferences_tool::{
    fun_select_gi_interference, fun_select_gk_interference, fun_select_itrasha_interference,
    fun_select_sk_interference, fun_select_traori_interference, fun_select_trasha_interference,
    make_epv_interference,
};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSTransition, store_interference_ds,
};
use crate::top_ope_brep_tool::{
    fun_tool_getxx_ng, fun_tool_is_closing_e, fun_tool_nggeom_f, fun_tool_orient_e_in_f,
    fun_tool_par_e, fun_tool_paron_ef, tggeom_e,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Direction3, Point2, Vector3};

type H = Handle<TopOpeBRepDSInterference>;

/// The first interference of a non-empty list.
fn first(l: &ListOfInterference) -> H {
    l.first().cloned().unwrap_or_default()
}

/// The orientation from IN of an interference.
fn ori(i: &H) -> Result<Orientation, Failure> {
    Ok(iref(i)?
        .transition()
        .orientation(State::In, ShapeEnum::Face))
}

/// One POINT of `FUN_ds_completeforSE4`: `Err(None)`-like abort as `Ok(false)` (the
/// native `return` out of the whole completion), else `Ok(true)`.
fn se4_point(
    bds: &mut TopOpeBRepDSDataStructure,
    se: &Edge,
    k: TopOpeBRepDSKind,
    loi: &ListOfInterference,
) -> Result<bool, Failure> {
    let mut loicopy = loi.clone();
    let (mut l1, mut l2) = (ListOfInterference::default(), ListOfInterference::default());
    fun_select_sk_interference(&mut loicopy, TopOpeBRepDSKind::Edge, &mut l1)?;
    if fun_select_trasha_interference(&mut l1, ShapeEnum::Face, &mut l2)? < 1 {
        return Ok(true);
    }
    let i = first(&l2);
    let (_, g, _, s) = fds_data(&i);
    let (_, isb, _, _) = fds_tdata(&i);
    let es = topods::edge(bds.shape(s, true))?;
    let ftra = topods::face(bds.shape(isb, true))?;
    if !fun_tool_is_closing_e(&es, &ftra, &ftra)? {
        return Ok(true);
    }
    let (mut has_for, mut has_rev) = (false, false);
    for i2 in l2.iter() {
        let (_, _, _, s2) = fds_data(i2);
        let (_, isb2, _, _) = fds_tdata(i2);
        if s2 != s || isb2 != isb {
            return Ok(false);
        }
        let o2 = ori(i2)?;
        has_for = has_for || o2 == Orientation::Forward;
        has_rev = has_rev || o2 == Orientation::Reversed;
    }
    if has_for == has_rev {
        return Ok(true);
    }
    let new_o = if has_for {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    let mut new_t = TopOpeBRepDSTransition::from_orientation(new_o);
    new_t.set_index(isb);
    let par = fds_parameter(&i)?;
    let new_i = make_epv_interference(&new_t, s, g, par, k, false);
    store_interference_ds(bds, &new_i, se)?;
    Ok(true)
}

/// `FUN_ds_completeforSE4(HDS)`: on each section edge, at a point with FACE/EDGE
/// interferences of one orientation on a closing edge of their face, the opposite
/// interference is added (the whole completion stops at inconsistent data);
/// `fun_ds_complete_for_se4(&hds)?`.
pub fn fun_ds_complete_for_se4(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_section_edges() {
        let se = bds.section_edge(i, true)?;
        let li = bds.shape_interferences(&se, true).clone();
        for (k, _, loi) in tki_groups(&li)? {
            if k != TopOpeBRepDSKind::Point {
                continue;
            }
            if !se4_point(&mut bds, &se, k, &loi)? {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// The direction normal to the edge `e2` inside the face at `par_e2`
/// (`FUN_tool_nggeomF` then `FUN_tool_getxx`); `None` when undefined.
fn xx_in_face(e2: &Edge, par_e2: f64, f: &Face) -> Result<Option<Direction3>, Failure> {
    let mut uv = Point2::default();
    if !fun_tool_paron_ef(e2, par_e2, f, &mut uv)? {
        return Ok(None);
    }
    let ng = dir(fun_tool_nggeom_f(uv, f)?)?;
    fun_tool_getxx_ng(f, e2, par_e2, ng)
}

/// The outcome of one POINT of `FUN_ds_completeforSE5`.
enum Se5 {
    /// The interferences kept (copied to the new list).
    Keep,
    /// Skipped (natively `continue` past the copy: the interferences are dropped from
    /// the new list).
    Skip,
    /// The native `return` out of the whole completion.
    Abort,
    /// The rewritten interferences.
    New(ListOfInterference),
}

/// The 2d and 3d sides of `FUN_ds_completeforSE5`: whether the faces of the 2d
/// interference and of the 3d one lie opposite (`Some(true)`), the same side
/// (`Some(false)`), or abort (`None`).
fn se5_opposite(
    bds: &TopOpeBRepDSDataStructure,
    se: &Edge,
    i2d: &H,
    i3d: &H,
) -> Result<Option<bool>, Failure> {
    let s2 = iref(i2d)?.support();
    let i_f2 = iref(i2d)?.transition().index()?;
    let e2 = topods::edge(bds.shape(s2, true))?;
    let par_se = fds_parameter(i2d)?;
    let f2 = topods::face(bds.shape(i_f2, true))?;
    let mut par_e2 = 0.;
    if !fun_tool_par_e(se, par_se, &e2, &mut par_e2)? {
        return Ok(None);
    }
    let Some(xx_f2) = xx_in_face(&e2, par_e2, &f2)? else {
        return Ok(None);
    };
    let i_f3 = iref(i3d)?.transition().index()?;
    let f3 = topods::face(bds.shape(i_f3, true))?;
    let Some(xx_f3) = xx_in_face(&e2, par_e2, &f3)? else {
        return Ok(None);
    };
    Ok(Some(xx_f2.dot(xx_f3) <= 0.))
}

/// One POINT of `FUN_ds_completeforSE5`.
fn se5_point(
    bds: &TopOpeBRepDSDataStructure,
    se: &Edge,
    loi: &ListOfInterference,
) -> Result<Se5, Failure> {
    let mut loicopy = loi.clone();
    let mut l = [(); 4].map(|_| ListOfInterference::default());
    let [lext, lint, lfor, lrev] = &mut l;
    let has_ext = fun_select_traori_interference(&mut loicopy, Orientation::External, lext)? > 0;
    let has_int = fun_select_traori_interference(&mut loicopy, Orientation::Internal, lint)? > 0;
    if has_ext == has_int {
        return Ok(Se5::Keep);
    }
    let has_for = fun_select_traori_interference(&mut loicopy, Orientation::Forward, lfor)? > 0;
    let has_rev = fun_select_traori_interference(&mut loicopy, Orientation::Reversed, lrev)? > 0;
    if has_for == has_rev {
        return Ok(Se5::Keep);
    }
    let i2d = first(if has_for { lfor } else { lrev });
    let i3d = first(if has_ext { lext } else { lint });
    match se5_opposite(bds, se, &i2d, &i3d)? {
        None => return Ok(Se5::Abort),
        Some(false) => return Ok(Se5::Skip),
        Some(true) => {}
    }
    let mut new_o = if has_for {
        Orientation::Forward
    } else {
        Orientation::Reversed
    };
    if has_int {
        new_o = top_abs::complement(new_o);
    }
    let mut new_li = ListOfInterference::default();
    for i in (if has_int { &*lint } else { &*lext }).iter() {
        iref(i)?.change_transition(|t| t.set_orientation(new_o));
        new_li.append(i.clone());
    }
    super::fds_copy(lrev, &mut new_li);
    super::fds_copy(lfor, &mut new_li);
    super::fds_copy(&loicopy, &mut new_li);
    Ok(Se5::New(new_li))
}

/// `FUN_ds_completeforSE5(HDS)`: on each section edge, at a point with an INTERNAL
/// (or EXTERNAL) 3d interference and a FORWARD (or REVERSED) 2d one on tangent faces
/// lying opposite, the 3d interferences are reoriented (the whole completion stops
/// when a projection fails); `fun_ds_complete_for_se5(&hds)?`.
pub fn fun_ds_complete_for_se5(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_section_edges() {
        let se = bds.section_edge(i, true)?;
        let mut loi_c = bds.shape_interferences(&se, true).clone();
        let mut li = ListOfInterference::default();
        if fun_select_sk_interference(&mut loi_c, TopOpeBRepDSKind::Edge, &mut li)? < 1 {
            continue;
        }
        let (mut new_li, mut hasnew_li) = (ListOfInterference::default(), false);
        for (k, _, loi) in tki_groups(&li)? {
            let outcome = if k == TopOpeBRepDSKind::Point {
                se5_point(&bds, &se, &loi)?
            } else {
                Se5::Keep
            };
            match outcome {
                Se5::Keep => super::fds_copy(&loi, &mut new_li),
                Se5::Skip => {}
                Se5::Abort => return Ok(()),
                Se5::New(l) => {
                    hasnew_li = hasnew_li || !l.is_empty();
                    super::fds_copy(&l, &mut new_li);
                }
            }
        }
        if hasnew_li {
            super::fds_copy(&loi_c, &mut new_li);
            *bds.change_shape_interferences(&se) = new_li;
        }
    }
    Ok(())
}

/// The vertex `g` of the edge `esd` has a same domain vertex (`FUN_ds_getoov` on HDS).
fn has_oov(bds: &TopOpeBRepDSDataStructure, v: &Shape) -> bool {
    has_same_domain(bds, v) && fun_ds_getoov(v, bds).is_some()
}

/// The 1d transition on the same domain edge `esd` of a 2d interference (`FUN_ds_
/// completeforSE6`), from its orientation, or the tangents at an INTERNAL vertex.
#[allow(clippy::too_many_arguments)] // the native locals
fn se6_transition(
    se: &Edge,
    esd: &Edge,
    vg: &topods::Vertex,
    closed: bool,
    o: Orientation,
    par_e: f64,
    isvf: bool,
) -> Result<TopOpeBRepDSTransition, Failure> {
    let mut new_t =
        TopOpeBRepDSTransition::new(State::Out, State::Out, ShapeEnum::Edge, ShapeEnum::Edge);
    if closed {
        new_t.set_orientation(Orientation::Internal);
    } else if o != Orientation::Internal {
        new_t.set_orientation(o);
    } else {
        let par_esd = BRepTool::parameter(vg, esd)?;
        let tg_esd = tggeom_e(par_esd, esd)?.unwrap_or_else(Vector3::default);
        let tg_e = tggeom_e(par_e, se)?.unwrap_or_else(Vector3::default);
        let so = tg_esd.dot(tg_e) > 0.;
        let isforw = so == isvf;
        new_t.set_orientation(if isforw {
            Orientation::Forward
        } else {
            Orientation::Reversed
        });
    }
    Ok(new_t)
}

/// The 1d interferences of one vertex `g` (number `iv`) of a same domain edge `esd`
/// added on the section edge (`FUN_ds_completeforSE6`).
#[allow(clippy::too_many_arguments)] // the native locals
fn se6_vertex(
    bds: &mut TopOpeBRepDSDataStructure,
    se: &Edge,
    esd: &Edge,
    (g, iv, closed): (i32, i32, bool),
    l1d_e: &mut ListOfInterference,
    l2d_fe: &mut ListOfInterference,
) -> Result<(), Failure> {
    let vg = topods::vertex(bds.shape(g, true))?;
    if has_oov(bds, &vg) {
        return Ok(());
    }
    let (mut l1d_g, mut l2d_g) = (ListOfInterference::default(), ListOfInterference::default());
    fun_select_gi_interference(l1d_e, g, &mut l1d_g)?;
    if fun_select_gi_interference(l2d_fe, g, &mut l2d_g)? == 0 {
        return Ok(());
    }
    let i_esd = bds.shape_index(esd, true);
    let mut l1d_g_esd = ListOfInterference::default();
    if fun_select_itrasha_interference(&mut l1d_g, i_esd, &mut l1d_g_esd)? != 0 {
        return Ok(());
    }
    for i2d in l2d_g.iter() {
        let i_trasha = iref(i2d)?.transition().index_before();
        let o = ori(i2d)?;
        let f = topods::face(bds.shape(i_trasha, true))?;
        if !fun_tool_orient_e_in_f(esd, &f)?.0 {
            continue;
        }
        let par_e = fds_parameter(i2d)?;
        let mut new_t = se6_transition(se, esd, &vg, closed, o, par_e, iv == 1)?;
        new_t.set_index(i_esd);
        let new_i = make_epv_interference(&new_t, i_esd, g, par_e, TopOpeBRepDSKind::Vertex, false);
        store_interference_ds(bds, &new_i, se)?;
    }
    Ok(())
}

/// `FUN_ds_completeforSE6(HDS)`: on each section edge with 3d same domain edges, at
/// the vertices of those edges with a 2d interference and no 1d one on the edge, the
/// 1d interferences are added; `fun_ds_complete_for_se6(&hds)?`.
pub fn fun_ds_complete_for_se6(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let mut bds = href(hds)?.change_ds();
    for i in 1..=bds.nb_section_edges() {
        let se = bds.section_edge(i, true)?;
        let ise = bds.shape_index(&se, true);
        let mut l_esd3d: List<Shape> = List::default();
        if !fds_has_same_domain3d(&bds, &se, Some(&mut l_esd3d)) {
            continue;
        }
        let mut licopy = bds.shape_interferences(&se, true).clone();
        let mut lia = ListOfInterference::default();
        if fun_select_gk_interference(&mut licopy, TopOpeBRepDSKind::Vertex, &mut lia)? == 0 {
            continue;
        }
        let mut lib = ListOfInterference::default();
        for ia in lia.iter() {
            if !has_oov(&bds, bds.shape(iref(ia)?.geometry(), true)) {
                lib.append(ia.clone());
            }
        }
        let mut l2d_fe = ListOfInterference::default();
        fun_ds_has_i2d(ise, &lib, &mut l2d_fe)?;
        let (mut licopy, mut l1d_e) = (lib.clone(), ListOfInterference::default());
        fun_select_trasha_interference(&mut licopy, ShapeEnum::Edge, &mut l1d_e)?;
        for esd in l_esd3d.iter() {
            let esd = topods::edge(esd)?;
            let (vf, vl) = topexp::vertices(&esd, false);
            if BRepTool::degenerated(&esd)? {
                continue;
            }
            let closed = vf.is_same(&vl);
            let (ivf, ivl) = (bds.shape_index(&vf, true), bds.shape_index(&vl, true));
            for (iv, g) in [(1, ivf), (2, ivl)] {
                if g != 0 {
                    se6_vertex(
                        &mut bds,
                        &se,
                        &esd,
                        (g, iv, closed),
                        &mut l1d_e,
                        &mut l2d_fe,
                    )?;
                }
            }
        }
    }
    Ok(())
}
