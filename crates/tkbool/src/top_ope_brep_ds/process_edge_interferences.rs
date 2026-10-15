// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_ProcessEdgeInterferences.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The filters of the edge interferences used by `TopOpeBRepDS_Filter`
//! (FUN_vertexofedge, FUN_unkeepEinterferences, FUN_unkeepEsymetrictransitions,
//! FUN_orderFFsamedomain, FUN_orderSTATETRANSonG, FUN_orderSTATETRANS,
//! FUN_resolveEUNKNOWN, FUN_purgeDSonSE).
use super::export::complete_se::tki_groups;
use super::export::{fds_copy, fds_data, fds_idata, fds_tdata, has_same_domain, href, iref};
use super::process_interferences_tool::select::{to_list, to_vec};
use super::process_interferences_tool::{
    fun_interfhassupport, fun_transition_shape_equal, fun_unkeep_unknown,
};
use super::{
    ListOfInterference, TopOpeBRepDSCurvePointInterference, TopOpeBRepDSDataStructure,
    TopOpeBRepDSHDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    fdscnx_edge_connexity_same_shape_ds,
};
use crate::top_ope_brep_tool::fsc_get_psc_with_reference;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape, Shell, Solid};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

type H = Handle<TopOpeBRepDSInterference>;

/// `FUN_vertexofedge(V, E)`: `v` is a vertex of `e`; `fun_vertexofedge(&v, &e)?`.
pub fn fun_vertexofedge(v: &Shape, e: &Shape) -> Result<bool, Failure> {
    for x in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        if x.is_same(v) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `FUN_keepEinterference(DS, I, E)` (static): an edge/vertex interference not on
/// `e` and not ON an edge; a curve/point interference not at a bound of `e`.
fn keep_e_interference(ds: &TopOpeBRepDSDataStructure, i: &H, e: &Shape) -> Result<bool, Failure> {
    if i.is_kind_name("TopOpeBRepDS_EdgeVertexInterference") {
        let t = iref(i)?.transition();
        let on_edge = |sha: ShapeEnum, st: State| sha == ShapeEnum::Edge && st == State::On;
        let k2 = !on_edge(t.shape_before(), t.before()) && !on_edge(t.shape_after(), t.after());
        let res = !fun_interfhassupport(ds, i, e)? && k2;
        // natively the vertices of the edge are compared with the same domain vertices
        // of the geometry without effect
        topexp::vertices(&topods::edge(e)?, false);
        return Ok(res);
    }
    let cpi: Handle<TopOpeBRepDSCurvePointInterference> = i.cast();
    if let Some(cpi) = cpi.get() {
        let eps = precision::p_confusion();
        let par = cpi.parameter();
        let (f, l) = BRepTool::range(&topods::edge(e)?)?;
        return Ok(!((par - f).abs() < eps || (par - l).abs() < eps));
    }
    Ok(true)
}

/// `FUN_unkeepEinterferences(LI, BDS, SIX)`: on a non-degenerated edge, the
/// interferences not kept by `FUN_keepEinterference` removed; the size of the list;
/// `fun_unkeep_e_interferences(&mut l, &ds, 3)?`.
pub fn fun_unkeep_e_interferences(
    li: &mut ListOfInterference,
    bds: &TopOpeBRepDSDataStructure,
    six: i32,
) -> Result<i32, Failure> {
    let e = bds.shape(six, true).clone();
    if BRepTool::degenerated(&topods::edge(&e)?)? {
        return Ok(li.extent());
    }
    let mut kept = Vec::new();
    for i in to_vec(li) {
        if keep_e_interference(bds, &i, &e)? {
            kept.push(i);
        }
    }
    *li = to_list(kept);
    Ok(li.extent())
}

/// Whether two interferences differ only by opposite (OUT/IN) states.
fn symmetric(i1: &H, i2: &H) -> Result<bool, Failure> {
    let ((_, isb1, _, isa1), d1) = fds_idata(i1);
    let ((_, isb2, _, isa2), d2) = fds_idata(i2);
    if d1 != d2 || isb1 != isb2 || isa1 != isa2 {
        return Ok(false);
    }
    let (t1, t2) = (iref(i1)?.transition(), iref(i2)?.transition());
    let outin = |a: State, b: State| a == State::Out && b == State::In;
    let oppostate = (outin(t1.before(), t2.before()) || outin(t2.before(), t1.before()))
        && (outin(t1.after(), t2.after()) || outin(t2.after(), t1.after()));
    Ok(fun_transition_shape_equal(&t1, &t2) && oppostate)
}

/// `FUN_unkeepEsymetrictransitions(LI, BDS, SIX)`: on a non-degenerated edge, the
/// pairs of interferences of opposite states removed;
/// `fun_unkeep_esymetrictransitions(&mut l, &ds, 3)?`.
pub fn fun_unkeep_esymetrictransitions(
    li: &mut ListOfInterference,
    bds: &TopOpeBRepDSDataStructure,
    six: i32,
) -> Result<(), Failure> {
    if BRepTool::degenerated(&topods::edge(bds.shape(six, true))?)? {
        return Ok(());
    }
    let mut v = to_vec(li);
    let mut a = 0;
    while a < v.len() {
        let mut remove1 = false;
        let mut b = a + 1;
        while b < v.len() {
            if symmetric(&v[a], &v[b])? {
                v.remove(b);
                remove1 = true;
            } else {
                b += 1;
            }
        }
        if remove1 {
            v.remove(a);
        } else {
            a += 1;
        }
    }
    *li = to_list(v);
    Ok(())
}

/// `FUN_orderFFsamedomain(LI, HDS, SIX)`: the FACE/FACE interferences between faces
/// with same domain faces first; `fun_order_ff_samedomain(&mut l, &hds, 3)?`.
pub fn fun_order_ff_samedomain(
    li: &mut ListOfInterference,
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    _six: i32,
) -> Result<(), Failure> {
    order_ff_samedomain_ds(li, &href(hds)?.ds())
}

/// `FUN_orderFFsamedomain` on the locked structure.
pub(crate) fn order_ff_samedomain_ds(
    li: &mut ListOfInterference,
    bds: &TopOpeBRepDSDataStructure,
) -> Result<(), Failure> {
    let (mut liffsd, mut liother) = (ListOfInterference::default(), ListOfInterference::default());
    for i in li.iter() {
        let (tsb1, isb1, tsa1, isa1) = fds_tdata(i);
        let ffsd = tsb1 == ShapeEnum::Face
            && tsa1 == ShapeEnum::Face
            && has_same_domain(bds, bds.shape(isb1, true))
            && has_same_domain(bds, bds.shape(isa1, true));
        if ffsd {
            liffsd.append(i.clone());
        } else {
            liother.append(i.clone());
        }
    }
    li.clear();
    fds_copy(&liffsd, li);
    fds_copy(&liother, li);
    Ok(())
}

/// `FUN_orderSTATETRANSonG(LI, HDS, SIX)`: the interferences with the same state and
/// shape before and after first; `fun_order_statetrans_on_g(&mut l)?`.
pub fn fun_order_statetrans_on_g(li: &mut ListOfInterference) -> Result<(), Failure> {
    let (mut l1, mut l2) = (ListOfInterference::default(), ListOfInterference::default());
    for i in li.iter() {
        let (tsb1, isb1, tsa1, isa1) = fds_tdata(i);
        let t1 = iref(i)?.transition();
        if tsb1 == tsa1 && isb1 == isa1 && t1.before() == t1.after() {
            l1.append(i.clone());
        } else {
            l2.append(i.clone());
        }
    }
    li.clear();
    fds_copy(&l1, li);
    fds_copy(&l2, li);
    Ok(())
}

/// `FUN_orderSTATETRANS(LI, HDS, SIX)`: the interferences grouped by geometry, each
/// group ordered by `FUN_orderSTATETRANSonG`; `fun_order_statetrans(&mut l)?`.
pub fn fun_order_statetrans(li: &mut ListOfInterference) -> Result<(), Failure> {
    let mut out = ListOfInterference::default();
    for (_, _, mut loi) in tki_groups(li)? {
        fun_order_statetrans_on_g(&mut loi)?;
        fds_copy(&loi, &mut out);
    }
    *li = out;
    Ok(())
}

/// The states of the edge points before and after a tangency point with the face
/// `fs`, classified against a solid of `fs` (`FUN_resolveEUNKNOWN`).
fn tangency_states(
    bds: &TopOpeBRepDSDataStructure,
    ee: &topods::Edge,
    s1: i32,
    parcpi: f64,
) -> Result<Option<(State, State)>, Failure> {
    let (ce, _, _) = BRepTool::curve(ee)?;
    let Some(ce) = ce.get() else {
        return Ok(None);
    };
    let (fe, le) = BRepTool::range(ee)?;
    let (ttb, tta) = (0.8, 0.2);
    let pb = ce.value((1. - ttb) * fe + ttb * parcpi)?;
    let pa = ce.value((1. - tta) * parcpi + tta * le)?;
    let bb = BRepBuilder::default();
    let fs = topods::face(bds.shape(s1, true))?;
    let (mut sh, mut so) = (Shell::default(), Solid::default());
    bb.make_shell(&mut sh);
    bb.make_solid(&mut so);
    bb.add(&mut sh, &fs)?;
    bb.add(&mut so, &sh)?;
    let so: Shape = so.into_shape();
    let (stateb, statea) = fsc_get_psc_with_reference(&so, |psc| -> Result<_, Failure> {
        psc.state_p3d_reference(pb)?;
        let stateb = psc.state();
        psc.state_p3d_reference(pa)?;
        Ok((stateb, psc.state()))
    })??;
    if stateb == State::Unknown || statea == State::Unknown {
        return Ok(None);
    }
    Ok(Some((stateb, statea)))
}

/// `FUN_resolveEUNKNOWN(LI, BDS, SIX)`: the unknown transitions of a POINT where the
/// edge is tangent to a face resolved by classifying near points against the face,
/// then the still unknown ones removed; `fun_resolve_e_unknown(&mut l, &mut ds, 3)?`.
pub fn fun_resolve_e_unknown(
    li: &mut ListOfInterference,
    bds: &mut TopOpeBRepDSDataStructure,
    six: i32,
) -> Result<(), Failure> {
    let ee = topods::edge(bds.shape(six, true))?;
    BRepTool::range(&ee)?;
    for i1 in li.iter() {
        if !iref(i1)?.transition().is_unknown() {
            continue;
        }
        let ((tsb1, isb1, tsa1, isa1), (gt1, _, st1, s1)) = fds_idata(i1);
        let idt = tsb1 == ShapeEnum::Face
            && tsa1 == ShapeEnum::Face
            && gt1 == TopOpeBRepDSKind::Point
            && st1 == TopOpeBRepDSKind::Face;
        if !(idt && isb1 == s1 && isa1 == s1) {
            continue;
        }
        let cpi: Handle<TopOpeBRepDSCurvePointInterference> = i1.cast();
        let Some(cpi) = cpi.get() else {
            continue;
        };
        if let Some((stateb, statea)) = tangency_states(bds, &ee, s1, cpi.parameter())? {
            iref(i1)?.change_transition(|t| t.set(stateb, statea, tsb1, tsa1));
        }
    }
    fun_unkeep_unknown(li, bds, six)
}

/// `FUN_purgeDSonSE(HDS, EIX, LI)`: on a section edge, the FACE/FACE interferences on a
/// face same domain (of the other rank) with a connex face of the edge removed;
/// `fun_purge_ds_on_se(&hds, 3, &mut l)?`.
pub fn fun_purge_ds_on_se(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    eix: i32,
    li: &mut ListOfInterference,
) -> Result<(), Failure> {
    purge_ds_on_se_ds(&href(hds)?.ds(), eix, li)
}

/// `FUN_purgeDSonSE` on the locked structure.
pub(crate) fn purge_ds_on_se_ds(
    bds: &TopOpeBRepDSDataStructure,
    eix: i32,
    li: &mut ListOfInterference,
) -> Result<(), Failure> {
    let se = topods::edge(bds.shape(eix, true))?;
    let rk_se = bds.ancestor_rank(&se);
    if !bds.is_section_edge(&se, true) {
        return Ok(());
    }
    let mut fsdm_fanc_se: Map<Shape, ShapeMapHasher> = Map::default();
    for f in fdscnx_edge_connexity_same_shape_ds(&se, bds)?.iter() {
        for sdmf in bds.shape_same_domain(f).iter() {
            if bds.ancestor_rank(sdmf) != rk_se {
                fsdm_fanc_se.add(sdmf.clone());
            }
        }
    }
    if fsdm_fanc_se.is_empty() {
        return Ok(());
    }
    let mut new_li = ListOfInterference::default();
    for (_, _, loi) in tki_groups(li)? {
        for i in loi.iter() {
            let (_, _, st, _) = fds_data(i);
            let (tsb, isb, _, _) = fds_tdata(i);
            let on_sdm = tsb == ShapeEnum::Face
                && st == TopOpeBRepDSKind::Face
                && fsdm_fanc_se.contains(bds.shape(isb, true));
            if !on_sdm {
                new_li.append(i.clone());
            }
        }
    }
    *li = new_li;
    Ok(())
}
