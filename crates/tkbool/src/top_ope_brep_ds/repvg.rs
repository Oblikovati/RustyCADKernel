// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_repvg.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reduction of the FACE/FACE interferences of an edge at a point or vertex met by
//! several edges (FDS_repvg, FDS_repvg2).
use super::export::{fds_idata, fds_parameter, fds_state_e_with_f2d, fun_ds_getoov, iref};
use super::process_interferences_tool::select::{to_list, to_vec};
use super::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSEdgeInterferenceTool,
    TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSPoint, TopOpeBRepDSTransition,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

type H = Handle<TopOpeBRepDSInterference>;

/// The geometry of a reduction: a point or a vertex.
enum Geometry {
    Point(TopOpeBRepDSPoint),
    Vertex(Shape),
}

/// The per-interference state of `FDS_repvg2`.
struct Reduction {
    eitool: TopOpeBRepDSEdgeInterferenceTool,
    meme_s: bool,
    trmeme_s: TopOpeBRepDSTransition,
    is_complex: bool,
}

impl Geometry {
    /// `EITool.Add(E, PDS, I)` for a point, `EITool.Add(Ei, VDS, I)` for a vertex.
    fn add(
        &self,
        t: &mut TopOpeBRepDSEdgeInterferenceTool,
        e: &Edge,
        ei: &Edge,
        i: &H,
    ) -> Result<(), Failure> {
        match self {
            Geometry::Point(p) => t.add_point(e, p, i),
            Geometry::Vertex(v) => t.add_vertex(ei, v, i),
        }
    }
}

/// Whether an interference has FACE transitions before and after.
fn face_face(i: &H) -> bool {
    let ((tsb, _, tsa, _), _) = fds_idata(i);
    tsb == ShapeEnum::Face && tsa == ShapeEnum::Face
}

/// The inner loop of `FDS_repvg2` for the interference `v[a]` of geometry `g1`.
#[allow(clippy::too_many_arguments)] // the native locals
fn compare_after(
    bds: &TopOpeBRepDSDataStructure,
    e: &Edge,
    (e1, f1): (&Edge, &topods::Face),
    (g1, s1, n_li): (i32, i32, i32),
    geom: &Geometry,
    v: &mut Vec<H>,
    a: usize,
    r: &mut Reduction,
) -> Result<(), Failure> {
    let i1 = v[a].clone();
    let gt1 = fds_idata(&i1).1.0;
    let mut b = a + 1;
    while b < v.len() {
        let i2 = v[b].clone();
        let (_, (_, g2, _, s2)) = fds_idata(&i2);
        if !face_face(&i2) || g1 != g2 {
            b += 1;
            continue;
        }
        let e2 = topods::edge(bds.shape(s2, true))?;
        if BRepTool::degenerated(&e2)? {
            b += 1;
            continue;
        }
        r.meme_s = s1 == s2 && n_li == 2;
        if !r.is_complex && r.meme_s {
            let pe = fds_parameter(&i1)?;
            if !fds_state_e_with_f2d(bds, e, pe, gt1, g1, f1, &mut r.trmeme_s)? {
                b += 1;
                continue;
            }
            v.remove(b);
        }
        if !r.is_complex && !r.meme_s {
            r.is_complex = true;
            r.eitool.init(e, &i1)?;
            geom.add(&mut r.eitool, e, e1, &i1)?;
        }
        if r.is_complex && !r.meme_s {
            geom.add(&mut r.eitool, e, &e2, &i2)?;
            v.remove(b);
        }
        if r.is_complex && r.meme_s {
            b += 1;
        }
    }
    Ok(())
}

/// The geometry index `g1` of `i1` (its same domain vertex for a vertex of the other
/// rank); `None` to skip `i1`.
fn repvg2_g1(
    bds: &TopOpeBRepDSDataStructure,
    gt: TopOpeBRepDSKind,
    i1: &H,
) -> Result<Option<i32>, Failure> {
    let (_, (_, g1, _, s1)) = fds_idata(i1);
    if gt == TopOpeBRepDSKind::Vertex && bds.ancestor_rank_index(g1) != bds.ancestor_rank_index(s1)
    {
        return Ok(fun_ds_getoov(bds.shape(g1, true), bds).map(|oov| bds.shape_index(&oov, true)));
    }
    Ok(Some(g1))
}

