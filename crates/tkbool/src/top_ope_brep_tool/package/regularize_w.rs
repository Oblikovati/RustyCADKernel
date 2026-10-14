// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_RegularizeW.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (the commented-out native code is left out).

//! `TopOpeBRepTool::RegularizeWires`, `Regularize`, `RegularizeFace` and
//! `FUN_tool_ClassifW`: the wires of a face split into closed UV loops, then sorted
//! into outer wires holding their holes, one face per outer wire.
use crate::top_ope_brep_tool::classi::{
    CLASSI_DIFF, CLASSI_ONE_IN_TWO, CLASSI_TWO_IN_ONE, TopOpeBRepToolCLASSI, fun_add_owlw,
};
use crate::top_ope_brep_tool::reguw::TopOpeBRepToolREGUW;
use crate::top_ope_brep_tool::tool::{ShapeListMap, TopOpeBRepToolTOOL};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `TopOpeBRepTool::RegularizeWires(theFace, mapoldWnewW, ESplits)`: each wire of the
/// face regularized (`TopOpeBRepTool_REGUW`), its new wires stored by old wire and the
/// splits of its edges by edge; false for a null face or a failed wire;
/// `regularize_wires(&f, &mut ow_nw, &mut esplits)?`.
pub fn regularize_wires(
    the_face: &Face,
    mapold_wnew_w: &mut ShapeListMap,
    e_splits: &mut ShapeListMap,
) -> Result<bool, Failure> {
    if the_face.is_null() {
        return Ok(false);
    }
    let a_face = topods::face(&the_face.oriented(Orientation::Forward))?;
    let mut reguw = TopOpeBRepToolREGUW::new(&a_face)?;
    reguw.set_ow_nw(mapold_wnew_w);
    reguw.set_esplits(e_splits);
    for w in Explorer::new(&a_face, ShapeEnum::Wire, ShapeEnum::Shape)? {
        reguw.init(&w)?;
        if !reguw.map_s()? || !reguw.split_eds()? || !reguw.regu()? {
            return Ok(false);
        }
    }
    *e_splits = reguw.get_esplits()?;
    *mapold_wnew_w = reguw.get_ow_nw()?;
    Ok(true)
}

/// `TopOpeBRepTool::Regularize(theFace, aListOfFaces, ESplits)`: the wires regularized
/// then the faces of the new wires (a regularizer is built first natively and kept
/// unused); `regularize(&f, &mut faces, &mut esplits)?`.
pub fn regularize(
    the_face: &Face,
    a_list_of_faces: &mut List<Shape>,
    e_splits: &mut ShapeListMap,
) -> Result<bool, Failure> {
    let _reguw = TopOpeBRepToolREGUW::new(the_face)?;
    a_list_of_faces.clear();
    let mut mapold_wnew_w = ShapeListMap::default();
    if !regularize_wires(the_face, &mut mapold_wnew_w, e_splits)? {
        return Ok(false);
    }
    regularize_face(the_face, &mapold_wnew_w, a_list_of_faces)
}

/// The wire is still to classify: a key of the result or not yet done.
fn pending(map_wlow: &ShapeListMap, mapdone: &Map<Shape, ShapeMapHasher>, w: &Shape) -> bool {
    map_wlow.is_bound(w) || !mapdone.contains(w)
}

/// The list of the smaller wire moved into the list of the greater one.
fn move_lower(
    map_wlow: &mut ShapeListMap,
    greater: &Shape,
    smaller: &Shape,
) -> Result<(), Failure> {
    let moved = map_wlow
        .change_seek(smaller)
        .map(std::mem::take)
        .unwrap_or_default();
    let lgre = map_wlow.change_seek(greater).ok_or_else(|| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "FUN_tool_ClassifW: greater wire not bound; expected a bound wire",
            "TopOpeBRepTool",
        )
    })?;
    for w in moved.iter() {
        lgre.append(w.clone());
    }
    map_wlow.unbind(smaller);
    Ok(())
}

/// The new wires of two old wires one inside the other classified pairwise (the
/// smaller inside the greater moved into it).
fn classify_pairs(
    classi: &mut TopOpeBRepToolCLASSI,
    (lsma, lgre): (&List<Shape>, &List<Shape>),
    toluv: f64,
    (map_wlow, mapdone): (&mut ShapeListMap, &Map<Shape, ShapeMapHasher>),
) -> Result<bool, Failure> {
    for wsma in lsma.iter() {
        if !pending(map_wlow, mapdone, wsma) {
            continue;
        }
        for wgre in lgre.iter() {
            if !pending(map_wlow, mapdone, wgre) {
                continue;
            }
            let stabnd2d = classi.classi_bnd2d(wsma, wgre, toluv, true)?;
            match classi.classip2d(wsma, wgre, stabnd2d)? {
                CLASSI_DIFF => continue,
                CLASSI_ONE_IN_TWO => move_lower(map_wlow, wgre, wsma)?,
                CLASSI_TWO_IN_ONE => move_lower(map_wlow, wsma, wgre)?,
                _ => return Ok(false),
            }
        }
    }
    Ok(true)
}

