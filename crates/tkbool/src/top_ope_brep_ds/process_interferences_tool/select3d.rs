//! The static selections of the 3d FACE/EDGE interferences (FUN_select3dI,
//! FUN_find3dISEsameISF, FUN_select3dISEsameISF).
use super::fun_select_itrasha_interference;
use super::select::{to_list, to_vec};
use crate::top_ope_brep_ds::export::{fds_data, fds_tdata, fun_ds_get_vsdm, fun_ds_sdm, iref};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSKind,
};
use crate::top_ope_brep_tool::fun_tool_eshared;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

type H = Handle<TopOpeBRepDSInterference>;

/// The vertex bookkeeping of `FUN_select3dI` for a VERTEX geometry `g1`: its same
/// domain vertex index in `oog`, the other vertex in `oov`, and the status (1 on the
/// edge, 2 off the edge, 3 with a same domain vertex).
fn vertex_status(
    bds: &TopOpeBRepDSDataStructure,
    g1: i32,
    rank_e: i32,
    oog: &mut i32,
    oov: &mut Shape,
) -> Result<i32, Failure> {
    let vg1: Shape = topods::vertex(bds.shape(g1, true))?.into_shape();
    let found = fun_ds_get_vsdm(bds, g1);
    *oog = found.unwrap_or(0);
    let g1hsd = found.is_some();
    if bds.ancestor_rank(&vg1) != rank_e {
        *oov = vg1;
        return Ok(if g1hsd { 3 } else { 2 });
    }
    if g1hsd {
        *oov = bds.shape(*oog, true).clone();
    }
    Ok(if g1hsd { 3 } else { 1 })
}

/// The count of the interferences of `l` on the transition shape `tra`.
fn count_itrasha(l: &ListOfInterference, tra: i32) -> Result<i32, Failure> {
    let (mut lcopy, mut lfound) = (l.clone(), ListOfInterference::default());
    fun_select_itrasha_interference(&mut lcopy, tra, &mut lfound)
}

/// The mutable state of the `FUN_select3dI` walk, kept from one interference to the
/// next as the native locals are.
struct Walk {
    oov: Shape,
    oog: i32,
    gsta: i32,
}

/// The inner loop of `FUN_select3dI`: the interferences after `a` paired with `fe[a]`
/// moved to `lresi` (same domain faces) or `l3d` (same or shared support edge);
/// whether one went to `l3d`.
fn pair_after(
    six: i32,
    bds: &mut TopOpeBRepDSDataStructure,
    fe: &mut Vec<H>,
    a: usize,
    w: &Walk,
    lresi: &mut ListOfInterference,
    l3d: &mut ListOfInterference,
) -> Result<bool, Failure> {
    let (gt1, _, st1, mut s1) = fds_data(&fe[a]);
    let (_, ib1, _, _) = fds_tdata(&fe[a]);
    let mut complex3d = false;
    let mut b = a + 1;
    while b < fe.len() {
        let (gt2, _, st2, mut s2) = fds_data(&fe[b]);
        let (_, ib2, _, _) = fds_tdata(&fe[b]);
        if gt2 != gt1 {
            break;
        }
        let f1 = topods::face(bds.shape(ib1, true))?.into_shape();
        let f2 = topods::face(bds.shape(ib2, true))?.into_shape();
        if ib1 == ib2 || fun_ds_sdm(bds, &f1, &f2) {
            lresi.append(fe.remove(b));
            continue;
        }
        let mut shared = st2 == st1 && s2 == s1;
        if !shared && w.gsta > 1 {
            let Some(eshared) = fun_tool_eshared(&w.oov, &f1, &f2)? else {
                b += 1;
                continue;
            };
            shared = true;
            if !bds.has_shape(&eshared, true) {
                let oorank = bds.ancestor_rank(&w.oov);
                bds.add_shape_with_rank(&eshared, oorank);
            }
            s1 = bds.shape_index(&eshared, true);
            s2 = s1;
        }
        let _ = s2;
        if shared && fun_ds_sdm(bds, bds.shape(six, true), bds.shape(s1, true)) {
            b += 1;
            continue;
        }
        if shared {
            l3d.append(fe.remove(b));
            complex3d = true;
        } else {
            b += 1;
        }
    }
    Ok(complex3d)
}

/// The residual check of `FUN_select3dI` with 3d interferences: each residual
/// interference without a 3d one on its transition face is removed by the native
/// `lFE.Remove(it)` with `it` walking `lFEresi`: at the head of `lFEresi` that removes
/// the first interference of `lFE` and the walk goes on in `lFE`; later it unlinks the
/// residual node (the native list of `lFE` then miscounts, which is not kept).
fn drop_residuals(
    fe: &mut Vec<H>,
    resi: &mut Vec<H>,
    l3d: &ListOfInterference,
) -> Result<(), Failure> {
    let (mut in_fe, mut pos, mut has_prev) = (false, 0usize, false);
    loop {
        let cur = if in_fe { &*fe } else { &*resi };
        let Some(i) = cur.get(pos) else {
            return Ok(());
        };
        let tra = iref(i)?.transition().index()?;
        if count_itrasha(l3d, tra)? != 0 {
            (pos, has_prev) = (pos + 1, true);
        } else if !has_prev {
            if fe.is_empty() {
                return Err(Failure::with_type(
                    "Standard_NoSuchObject",
                    "NCollection_BaseList::pRemoveFirst",
                    "NCollection_BaseList",
                ));
            }
            fe.remove(0);
            (in_fe, pos) = (true, 0);
        } else if in_fe {
            fe.remove(pos);
        } else {
            resi.remove(pos);
        }
    }
}