/// The point or vertex `g1`; Standard_Failure for another kind.
fn repvg2_geometry(
    bds: &TopOpeBRepDSDataStructure,
    gt: TopOpeBRepDSKind,
    g1: i32,
) -> Result<Geometry, Failure> {
    match gt {
        TopOpeBRepDSKind::Point => Ok(Geometry::Point(*bds.point(g1)?)),
        TopOpeBRepDSKind::Vertex => Ok(Geometry::Vertex(bds.shape(g1, true).clone())),
        _ => Err(Failure::new("TopOpeBRepDS FDS_repvg2 1")),
    }
}

/// `FDS_repvg2(BDS, EIX, GT, LI, RLI)`: on the non-degenerated edge `eix`, each
/// FACE/FACE interference of geometry kind `gt` compared with the later ones at the
/// same geometry: a pair on the same edge (when the list has two) gets the states of
/// the edge on its face, several edges the accumulated transition; the reduced ones
/// move from `li` to `rli`; `fds_repvg2(&ds, 3, TopOpeBRepDSKind::Vertex, &mut l, &mut r)?`.
pub fn fds_repvg2(
    bds: &TopOpeBRepDSDataStructure,
    eix: i32,
    gt: TopOpeBRepDSKind,
    li: &mut ListOfInterference,
    rli: &mut ListOfInterference,
) -> Result<(), Failure> {
    let e = topods::edge(bds.shape(eix, true))?;
    if BRepTool::degenerated(&e)? {
        return Ok(());
    }
    let n_li = li.extent();
    let mut v = to_vec(li);
    let mut a = 0;
    while a < v.len() {
        let i1 = v[a].clone();
        let ((_, isb1, _, _), (gt1, _, _, s1)) = fds_idata(&i1);
        if !face_face(&i1) || gt1 != gt {
            a += 1;
            continue;
        }
        let Some(g1) = repvg2_g1(bds, gt, &i1)? else {
            a += 1;
            continue;
        };
        let e1 = topods::edge(bds.shape(s1, true))?;
        let f1 = topods::face(bds.shape(isb1, true))?;
        let geom = repvg2_geometry(bds, gt, g1)?;
        if BRepTool::degenerated(&e1)? {
            a += 1;
            continue;
        }
        let mut r = Reduction {
            eitool: TopOpeBRepDSEdgeInterferenceTool::new(),
            meme_s: false,
            trmeme_s: TopOpeBRepDSTransition::default(),
            is_complex: false,
        };
        compare_after(
            bds,
            &e,
            (&e1, &f1),
            (g1, s1, n_li),
            &geom,
            &mut v,
            a,
            &mut r,
        )?;
        if !r.is_complex && r.meme_s {
            r.trmeme_s.set_index(iref(&i1)?.transition().index()?);
            iref(&i1)?.set_transition(&r.trmeme_s);
        } else if r.is_complex && !r.meme_s {
            r.eitool.transition(&i1)?;
        } else {
            a += 1;
            continue;
        }
        rli.append(v.remove(a));
    }
    *li = to_list(v);
    Ok(())
}

/// `FDS_repvg(BDS, EIX, GT, LOI, RLOI)`: the interferences grouped by transition
/// shape, each group of several reduced with `FDS_repvg2` (the groups of one
/// interference are dropped from `loi`, as natively);
/// `fds_repvg(&ds, 3, TopOpeBRepDSKind::Vertex, &mut l, &mut r)?`.
pub fn fds_repvg(
    bds: &TopOpeBRepDSDataStructure,
    eix: i32,
    gt: TopOpeBRepDSKind,
    loi: &mut ListOfInterference,
    rloi: &mut ListOfInterference,
) -> Result<(), Failure> {
    let mut map_itrasha: DataMap<i32, ListOfInterference> = DataMap::default();
    for i in loi.iter() {
        let isa = iref(i)?.transition().index()?;
        match map_itrasha.change_seek(&isa) {
            Some(l) => {
                l.append(i.clone());
            }
            None => {
                let mut l = ListOfInterference::default();
                l.append(i.clone());
                map_itrasha.bind(isa, l);
            }
        }
    }
    loi.clear();
    let keys: Vec<i32> = map_itrasha.items().map(|(k, _)| *k).collect();
    for isa in keys {
        let Some(l) = map_itrasha.change_seek(&isa) else {
            continue;
        };
        if l.extent() < 2 {
            continue;
        }
        let mut rl = ListOfInterference::default();
        fds_repvg2(bds, eix, gt, l, &mut rl)?;
        super::export::fds_copy(l, loi);
        super::export::fds_copy(&rl, rloi);
    }
    Ok(())
}
