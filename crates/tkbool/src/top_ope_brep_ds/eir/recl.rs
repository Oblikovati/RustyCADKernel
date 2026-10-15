//! The reductions of closed same domain edges and pure 2d interferences of the edge
//! interference reducer (FUN_reclSE2, FUN_reclSE, FUN_unkeepEVIonGb1, FUN_keepl3dF,
//! FUN_reducepure2dI0, FUN_reducepure2dI).
use crate::top_ope_brep_ds::export::complete_se::tki_groups;
use crate::top_ope_brep_ds::export::{fds_copy, fds_data, fds_idata, fun_ds_sdm, iref};
use crate::top_ope_brep_ds::process_interferences_tool::select::{to_list, to_vec};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    TopOpeBRepDSTKI, TopOpeBRepDSTransition,
};
use crate::top_ope_brep_tool::{TopOpeBRepToolTOOL, fun_tool_orient_v_in_e};
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};

type H = Handle<TopOpeBRepDSInterference>;

/// The orientation from IN of an interference.
fn ori(i: &H) -> Result<Orientation, Failure> {
    Ok(iref(i)?
        .transition()
        .orientation(State::In, ShapeEnum::Face))
}

/// Whether `v[a]` is a FORWARD/REVERSED interference at the closing vertex of a closed
/// edge same domain as `e` with an opposite one later (removed from `v`).
fn has_closing_opposite(
    bds: &TopOpeBRepDSDataStructure,
    e: &topods::Edge,
    v: &mut Vec<H>,
    a: usize,
) -> Result<bool, Failure> {
    let (_, g1, _, s1) = fds_data(&v[a]);
    let o1 = ori(&v[a])?;
    if o1 == Orientation::Internal || o1 == Orientation::External {
        return Ok(false);
    }
    let co1 = top_abs::complement(o1);
    let v1 = topods::vertex(bds.shape(g1, true))?;
    let e1 = topods::edge(bds.shape(s1, true))?;
    let Some(vclo1) = TopOpeBRepToolTOOL::closed_e(&e1)? else {
        return Ok(false);
    };
    if !vclo1.is_same(&v1) || !fun_ds_sdm(bds, e, &e1) {
        return Ok(false);
    }
    for b in a + 1..v.len() {
        let o2 = ori(&v[b])?;
        if fds_data(&v[b]).3 == s1 && o2 == co1 {
            v.remove(b);
            return Ok(true);
        }
    }
    Ok(false)
}

/// `FUN_reclSE2(SIX, BDS, LI, reducedLI)` (static): a FORWARD and a REVERSED
/// interference at the closing vertex of a closed same domain edge reduced to one
/// INTERNAL interference (in `reduced_li`).
fn fun_recl_se2(
    six: i32,
    bds: &TopOpeBRepDSDataStructure,
    li: &mut ListOfInterference,
    reduced_li: &mut ListOfInterference,
) -> Result<(), Failure> {
    reduced_li.clear();
    let e = topods::edge(bds.shape(six, true))?;
    let mut v = to_vec(li);
    let mut a = 0;
    while a < v.len() {
        if has_closing_opposite(bds, &e, &mut v, a)? {
            let i1 = v.remove(a);
            iref(&i1)?.change_transition(|t| t.set_orientation(Orientation::Internal));
            reduced_li.append(i1);
        } else {
            a += 1;
        }
    }
    *li = to_list(v);
    Ok(())
}

/// `FUN_reclSE(EIX, BDS, LOI, RLOI)`: the VERTEX interferences reduced per geometry by
/// `FUN_reclSE2` (the reduced in `rloi`); the other geometries are dropped from `loi`,
/// as natively; `fun_recl_se(3, &ds, &mut loi, &mut rloi)?`.
pub fn fun_recl_se(
    eix: i32,
    bds: &TopOpeBRepDSDataStructure,
    loi: &mut ListOfInterference,
    rloi: &mut ListOfInterference,
) -> Result<(), Failure> {
    let groups = tki_groups(loi)?;
    loi.clear();
    for (k, _, mut l) in groups {
        if k != TopOpeBRepDSKind::Vertex || l.is_empty() {
            continue;
        }
        if l.extent() > 1 {
            let mut rl = ListOfInterference::default();
            fun_recl_se2(eix, bds, &mut l, &mut rl)?;
            fds_copy(&rl, rloi);
        }
        fds_copy(&l, loi);
    }
    Ok(())
}

