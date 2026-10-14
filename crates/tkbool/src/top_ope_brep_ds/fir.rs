// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_FIR.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_FIR`: the face interference reducer, reducing the FACE/EDGE
//! interferences of several faces along one edge to complex face transitions (with the
//! free functions FUN_isPonF, FUN_findPonF, FUN_reduceEDGEgeometry1, FUN_GmapS,
//! FUN_stateedgeface, FUN_reduceEDGEgeometry).
use super::export::{KindIndices, fds_copy, fds_data, fds_data_iter, href, iref};
use super::face_interference_tool::fun_parameters;
use super::process_interferences_tool::select::{to_list, to_vec};
use super::process_interferences_tool::{
    fun_reducedoublons, fun_select_gk_interference, fun_select_sk_interference,
    fun_select_trasha_interference,
};
use super::tool::MapOfShapeOn1State;
use super::{
    ListOfInterference, TopOpeBRepDSCurvePointInterference, TopOpeBRepDSDataStructure,
    TopOpeBRepDSFaceInterferenceTool, TopOpeBRepDSHDataStructure, TopOpeBRepDSInterference,
    TopOpeBRepDSKind, TopOpeBRepDSShapeShapeInterference,
};
use crate::top_ope_brep_tool::{
    fsc_get_psc_with_reference, fun_tool_find_pin_bac, fun_tool_find_pin_e,
    fun_tool_orient_e_in_f_forward,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::{Point2, Point3};

type H = Handle<TopOpeBRepDSInterference>;

/// `FUN_isPonF(LIF, P, BDS, E)`: `p` projects on each support face of `lif` where `e`
/// is not an edge; `fun_is_p_on_f(&lif, p, &ds, &e)?`.
pub fn fun_is_p_on_f(
    lif: &ListOfInterference,
    p: Point3,
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
) -> Result<bool, Failure> {
    let mut pok = true;
    for i in lif.iter() {
        let f = topods::face(bds.shape(fds_data(i).3, true))?;
        if fun_tool_orient_e_in_f_forward(e, &f)?.0 {
            pok = true;
        } else {
            pok = fun_parameters(p, &f)?.is_some();
            if !pok {
                break;
            }
        }
    }
    Ok(pok)
}

/// The parameter of a point interference of the edge for `FUN_findPonF` (`None` when
/// not defined).
fn interference_par(
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    i: &H,
) -> Result<Option<f64>, Failure> {
    let (gt1, g1, _, _) = fds_data(i);
    let cpi: Handle<TopOpeBRepDSCurvePointInterference> = i.cast();
    if let Some(c) = cpi.get() {
        return Ok(Some(c.parameter()));
    }
    let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.cast();
    let Some(ssi) = ssi.get() else {
        return Ok(None);
    };
    if ssi.gbound() {
        let v = topods::vertex(bds.shape(g1, true))?;
        BRepTool::pnt(&v)?;
        return BRepTool::parameter(&v, e).map(Some);
    }
    // natively the point is read but the parameter stays undefined (`pardef` false)
    match gt1 {
        TopOpeBRepDSKind::Point => {
            bds.point(g1)?;
        }
        TopOpeBRepDSKind::Vertex => {
            BRepTool::pnt(&topods::vertex(bds.shape(g1, true))?)?;
        }
        _ => {}
    }
    Ok(None)
}

/// `FUN_findPonF(E, BDS, LIF, P, par)`: a point of the edge (at one of its point
/// interferences, else inside) on the faces of `lif`; `fun_find_p_on_f(&e, &ds, &lif)?`.
pub fn fun_find_p_on_f(
    e: &Edge,
    bds: &TopOpeBRepDSDataStructure,
    lif: &ListOfInterference,
) -> Result<Option<(Point3, f64)>, Failure> {
    let bac = BRepAdaptorCurve::with_edge(e)?;
    let lie = bds.shape_interferences(e, true);
    if lie.is_empty() {
        let (p, par) = fun_tool_find_pin_bac(&bac)?;
        return Ok(fun_is_p_on_f(lif, p, bds, e)?.then_some((p, par)));
    }
    for i in lie.iter() {
        let Some(par) = interference_par(bds, e, i)? else {
            continue;
        };
        let p = bac.value(par)?;
        if fun_is_p_on_f(lif, p, bds, e)? {
            return Ok(Some((p, par)));
        }
    }
    Ok(None)
}

/// The complex transition of `v[a]` with the later interferences of the same edge
/// geometry (removed from `v`), on `ecpx` (`FUN_reduceEDGEgeometry1` inner loop).
fn complex_on(
    bds: &TopOpeBRepDSDataStructure,
    fitool: &mut TopOpeBRepDSFaceInterferenceTool<'_>,
    (fi, ecpx, is_egsp): (&Shape, &Shape, bool),
    v: &mut Vec<H>,
    a: usize,
    (gt1, g1, st1, s1): KindIndices,
) -> Result<(), Failure> {
    let i1 = v[a].clone();
    let f1 = topods::face(bds.shape(s1, true))?.into_shape();
    let mut is_complex = false;
    let mut b = a + 1;
    while b < v.len() {
        let mut i2 = H::default();
        let mut d2: KindIndices = (TopOpeBRepDSKind::Unknown, 0, TopOpeBRepDSKind::Unknown, 0);
        let u2 = fds_data_iter(Some(&v[b]), &mut i2, &mut d2)?;
        let (gt2, g2, st2, s2) = d2;
        if u2 || !(gt2 == gt1 && g2 == g1 && st2 == st1) {
            b += 1;
            continue;
        }
        let f2 = topods::face(bds.shape(s2, true))?.into_shape();
        if !is_complex {
            is_complex = true;
            fitool.init(fi, ecpx, is_egsp, &i1)?;
            fitool.add(fi, &f1, ecpx, is_egsp, &i1)?;
        }
        fitool.add(fi, &f2, ecpx, is_egsp, &i2)?;
        v.remove(b);
    }
    if is_complex {
        fitool.transition(&i1)?;
    }
    Ok(())
}

/// `FUN_reduceEDGEgeometry1(LI, BDS, iFI, iEG, EGsp, MEsp)`: the FACE/EDGE
/// interferences of the face `ifi` on the edge `ieg` (or its split `egsp`) reduced to
/// complex transitions; `li` emptied when no point of the edge lies on the faces;
/// `fun_reduce_edge_geometry1(&mut l, &ds, 1, 3, &Shape::default(), &mesp)?`.
pub fn fun_reduce_edge_geometry1(
    li: &mut ListOfInterference,
    bds: &TopOpeBRepDSDataStructure,
    i_fi: i32,
    i_eg: i32,
    egsp: &Shape,
    _mesp: &MapOfShapeOn1State,
) -> Result<(), Failure> {
    if li.is_empty() {
        return Ok(());
    }
    let fi = topods::face(bds.shape(i_fi, true))?.into_shape();
    let is_egsp = !egsp.is_null();
    let ecpx = topods::edge(if is_egsp { egsp } else { bds.shape(i_eg, true) })?;
    let mut fitool = TopOpeBRepDSFaceInterferenceTool::new(bds);
    if li.extent() >= 2 {
        let pok = if is_egsp {
            Some(fun_tool_find_pin_e(&ecpx)?)
        } else {
            fun_find_p_on_f(&ecpx, bds, li)?
        };
        let Some((p, par)) = pok else {
            li.clear();
            return Ok(());
        };
        fitool.set_edge_pnt_par(p, par);
    }
    let ecpx = ecpx.into_shape();
    let mut v = to_vec(li);
    let mut a = 0;
    while a < v.len() {
        let mut i1 = H::default();
        let mut d1: KindIndices = (TopOpeBRepDSKind::Unknown, 0, TopOpeBRepDSKind::Unknown, 0);
        let u1 = fds_data_iter(Some(&v[a]), &mut i1, &mut d1)?;
        if !u1 && d1.0 == TopOpeBRepDSKind::Edge {
            complex_on(bds, &mut fitool, (&fi, &ecpx, is_egsp), &mut v, a, d1)?;
        }
        a += 1;
    }
    *li = to_list(v);
    Ok(())
}

/// `FUN_GmapS(LI, BDS, mosd)`: the FACE/EDGE interferences grouped by their edge (in
/// order of appearance); `let (edges, lists) = fun_gmap_s(&l, &ds);`.
pub fn fun_gmap_s(
    li: &ListOfInterference,
    bds: &TopOpeBRepDSDataStructure,
) -> (IndexedMap<Shape, ShapeMapHasher>, Vec<ListOfInterference>) {
    let mut keys: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
    let mut lists: Vec<ListOfInterference> = Vec::new();
    for i1 in li.iter() {
        let (gt1, g1, st1, _) = fds_data(i1);
        if gt1 != TopOpeBRepDSKind::Edge || st1 != TopOpeBRepDSKind::Face {
            continue;
        }
        let sg1 = bds.shape(g1, true);
        if !keys.contains(sg1) {
            keys.add(sg1.clone());
            lists.push(ListOfInterference::default());
        }
        let k = keys.find_index(sg1) - 1;
        lists[k].append(i1.clone());
    }
    (keys, lists)
}

/// `FUN_stateedgeface(E, F, P)`: the state in the face of the projection of an inner
/// point of the edge (UNKNOWN when it does not project); with the point;
/// `let (st, p) = fun_stateedgeface(&e, &f)?;`.
pub fn fun_stateedgeface(e: &Shape, f: &Shape) -> Result<(State, Point3), Failure> {
    let (p, _) = fun_tool_find_pin_e(e)?;
    let Some((u, v)) = fun_parameters(p, f)? else {
        return Ok((State::Unknown, p));
    };
    let state = fsc_get_psc_with_reference(f, |psc| -> Result<State, Failure> {
        psc.state_p2d_reference(Point2::new(u, v))?;
        Ok(psc.state())
    })??;
    Ok((state, p))
}

/// The FACE interferences of `lieg` on whose face the split `egsp` lies (IN, or ON
/// for a section edge or an edge with same domain edges), reduced.
#[allow(clippy::too_many_arguments)] // the native locals
fn on_split(
    bds: &TopOpeBRepDSDataStructure,
    lieg: &ListOfInterference,
    egsp: &Shape,
    on_ok: bool,
    (i_fi, i_eg): (i32, i32),
    mesp: &MapOfShapeOn1State,
) -> Result<ListOfInterference, Failure> {
    let mut lisfin = ListOfInterference::default();
    for ilieg in lieg.iter() {
        let ii = iref(ilieg)?;
        if ii.support_type() != TopOpeBRepDSKind::Face {
            continue;
        }
        let (staef, _) = fun_stateedgeface(egsp, bds.shape(ii.support(), true))?;
        if staef == State::In || (on_ok && staef == State::On) {
            lisfin.append(ilieg.clone());
        }
    }
    if lisfin.extent() >= 2 {
        let first = lisfin.first().cloned().unwrap_or_default();
        let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = first.cast();
        let gb = ssi.get().map(|s| s.gbound()).ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "FUN_reduceEDGEgeometry: no shape/shape interference",
                "TopOpeBRepDS_FIR",
            )
        })?;
        if gb {
            lisfin.clear();
            lisfin.append(first);
        } else {
            fun_reduce_edge_geometry1(&mut lisfin, bds, i_fi, i_eg, egsp, mesp)?;
        }
    }
    Ok(lisfin)
}

