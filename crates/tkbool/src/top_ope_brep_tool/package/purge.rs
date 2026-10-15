// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_PURGE.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (the commented-out native code is left out).

//! `TopOpeBRepTool::PurgeClosingEdges`, `CorrectONUVISO`, `MakeFaces` and
//! `FUN_tool_ClosedW`: the faulty closing edges of split faces of a closed face, the
//! correction of their UV representations by a period and the faces rebuilt without
//! the faulty shapes.
use crate::top_ope_brep_tool::corriso::{
    CORRISO_DECREASE, CORRISO_INCREASE, EdgeFlagMap, TopOpeBRepToolCORRISO,
};
use crate::top_ope_brep_tool::reguw::fun_tool_add;
use crate::top_ope_brep_tool::tool::{ShapeListMap, TopOpeBRepToolTOOL};
use crate::top_ope_brep_tool::topology::fun_tool_closed_s_flag;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// A shape oriented FORWARD when it is INTERNAL or EXTERNAL.
fn oriented_shape(e: &Shape) -> Shape {
    if matches!(
        e.orientation(),
        Orientation::Internal | Orientation::External
    ) {
        e.oriented(Orientation::Forward)
    } else {
        e.clone()
    }
}

/// `FUN_getv(orivine, e, v)` (static): the first vertex of the edge (FORWARD for a
/// non-oriented edge) with the orientation.
fn fun_getv(orivine: Orientation, e: &Shape) -> Result<Option<Shape>, Failure> {
    let eo = oriented_shape(e);
    Ok(Explorer::new(&eo, ShapeEnum::Vertex, ShapeEnum::Shape)?
        .find(|v| v.orientation() == orivine))
}

/// The edges of a wire per vertex orientation: FORWARD, REVERSED and INTERNAL.
fn vertex_maps(w: &Wire) -> Result<[ShapeListMap; 3], Failure> {
    let mut maps = [
        ShapeListMap::default(),
        ShapeListMap::default(),
        ShapeListMap::default(),
    ];
    for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
        for v in Explorer::new(&oriented_shape(&e), ShapeEnum::Vertex, ShapeEnum::Shape)? {
            match v.orientation() {
                Orientation::Forward => fun_tool_add(&mut maps[0], &v, &e),
                Orientation::Reversed => fun_tool_add(&mut maps[1], &v, &e),
                Orientation::Internal => fun_tool_add(&mut maps[2], &v, &e),
                Orientation::External => {}
            }
        }
    }
    Ok(maps)
}