/// `FUN_unkeepEVIonGb1(BDS, EIX, LI)` (static): the edge/vertex interferences at a
/// vertex of the edge removed.
pub(crate) fn fun_unkeep_evi_on_gb1(
    bds: &TopOpeBRepDSDataStructure,
    eix: i32,
    li: &mut ListOfInterference,
) -> Result<(), Failure> {
    let e = topods::edge(bds.shape(eix, true))?;
    let mut kept = Vec::new();
    for i in to_vec(li) {
        let evi = i.is_kind_name("TopOpeBRepDS_EdgeVertexInterference");
        let (gt, g, _, _) = fds_data(&i);
        if evi && gt == TopOpeBRepDSKind::Vertex {
            let v = topods::vertex(bds.shape(g, true))?;
            if fun_tool_orient_v_in_e(&v, &e) != 0 {
                continue;
            }
        }
        kept.push(i);
    }
    *li = to_list(kept);
    Ok(())
}

/// `FUN_keepl3dF(EIX, HDS, l3dF, LR3dFE, l3dFkeep)` (static): the FACE interferences
/// of the geometries not reduced in `lr3d_fe` appended to `l3d_fkeep`.
pub(crate) fn fun_keepl3d_f(
    l3d_f: &ListOfInterference,
    lr3d_fe: &ListOfInterference,
    l3d_fkeep: &mut ListOfInterference,
) -> Result<(), Failure> {
    let mut tki_r = TopOpeBRepDSTKI::new();
    tki_r.fill_on_geometry(lr3d_fe)?;
    for (k, g, loi) in tki_groups(l3d_f)? {
        if !tki_r.is_bound(k, g) {
            fds_copy(&loi, l3d_fkeep);
        }
    }
    Ok(())
}

/// `FUN_reducepure2dI0(LI, RLI)` (static): two 2d POINT interferences REVERSED then
/// FORWARD (or the opposite) on different faces reduced to one INTERNAL interference
/// between them (in `rli`, `li` emptied).
fn fun_reducepure2d_i0(
    li: &mut ListOfInterference,
    rli: &mut ListOfInterference,
) -> Result<(), Failure> {
    let (Some(i1), Some(i2)) = (li.first().cloned(), li.last().cloned()) else {
        return Ok(());
    };
    let (o1, o2) = (ori(&i1)?, ori(&i2)?);
    let ((tsb1, isb1, _, isa1), (_, g1, _, _)) = fds_idata(&i1);
    let ((tsb2, isb2, _, isa2), (_, g2, _, _)) = fds_idata(&i2);
    if isb1 == isb2 {
        return Ok(());
    }
    if !(g1 == g2 && tsb1 == ShapeEnum::Face && tsb1 == tsb2 && isb1 == isa1 && isb2 == isa2) {
        return Ok(());
    }
    let int12 = o1 == Orientation::Reversed && o2 == Orientation::Forward;
    let int21 = o2 == Orientation::Reversed && o1 == Orientation::Forward;
    if !int12 && !int21 {
        return Ok(());
    }
    let mut new_t = TopOpeBRepDSTransition::from_orientation(Orientation::Internal);
    new_t.set_index_before(if int12 { isb1 } else { isb2 });
    new_t.set_index_after(if int21 { isb1 } else { isb2 });
    iref(&i1)?.set_transition(&new_t);
    rli.append(i1);
    li.clear();
    Ok(())
}

/// `FUN_reducepure2dI(LI, RLI)` (static): per POINT geometry with two interferences,
/// `FUN_reducepure2dI0`.
pub(crate) fn fun_reducepure2d_i(
    li: &mut ListOfInterference,
    rli: &mut ListOfInterference,
) -> Result<(), Failure> {
    let mut new_li = ListOfInterference::default();
    for (k, _, mut loi) in tki_groups(li)? {
        let mut rloi = ListOfInterference::default();
        if loi.extent() == 2 && k == TopOpeBRepDSKind::Point {
            fun_reducepure2d_i0(&mut loi, &mut rloi)?;
        }
        fds_copy(&rloi, rli);
        fds_copy(&loi, &mut new_li);
    }
    *li = new_li;
    Ok(())
}