/// `FUN_reduceEDGEgeometry(LI, BDS, iFI, MEsp)`: the FACE/EDGE interferences of the
/// face `ifi` grouped by edge and reduced (per split ON of a split edge); the others
/// are dropped; `fun_reduce_edge_geometry(&mut l, &ds, 1, &mesp)?`.
pub fn fun_reduce_edge_geometry(
    li: &mut ListOfInterference,
    bds: &TopOpeBRepDSDataStructure,
    i_fi: i32,
    mesp: &MapOfShapeOn1State,
) -> Result<(), Failure> {
    if li.is_empty() {
        return Ok(());
    }
    let (keys, mut lists) = fun_gmap_s(li, bds);
    let mut liout = ListOfInterference::default();
    for (k, lieg) in lists.iter_mut().enumerate() {
        let eg = keys.find_key(k + 1)?.clone();
        let i_eg = bds.shape_index(&eg, true);
        let egissect = bds.is_section_edge(&topods::edge(&eg)?, true);
        let eghasesd = !bds.shape_same_domain_index(i_eg).is_empty();
        match lieg.extent() {
            0 => continue,
            1 => fds_copy(lieg, &mut liout),
            _ => {
                let los1 = mesp.seek(&eg).filter(|l| l.is_split());
                if let Some(los1) = los1 {
                    for egsp in los1.list_on_state().iter() {
                        let lisfin =
                            on_split(bds, lieg, egsp, eghasesd || egissect, (i_fi, i_eg), mesp)?;
                        fds_copy(&lisfin, &mut liout);
                    }
                } else {
                    fun_reduce_edge_geometry1(lieg, bds, i_fi, i_eg, &Shape::default(), mesp)?;
                    fds_copy(lieg, &mut liout);
                }
            }
        }
    }
    *li = liout;
    Ok(())
}