/// `FUN_select3dI(SIX, BDS, lFE, lFEresi, l3dFE)` (static): the pairs of interferences
/// of `lfe` of same geometry kind on different faces sharing their support edge moved
/// to `l3d_fe`; those on same domain faces to `lfe_resi`; the count of `l3d_fe`.
pub(crate) fn fun_select3d_i(
    six: i32,
    bds: &mut TopOpeBRepDSDataStructure,
    lfe: &mut ListOfInterference,
    lfe_resi: &mut ListOfInterference,
    l3d_fe: &mut ListOfInterference,
) -> Result<i32, Failure> {
    l3d_fe.clear();
    lfe_resi.clear();
    if lfe.extent() <= 1 {
        return Ok(0);
    }
    let e = topods::edge(bds.shape(six, true))?;
    let rank_e = bds.ancestor_rank(&e);
    let mut w = Walk {
        oov: Shape::default(),
        oog: 0,
        gsta: 0,
    };
    let mut fe = to_vec(lfe);
    let mut a = 0;
    while a < fe.len() {
        let (gt1, g1, _, _) = fds_data(&fe[a]);
        if gt1 == TopOpeBRepDSKind::Vertex {
            w.gsta = vertex_status(bds, g1, rank_e, &mut w.oog, &mut w.oov)?;
        }
        if pair_after(six, bds, &mut fe, a, &w, lfe_resi, l3d_fe)? {
            l3d_fe.append(fe.remove(a));
        } else {
            a += 1;
        }
    }
    let n3d = l3d_fe.extent();
    if lfe_resi.is_empty() {
        *lfe = to_list(fe);
        return Ok(n3d);
    }
    if n3d != 0 {
        let mut resi = to_vec(lfe_resi);
        drop_residuals(&mut fe, &mut resi, l3d_fe)?;
        *lfe_resi = to_list(resi);
    } else {
        fe.extend(to_vec(lfe_resi));
        lfe_resi.clear();
    }
    *lfe = to_list(fe);
    Ok(n3d)
}

/// `FUN_find3dISEsameISF(I1, it2)` (static): the first position from `from` of an
/// interference of `l` with the same transition (shape and index before, orientation)
/// and geometry as `i1`.
fn find3d_ise_same_isf(i1: &H, l: &[H], from: usize) -> Result<Option<usize>, Failure> {
    let o1 = iref(i1)?
        .transition()
        .orientation(State::In, ShapeEnum::Face);
    let (gt1, g1, _, _) = fds_data(i1);
    let (sb1, ib1, _, _) = fds_tdata(i1);
    for (k, i2) in l.iter().enumerate().skip(from) {
        let o2 = iref(i2)?
            .transition()
            .orientation(State::In, ShapeEnum::Face);
        let (gt2, g2, _, _) = fds_data(i2);
        let (sb2, ib2, _, _) = fds_tdata(i2);
        if sb1 == sb2 && ib1 == ib2 && o1 == o2 && gt1 == gt2 && g1 == g2 {
            return Ok(Some(k));
        }
    }
    Ok(None)
}

/// `FUN_select3dISEsameISF(lFE, l3dFE, l3dFEresi, lF, l3dF)` (static): each 3d
/// FACE/EDGE interference matched with a FACE interference of `lf` (moved to `l3d_f`)
/// stays; the others leave `l3d_fe` for `l3d_fe_resi` (matched in `l3d_f`) or `lfe`;
/// the count of `l3d_f`.
pub(crate) fn fun_select3d_ise_same_isf(
    lfe: &mut ListOfInterference,
    l3d_fe: &mut ListOfInterference,
    l3d_fe_resi: &mut ListOfInterference,
    lf: &mut ListOfInterference,
    l3d_f: &mut ListOfInterference,
) -> Result<i32, Failure> {
    let (mut v3d, mut vf) = (to_vec(l3d_fe), to_vec(lf));
    let mut a = 0;
    while a < v3d.len() {
        let i1 = v3d[a].clone();
        if let Some(p) = find3d_ise_same_isf(&i1, &vf, 0)? {
            l3d_f.append(vf.remove(p));
            a += 1;
            continue;
        }
        if find3d_ise_same_isf(&i1, &to_vec(l3d_f), 0)?.is_some() {
            l3d_fe_resi.append(i1);
        } else {
            lfe.append(i1);
        }
        v3d.remove(a);
    }
    *l3d_fe = to_list(v3d);
    *lf = to_list(vf);
    Ok(l3d_f.extent())
}
