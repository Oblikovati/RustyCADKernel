//! The selections comparing the interferences of a list two by two (FUN_reducedoublons,
//! FUN_unkeepUNKNOWN, FUN_select2dI, FUN_selectpure2dI, FUN_select1dI,
//! FUN_select3dinterference). The lists are walked as vectors: removing at an index
//! is the native removal at an iterator.
use crate::top_ope_brep_ds::export::{fds_data, fds_idata, fun_ds_getoov, fun_ds_sdm, iref};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSEdgeVertexInterference,
    TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSTKI,
};
use crate::top_ope_brep_tool::{TopOpeBRepToolTOOL, fun_tool_maxtol};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

type H = Handle<TopOpeBRepDSInterference>;

/// The interferences of a list, in order.
pub(crate) fn to_vec(l: &ListOfInterference) -> Vec<H> {
    l.iter().cloned().collect()
}

/// A list of the interferences, in order.
pub(crate) fn to_list(v: Vec<H>) -> ListOfInterference {
    v.into_iter().collect()
}

/// Whether two edge/vertex interferences of same data are the same: on the closing
/// vertex of a closed edge (natively `vG.IsSame(vG)`, so any vertex of a closed edge),
/// else at the same parameter within the parametric tolerance of the edge.
fn same_evi(
    bds: &TopOpeBRepDSDataStructure,
    e: &Shape,
    g1: i32,
    evi1: &TopOpeBRepDSEdgeVertexInterference,
    evi2: &TopOpeBRepDSEdgeVertexInterference,
) -> Result<bool, Failure> {
    let vg = topods::vertex(bds.shape(g1, true))?;
    let _oov = fun_ds_getoov(&vg, bds);
    if TopOpeBRepToolTOOL::closed_e(&topods::edge(e)?)?.is_some() {
        return Ok(true);
    }
    let t = precision::parametric(fun_tool_maxtol(e)?);
    Ok((evi1.parameter() - evi2.parameter()).abs() <= t)
}

/// `FUN_reducedoublons(LI, BDS, SIX)`: the later interferences of same geometry,
/// support and transition as an earlier one (and, for two edge/vertex interferences,
/// at the same vertex parameter on the edge `six`) removed;
/// `fun_reducedoublons(&mut l, &ds, 3)?`.
pub fn fun_reducedoublons(
    li: &mut ListOfInterference,
    bds: &TopOpeBRepDSDataStructure,
    six: i32,
) -> Result<(), Failure> {
    let e = bds.shape(six, true).clone();
    let mut v = to_vec(li);
    let mut a = 0;
    while a < v.len() {
        let i1 = v[a].clone();
        let t1 = iref(&i1)?.transition();
        let d1 = fds_data(&i1);
        let mut b = a + 1;
        while b < v.len() {
            let i2 = v[b].clone();
            let t2 = iref(&i2)?.transition();
            let mut same = fds_data(&i2) == d1 && super::fun_transition_equal(&t1, &t2);
            if same {
                let (e1, e2): (
                    Handle<TopOpeBRepDSEdgeVertexInterference>,
                    Handle<TopOpeBRepDSEdgeVertexInterference>,
                ) = (i1.cast(), i2.cast());
                if let (Some(evi1), Some(evi2)) = (e1.get(), e2.get()) {
                    same = same_evi(bds, &e, d1.1, evi1, evi2)?;
                }
            }
            if same {
                v.remove(b);
            } else {
                b += 1;
            }
        }
        a += 1;
    }
    *li = to_list(v);
    Ok(())
}

/// `FUN_unkeepUNKNOWN(LI, BDS, SIX)`: the interferences of unknown transition
/// removed; `fun_unkeep_unknown(&mut l, &mut ds, 3)?`.
pub fn fun_unkeep_unknown(
    li: &mut ListOfInterference,
    _bds: &mut TopOpeBRepDSDataStructure,
    _six: i32,
) -> Result<(), Failure> {
    let mut v = to_vec(li);
    let mut keep = Vec::with_capacity(v.len());
    for i in v.drain(..) {
        if !iref(&i)?.transition().is_unknown() {
            keep.push(i);
        }
    }
    *li = to_list(keep);
    Ok(())
}