/// The check of a vertex met in one sense only (`FUN_tool_ClosedW`); false for a
/// faulty wire.
fn check_lonely_vertex(
    vtocheck: &Shape,
    v_fine: bool,
    [map_fine, map_rine, map_iine]: &[ShapeListMap; 3],
    mapvok: &Map<Shape, ShapeMapHasher>,
) -> Result<bool, Failure> {
    let eds = if v_fine {
        map_fine.seek(vtocheck)
    } else {
        map_rine.seek(vtocheck)
    };
    let eds = eds.cloned().unwrap_or_default();
    if eds.extent() > 1 {
        return Ok(false);
    }
    let Some(e) = eds.first() else {
        return Ok(false);
    };
    let ovori = if v_fine {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    let Some(ov) = fun_getv(ovori, e)? else {
        return Ok(false);
    };
    if matches!(
        e.orientation(),
        Orientation::Forward | Orientation::Reversed
    ) || !mapvok.contains(&ov)
    {
        return Ok(false);
    }
    if map_iine.is_bound(&ov) {
        return Ok(true);
    }
    // Natively both tests read the REVERSED map.
    if !map_rine.is_bound(&ov) {
        return Ok(false);
    }
    let ovf = map_fine.seek(&ov).cloned().unwrap_or_default();
    let ovr = map_rine.seek(&ov).cloned().unwrap_or_default();
    if ovf.extent() > 1 || ovr.extent() > 1 {
        return Ok(true);
    }
    let same_f = ovf.first().is_some_and(|x| x.is_equal(e));
    let same_r = ovr.first().is_some_and(|x| x.is_equal(e));
    Ok(!(same_f || same_r))
}

/// `FUN_tool_ClosedW(W)`: whether the wire is closed: each vertex met FORWARD only (or
/// REVERSED only) must close through an INTERNAL or EXTERNAL edge; false for an empty
/// wire; `fun_tool_closed_w(&w)?`.
pub fn fun_tool_closed_w(w: &Wire) -> Result<bool, Failure> {
    let maps = vertex_maps(w)?;
    if maps[0].extent() == 0 {
        return Ok(false);
    }
    let mut mapvok: Map<Shape, ShapeMapHasher> = Map::default();
    let mut only_f: Vec<Shape> = Vec::new();
    for (v, _) in maps[0].items() {
        if maps[2].is_bound(v) || maps[1].is_bound(v) {
            mapvok.add(v.clone());
            continue;
        }
        only_f.push(v.clone());
    }
    let mut only_r: Vec<Shape> = Vec::new();
    for (v, _) in maps[1].items() {
        if mapvok.contains(v) {
            continue;
        }
        if maps[2].is_bound(v) {
            mapvok.add(v.clone());
            continue;
        }
        only_r.push(v.clone());
    }
    // Natively the lists are maps: their order is the map order.
    let only_f = map_order(&only_f);
    let only_r = map_order(&only_r);
    for (v_fine, list) in [(true, &only_f), (false, &only_r)] {
        for vtocheck in list {
            if !check_lonely_vertex(vtocheck, v_fine, &maps, &mapvok)? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// The shapes in the iteration order of a `NCollection_Map` filled in that order.
fn map_order(shapes: &[Shape]) -> Vec<Shape> {
    let mut m: Map<Shape, ShapeMapHasher> = Map::default();
    for s in shapes {
        m.add(s.clone());
    }
    m.iter().cloned().collect()
}

/// The closing edges of the wire of the UV corrector (closed on the face or iso on
/// its closing line); `None` when an edge has no UV representation.
fn closing_edges(
    corriso: &TopOpeBRepToolCORRISO,
    fin: &Face,
    (in_u, xmin, xper, tolx): (bool, f64, f64, f64),
) -> Result<Option<List<Shape>>, Failure> {
    let mut c_eds: List<Shape> = List::default();
    for es in corriso.eds().iter() {
        let e = topods::edge(es)?;
        let mut closing = BRepTool::is_closed_on_face(&e, fin)?;
        if !closing {
            // xpu231198: the p-curve was modified, the information is lost.
            let Some(c2df) = corriso.uv_rep(es) else {
                return Ok(None);
            };
            closing = TopOpeBRepToolTOOL::ison_clo_c2df(&c2df, in_u, xmin, xper, tolx)?;
        }
        if closing {
            c_eds.append(es.clone());
        }
    }
    Ok(Some(c_eds))
}

/// `TopOpeBRepTool::PurgeClosingEdges(Fin, FF, MWisOld, MshNOK)`: for each wire of
/// the split face not closed in UV, its faulty closing edges, the wire and the face
/// recorded as not valid; the native loop stops (true) at the first wire with at most
/// one closing edge; false for a face not closed;
/// `purge_closing_edges_face(&fin, &ff, &mwisold, &mut nok)?`.
pub fn purge_closing_edges_face(
    fin: &Face,
    ff: &Face,
    _mwis_old: &DataMap<Shape, i32, ShapeMapHasher>,
    msh_nok: &mut IndexedMap<Shape>,
) -> Result<bool, Failure> {
    let mut corriso = TopOpeBRepToolCORRISO::with_face(fin)?;
    let tol_f = BRepTool::face_tolerance(fin)?;
    let (uclosed, uperiod) = corriso.refclosed(1).unwrap_or_default();
    let (vclosed, vperiod) = corriso.refclosed(2).unwrap_or_default();
    if !uclosed && !vclosed {
        return Ok(false);
    }
    let in_u = uclosed;
    let gas = corriso.gas_ref();
    let xmin = if in_u {
        gas.first_u_parameter()?
    } else {
        gas.first_v_parameter()?
    };
    let xper = if in_u { uperiod } else { vperiod };
    let tolx = corriso.tol(if in_u { 1 } else { 2 }, tol_f)?;
    for w in Explorer::new(ff, ShapeEnum::Wire, ShapeEnum::Shape)? {
        corriso.init(&w)?;
        if corriso.uv_closed()? {
            continue;
        }
        let Some(c_eds) = closing_edges(&corriso, fin, (in_u, xmin, xper, tolx))? else {
            return Ok(false);
        };
        if c_eds.extent() <= 1 {
            return Ok(true);
        }
        let mut fy_eds: List<Shape> = List::default();
        if corriso.purge_fy_closing_e(&c_eds, &mut fy_eds)? {
            for e in fy_eds.iter() {
                msh_nok.add(e.clone());
            }
            msh_nok.add(w.clone());
            msh_nok.add(ff.clone().into_shape());
        }
    }
    Ok(true)
}

/// `TopOpeBRepTool::PurgeClosingEdges(Fin, LOF, MWisOld, MshNOK)`: on each split face
/// of a closed face; `purge_closing_edges(&fin, &faces, &mwisold, &mut nok)?`.
pub fn purge_closing_edges(
    fin: &Face,
    lof: &List<Shape>,
    mwis_old: &DataMap<Shape, i32, ShapeMapHasher>,
    msh_nok: &mut IndexedMap<Shape>,
) -> Result<bool, Failure> {
    if !fun_tool_closed_s_flag(fin)? {
        return Ok(true);
    }
    for f in lof.iter() {
        if !purge_closing_edges_face(fin, &topods::face(f)?, mwis_old, msh_nok)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The translation code of an edge connected one period away to the faulty edge.
fn period_code(
    corriso: &TopOpeBRepToolCORRISO,
    fy: &Shape,
    ii: i32,
    on_u: bool,
    (xperiod, xtol): (f64, f64),
) -> Result<Option<Vec<i32>>, Failure> {
    let fye = topods::edge(fy)?;
    let Some(c2df) = corriso.uv_rep(fy) else {
        return Ok(None);
    };
    let vff = TopOpeBRepToolTOOL::vertex(ii, &fye)?;
    let uvff = TopOpeBRepToolTOOL::uvf(TopOpeBRepToolTOOL::par_e(ii, &fye)?, &c2df)?;
    let Some(loe) = corriso.connexity(&vff) else {
        return Ok(None);
    };
    let mut codes = Vec::new();
    for es in loe.iter() {
        let ee = topods::edge(es)?;
        let vee = TopOpeBRepToolTOOL::vertices(&ee)?;
        for (k, ve) in vee.iter().enumerate() {
            let ive = k as i32 + 1;
            if !ve.is_same(&vff) || ive == ii {
                continue;
            }
            let Some(c2dfe) = corriso.uv_rep(es) else {
                return Ok(None);
            };
            let uve = TopOpeBRepToolTOOL::uvf(TopOpeBRepToolTOOL::par_e(ive, &ee)?, &c2dfe)?;
            let dxx = if on_u {
                uve.x() - uvff.x()
            } else {
                uve.y() - uvff.y()
            };
            if (xperiod - dxx.abs()).abs() < xtol {
                codes.push(if dxx > 0. {
                    CORRISO_INCREASE
                } else {
                    CORRISO_DECREASE
                });
            }
        }
    }
    Ok(Some(codes))
}

/// `FUN_connexX(onU, CORRISO, EdstoCheck, fyEds)` (static): the edge with both UV ends
/// faulty translated by the period towards an edge connected one period away.
fn fun_connex_x(
    on_u: bool,
    corriso: &TopOpeBRepToolCORRISO,
    eds_to_check: &List<Shape>,
    fy_eds: &mut EdgeFlagMap,
) -> Result<bool, Failure> {
    fy_eds.clear(false);
    let tol_f = BRepTool::face_tolerance(corriso.fref())?;
    let index = if on_u { 1 } else { 2 };
    let (xclosed, xperiod) = corriso.refclosed(index).unwrap_or_default();
    if !xclosed {
        return Ok(false);
    }
    let xtol = corriso.tol(index, tol_f)?;
    let Some((fy, _)) = corriso.first_edge_with_faulty_uv(eds_to_check, 2)? else {
        return Ok(false);
    };
    if corriso.uv_rep(&fy).is_none() {
        return Ok(false);
    }
    for ii in 1..=2 {
        let Some(codes) = period_code(corriso, &fy, ii, on_u, (xperiod, xtol))? else {
            return Ok(false);
        };
        for code in codes {
            fy_eds.bind(fy.clone(), code);
        }
    }
    Ok(!fy_eds.is_empty())
}

/// `TopOpeBRepTool::CorrectONUVISO(Fin, Fsp)`: a split face of a closed face not closed
/// in UV corrected by translating its faulty edges by a period (connected ones first,
/// then the ones out of the bounds); false when it stays faulty;
/// `correct_on_uviso(&fin, &mut fsp)?`.
pub fn correct_on_uviso(fin: &Face, fsp: &mut Face) -> Result<bool, Failure> {
    let tol_f = BRepTool::face_tolerance(fin)?;
    let mut corriso = TopOpeBRepToolCORRISO::with_face(fin)?;
    let (uclosed, _) = corriso.refclosed(1).unwrap_or_default();
    let (vclosed, _) = corriso.refclosed(2).unwrap_or_default();
    if !uclosed && !vclosed {
        return Ok(false);
    }
    corriso.init(fsp)?;
    if corriso.uv_closed()? {
        return Ok(true);
    }
    for i in 1..=2 {
        let on_u = i == 1;
        let tocheck = corriso.eds().clone();
        let mut fy_eds = EdgeFlagMap::default();
        if !fun_connex_x(on_u, &corriso, &tocheck, &mut fy_eds)?
            || !corriso.trsl_uv(on_u, &fy_eds)?
            || !corriso.uv_closed()?
        {
            continue;
        }
        return finish_correction(&corriso, fsp);
    }
    for i in 1..=2 {
        let on_u = i == 1;
        let (xclosed, _) = corriso.refclosed(i).unwrap_or_default();
        if !xclosed {
            continue;
        }
        let tolx = corriso.tol(i, tol_f)? * 1.0e2; // BUC60380
        let mut fy_eds = EdgeFlagMap::default();
        let eds = corriso.eds().clone();
        if !corriso.edges_out_of_bounds_uv(&eds, on_u, tolx, &mut fy_eds)? {
            continue;
        }
        if !corriso.trsl_uv(on_u, &fy_eds)? {
            return Ok(false);
        }
        if !corriso.uv_closed()? {
            continue;
        }
        return finish_correction(&corriso, fsp);
    }
    Ok(false)
}

/// The corrected face stored in `fsp` (nullified when it is not a face).
fn finish_correction(corriso: &TopOpeBRepToolCORRISO, fsp: &mut Face) -> Result<bool, Failure> {
    match corriso.getnew_s()? {
        Some(f) => {
            *fsp = f;
            Ok(true)
        }
        None => {
            *fsp = Face::default();
            Ok(false)
        }
    }
}

/// The wire without the edges not valid; `None` when no edge is left.
fn purged_wire(w: &Shape, msh_nok: &IndexedMap<Shape>) -> Result<Option<Wire>, Failure> {
    let bb = BRepBuilder::default();
    let mut new_wire = Wire::default();
    bb.make_wire(&mut new_wire);
    let mut ne = 0;
    for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if msh_nok.contains(&e) {
            continue;
        }
        bb.add(&mut new_wire, &e)?;
        ne += 1;
    }
    if ne == 0 {
        return Ok(None);
    }
    let closed = fun_tool_closed_w(&new_wire)?;
    new_wire.set_closed(closed)?;
    Ok(Some(new_wire))
}

/// `TopOpeBRepTool::MakeFaces(Fin, LOF, MshNOK, LOFF)`: the valid split faces kept,
/// the others rebuilt on an empty copy of `Fin` from their valid wires and the valid
/// edges of their other wires; `make_faces(&fin, &faces, &nok, &mut out)?`.
pub fn make_faces(
    fin: &Face,
    lof: &List<Shape>,
    msh_nok: &IndexedMap<Shape>,
    loff: &mut List<Shape>,
) -> Result<bool, Failure> {
    let bb = BRepBuilder::default();
    loff.clear();
    for ff in lof.iter() {
        topods::face(ff)?;
        if !msh_nok.contains(ff) {
            loff.append(ff.clone());
            continue;
        }
        let mut new_face = topods::face(&fin.empty_copied()?)?;
        for w in Explorer::new(ff, ShapeEnum::Wire, ShapeEnum::Shape)? {
            if !msh_nok.contains(&w) {
                bb.add(&mut new_face, &w)?;
                continue;
            }
            if let Some(new_wire) = purged_wire(&w, msh_nok)? {
                bb.add(&mut new_face, &new_wire)?;
            }
        }
        loff.append(new_face.into_shape());
    }
    Ok(true)
}