/// The face interference reducer; `let fir = TopOpeBRepDSFIR::new(&hds);`.
pub struct TopOpeBRepDSFIR {
    my_hds: Handle<TopOpeBRepDSHDataStructure>,
}

impl TopOpeBRepDSFIR {
    /// `TopOpeBRepDS_FIR(HDS)`; `TopOpeBRepDSFIR::new(&hds)`.
    pub fn new(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Self {
        Self {
            my_hds: hds.clone(),
        }
    }

    /// `ProcessFaceInterferences(M)`: the interferences of every face reduced;
    /// `fir.process_face_interferences(&m)?`.
    pub fn process_face_interferences(&self, m: &MapOfShapeOn1State) -> Result<(), Failure> {
        let nshape = href(&self.my_hds)?.ds().nb_shapes();
        for i in 1..=nshape {
            let is_face = {
                let bds = href(&self.my_hds)?.ds();
                let s = bds.shape(i, true);
                !s.is_null() && s.shape_type()? == ShapeEnum::Face
            };
            if is_face {
                self.process_face_interferences_index(i, m)?;
            }
        }
        Ok(())
    }

    /// `ProcessFaceInterferences(SIX, MEsp)`: the FACE, FACE/EDGE and EDGE interferences
    /// of the face `six` kept (the FACE/EDGE ones on faces reduced), then the doubles
    /// removed; `fir.process_face_interferences_index(1, &mesp)?`.
    pub fn process_face_interferences_index(
        &self,
        six: i32,
        mesp: &MapOfShapeOn1State,
    ) -> Result<(), Failure> {
        let mut bds = href(&self.my_hds)?.change_ds();
        let mut lw = bds.change_shape_interferences_index(six)?.clone();
        let mut l = [(); 4].map(|_| ListOfInterference::default());
        let [le, lfe, lfef, lf] = &mut l;
        fun_select_trasha_interference(&mut lw, ShapeEnum::Face, lf)?;
        fun_select_gk_interference(lf, TopOpeBRepDSKind::Edge, lfe)?;
        fun_select_sk_interference(lfe, TopOpeBRepDSKind::Face, lfef)?;
        fun_select_trasha_interference(&mut lw, ShapeEnum::Edge, le)?;
        fun_reduce_edge_geometry(lfef, &bds, six, mesp)?;
        let mut li = ListOfInterference::default();
        for part in [&*lf, &*lfe, &*lfef, &*le] {
            fds_copy(part, &mut li);
        }
        fun_reducedoublons(&mut li, &bds, six)?;
        *bds.change_shape_interferences_index(six)? = li;
        Ok(())
    }
}