/// `FUN_select2dI(SIX, BDS, TRASHAk, lI, l2dI)`: among the interferences of `li`
/// whose transition face is no support of a FACE interference of the edge `six`, the
/// pairs with the same transition (before of kind `trashak`), geometry and support
/// kind on the same (or a same domain) face moved to the cleared `l2d_i`; their count;
/// `fun_select2d_i(3, &mut ds, ShapeEnum::Face, &mut l, &mut l2d)?`.
pub fn fun_select2d_i(
    six: i32,
    bds: &mut TopOpeBRepDSDataStructure,
    trashak: ShapeEnum,
    li: &mut ListOfInterference,
    l2d_i: &mut ListOfInterference,
) -> Result<i32, Failure> {
    l2d_i.clear();
    if li.extent() <= 1 {
        return Ok(0);
    }
    let ton_face = trashak == ShapeEnum::Face;
    let mut mapftra: Map<i32> = Map::default();
    for i in bds.shape_interferences_index(six, true)?.iter() {
        if iref(i)?.support_type() == TopOpeBRepDSKind::Face {
            mapftra.add(iref(i)?.support());
        }
    }
    let (mut kept, mut lii) = (Vec::new(), Vec::new());
    let v = to_vec(li);
    for (k, i) in v.iter().enumerate() {
        let ind = match iref(i).and_then(|ii| ii.transition().index()) {
            Ok(ind) => ind,
            Err(e) => {
                // natively the interferences already moved to lII are lost
                kept.extend_from_slice(&v[k..]);
                *li = to_list(kept);
                return Err(e);
            }
        };
        if mapftra.contains(&ind) {
            kept.push(i.clone());
        } else {
            lii.push(i.clone());
        }
    }
    let mut a = 0;
    while a < lii.len() {
        let i1 = lii[a].clone();
        let ((sb1, ib1, _, ia1), (gt1, g1, st1, _)) = fds_idata(&i1);
        if sb1 != trashak {
            a += 1;
            continue;
        }
        let mut complex2d = false;
        let mut b = a + 1;
        while b < lii.len() {
            let ((sb2, ib2, _, ia2), (gt2, g2, st2, _)) = fds_idata(&lii[b]);
            if !(sb1 == sb2 && ib1 == ib2 && ia1 == ia2) || !(gt2 == gt1 && g2 == g1 && st2 == st1)
            {
                b += 1;
                continue;
            }
            complex2d = ib1 == ib2;
            if ton_face {
                let f1 = topods::face(bds.shape(ib1, true))?;
                let f2 = topods::face(bds.shape(ib2, true))?;
                complex2d = complex2d || fun_ds_sdm(bds, &f1, &f2);
            }
            if complex2d {
                l2d_i.append(lii.remove(b));
            } else {
                b += 1;
            }
        }
        if complex2d {
            l2d_i.append(lii.remove(a));
        } else {
            a += 1;
        }
    }
    kept.extend(lii);
    *li = to_list(kept);
    Ok(l2d_i.extent())
}

/// `FUN_selectpure2dI(lF, lFE, l2dFE)`: the interferences of `lfe` with a transition
/// on one face that supports no interference of `lf` moved to the cleared `l2d_fe`;
/// their count; `fun_selectpure2d_i(&lf, &mut lfe, &mut l2d)?`.
pub fn fun_selectpure2d_i(
    lf: &ListOfInterference,
    lfe: &mut ListOfInterference,
    l2d_fe: &mut ListOfInterference,
) -> Result<i32, Failure> {
    l2d_fe.clear();
    let mut kept = Vec::new();
    for ife in to_vec(lfe) {
        let tfe = iref(&ife)?.transition();
        let (ib, ia) = (tfe.index_before(), tfe.index_after());
        if ib != ia {
            kept.push(ife);
            continue;
        }
        let mut found_if = false;
        for i in lf.iter() {
            if iref(i)?.support() == ib {
                found_if = true;
                break;
            }
        }
        if found_if {
            kept.push(ife);
        } else {
            l2d_fe.append(ife);
        }
    }
    *lfe = to_list(kept);
    Ok(l2d_fe.extent())
}