/// The first old wire compared with the next ones: `Ok(None)` when it is apart from
/// all of them, the other wire and the classification else; `Err(())` when two wires
/// cannot be classified.
fn first_relation(
    classi: &mut TopOpeBRepToolCLASSI,
    l_ows: &[Shape],
    toluv: f64,
    (map_wlow, mapdone): (&ShapeListMap, &Map<Shape, ShapeMapHasher>),
) -> Result<Result<(bool, Shape, i32), ()>, Failure> {
    let ow1 = &l_ows[0];
    let (mut out_all, mut ow2, mut sta12) = (false, Shape::default(), 0);
    for w in &l_ows[1..] {
        ow2 = w.clone();
        if !pending(map_wlow, mapdone, &ow2) {
            continue;
        }
        let stabnd2d12 = classi.classi_bnd2d(ow1, &ow2, toluv, true)?;
        sta12 = classi.classip2d(ow1, &ow2, stabnd2d12)?;
        if sta12 == CLASSI_DIFF {
            out_all = true;
            continue;
        }
        if sta12 != CLASSI_ONE_IN_TWO && sta12 != CLASSI_TWO_IN_ONE {
            return Ok(Err(()));
        }
        break;
    }
    Ok(Ok((out_all, ow2, sta12)))
}

/// `FUN_tool_ClassifW(F, mapOwNw, mapWlow)`: the new wires sorted into outer wires
/// (keys) holding the wires inside them; with one old wire by `CLASSI::Classilist`,
/// else old wire against old wire; false when two wires cannot be classified;
/// `fun_tool_classif_w(&f, &ow_nw, &mut wlow)?`.
pub fn fun_tool_classif_w(
    f: &Face,
    map_ow_nw: &ShapeListMap,
    map_wlow: &mut ShapeListMap,
) -> Result<bool, Failure> {
    let tol_f = BRepTool::face_tolerance(f)?;
    let toluv = TopOpeBRepToolTOOL::tol_uv(f, tol_f)?;
    let ffor = topods::face(&f.oriented(Orientation::Forward))?;
    let mut classi = TopOpeBRepToolCLASSI::new();
    classi.init2d(&ffor);
    let nold_w = map_ow_nw.extent();
    if nold_w == 1 {
        let low = map_ow_nw
            .items()
            .next()
            .map(|(_, l)| l.clone())
            .unwrap_or_default();
        return classi.classilist(&low, map_wlow);
    }
    let mut l_ows: Vec<Shape> = Vec::new();
    for (owi, low) in map_ow_nw.items() {
        l_ows.push(owi.clone());
        let mut lwresu: List<Shape> = List::default();
        fun_add_owlw(owi, low, &mut lwresu);
        for w in lwresu.iter() {
            map_wlow.bind(w.clone(), List::default());
        }
    }
    let mut mapdone: Map<Shape, ShapeMapHasher> = Map::default();
    while !l_ows.is_empty() {
        let ow1 = l_ows[0].clone();
        if !pending(map_wlow, &mapdone, &ow1) {
            // Natively the loop continues without removing the wire: it never ends.
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "FUN_tool_ClassifW: an old wire already done and not bound; the native loop does not end",
                "TopOpeBRepTool",
            ));
        }
        let lw1 = map_ow_nw.seek(&ow1).cloned().unwrap_or_default();
        if l_ows.len() == 1 {
            break;
        }
        let (out_all, ow2, sta12) =
            match first_relation(&mut classi, &l_ows, toluv, (map_wlow, &mapdone))? {
                Ok(found) => found,
                Err(()) => return Ok(false),
            };
        let mut lw1r: List<Shape> = List::default();
        fun_add_owlw(&ow1, &lw1, &mut lw1r);
        if out_all {
            for w in lw1r.iter() {
                mapdone.add(w.clone());
            }
            l_ows.remove(0);
            continue;
        }
        let lw2 = map_ow_nw.seek(&ow2).cloned().unwrap_or_default();
        let mut lw2r: List<Shape> = List::default();
        fun_add_owlw(&ow2, &lw2, &mut lw2r);
        let (lgre, lsma) = match sta12 {
            CLASSI_ONE_IN_TWO => (lw2r, lw1r),
            CLASSI_TWO_IN_ONE => (lw1r, lw2r),
            _ => (List::default(), List::default()),
        };
        if !classify_pairs(&mut classi, (&lsma, &lgre), toluv, (map_wlow, &mapdone))? {
            return Ok(false);
        }
        l_ows.remove(0);
    }
    Ok(true)
}

/// `TopOpeBRepTool::RegularizeFace(theFace, mapoldWnewW, newFaces)`: the new wires
/// classified then one face per outer wire (`TopOpeBRepTool_TOOL::WireToFace`);
/// `regularize_face(&f, &ow_nw, &mut faces)?`.
pub fn regularize_face(
    the_face: &Face,
    mapold_wnew_w: &ShapeListMap,
    new_faces: &mut List<Shape>,
) -> Result<bool, Failure> {
    let mut map_wlow = ShapeListMap::default();
    let a_face = topods::face(&the_face.oriented(Orientation::Forward))?;
    if !fun_tool_classif_w(&a_face, mapold_wnew_w, &mut map_wlow)? {
        return Ok(false);
    }
    TopOpeBRepToolTOOL::wire_to_face(the_face, &map_wlow, new_faces)
}