/// The 1d interferences of one vertex geometry (`FUN_select1dI` loop body): appended
/// to `new_li` or `l1d_i` as natively (the later ones once per earlier match).
fn select1d_vertex(
    bds: &TopOpeBRepDSDataStructure,
    eix: &Shape,
    loi: &[H],
    new_li: &mut ListOfInterference,
    l1d_i: &mut ListOfInterference,
) -> Result<(), Failure> {
    for (a, i1) in loi.iter().enumerate() {
        let ((sb1, ib1, _, ia1), (_, _, _, s1)) = fds_idata(i1);
        if !(sb1 == ShapeEnum::Edge && ib1 == ia1) || !fun_ds_sdm(bds, eix, bds.shape(s1, true)) {
            new_li.append(i1.clone());
            continue;
        }
        let mut complex1d = false;
        for i2 in &loi[a + 1..] {
            let ((sb2, ib2, _, ia2), (_, _, _, s2)) = fds_idata(i2);
            if !(sb1 == sb2 && ib2 == ia2) {
                new_li.append(i2.clone());
                continue;
            }
            complex1d = fun_ds_sdm(bds, eix, bds.shape(s2, true));
            if !complex1d {
                new_li.append(i2.clone());
                continue;
            }
            l1d_i.append(i2.clone());
        }
        if complex1d {
            l1d_i.append(i1.clone());
        }
    }
    Ok(())
}

/// `FUN_select1dI(SIX, BDS, LI, l1dI)`: for each VERTEX geometry of several
/// interferences, those of 1d transitions on edges same domain as the edge `six`
/// moved to the cleared `l1d_i` (as the native loops do); their count;
/// `fun_select1d_i(3, &mut ds, &mut l, &mut l1d)?`.
pub fn fun_select1d_i(
    six: i32,
    bds: &mut TopOpeBRepDSDataStructure,
    li: &mut ListOfInterference,
    l1d_i: &mut ListOfInterference,
) -> Result<i32, Failure> {
    l1d_i.clear();
    if li.extent() <= 1 {
        return Ok(0);
    }
    let mut new_li = ListOfInterference::default();
    let eix = bds.shape(six, true).clone();
    let mut tki = TopOpeBRepDSTKI::new();
    tki.fill_on_geometry(li)?;
    tki.init();
    while let Some((k, _, loi)) = tki.value() {
        let loi = to_vec(loi);
        if k != TopOpeBRepDSKind::Vertex || loi.len() <= 1 {
            new_li.extend_from(&loi);
        } else {
            select1d_vertex(bds, &eix, &loi, &mut new_li, l1d_i)?;
        }
        tki.next();
    }
    *li = new_li;
    Ok(l1d_i.extent())
}

/// Appending a slice of interferences to a list.
trait ExtendFrom {
    fn extend_from(&mut self, v: &[H]);
}

impl ExtendFrom for ListOfInterference {
    fn extend_from(&mut self, v: &[H]) {
        for i in v {
            self.append(i.clone());
        }
    }
}

/// `FUN_select3dinterference(SIX, BDS, lF, l3dF, lFE, lFEresi, l3dFE, l3dFEresi,
/// l2dFE)`: on a non degenerated edge `six`, the 3d FACE/EDGE interferences of `lfe`
/// selected into `l3d_fe` (the residual same domain ones in `lfe_resi`), those with a
/// FACE interference of `lf` of same transition and geometry paired into `l3d_f`, then
/// the 2d ones of `lfe` into `l2d_fe`;
/// `fun_select3d_interference(3, &mut ds, &mut lf, &mut l3df, &mut lfe, &mut resi, &mut l3dfe, &mut l3dferesi, &mut l2dfe)?`.
#[allow(clippy::too_many_arguments)] // the native signature
pub fn fun_select3d_interference(
    six: i32,
    bds: &mut TopOpeBRepDSDataStructure,
    lf: &mut ListOfInterference,
    l3d_f: &mut ListOfInterference,
    lfe: &mut ListOfInterference,
    lfe_resi: &mut ListOfInterference,
    l3d_fe: &mut ListOfInterference,
    l3d_fe_resi: &mut ListOfInterference,
    l2d_fe: &mut ListOfInterference,
) -> Result<(), Failure> {
    let e = topods::edge(bds.shape(six, true))?;
    if BRepTool::degenerated(&e)? {
        return Ok(());
    }
    l3d_f.clear();
    l3d_fe.clear();
    super::select3d::fun_select3d_i(six, bds, lfe, lfe_resi, l3d_fe)?;
    super::select3d::fun_select3d_ise_same_isf(lfe, l3d_fe, l3d_fe_resi, lf, l3d_f)?;
    fun_select2d_i(six, bds, ShapeEnum::Face, lfe, l2d_fe)?;
    Ok(())
}
