// Rust translation of BRepFill_OffsetWire::UpdateDetromp and of the static TrimEdge,
// StoreInMap, IsInnerEdge and DoubleOrNotInside
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trimming of the parallels: the "proofing" (Detromp) of the vertices bounding the
//! parts of the bisectors between the spine and the offset, and the parallel edges cut
//! between the vertices it keeps.
use super::BRepFillOffsetWire;
use crate::brep_fill::BRepFillTrimEdgeTool;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::bisector::Bisec;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;

/// The proofing of each spine element (default-hashed, orientation sensitive).
pub(super) type Detromp = DataMap<Shape, List<Shape>>;

/// The vertices merged by the trimming: old vertex to the vertex replacing it.
pub(super) type MergedVertices = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

/// The bisector of an arc, its points and the trim tool, as UpdateDetromp reads them.
pub(super) struct BisectorPoints<'a> {
    /// The vertices on the bisector.
    pub vertices: &'a [Shape],
    /// Their parameters (on the bisector, on the first and on the second parallel).
    pub params: &'a [Point3],
    /// The bisector.
    pub bisec: &'a Bisec,
    /// Whether the first (last) point of the bisector is on the offset.
    pub on_edge: (bool, bool),
    /// The trim tool of the bisector.
    pub trim: &'a BRepFillTrimEdgeTool,
}

/// The vertex appended to the proofing of both elements.
fn append_both(
    detromp: &mut Detromp,
    (shape1, shape2): (&Shape, &Shape),
    v: &Shape,
) -> Result<(), Failure> {
    detromp
        .change_seek(shape1)
        .ok_or_else(|| not_bound(shape1))?
        .append(v.clone());
    detromp
        .change_seek(shape2)
        .ok_or_else(|| not_bound(shape2))?
        .append(v.clone());
    Ok(())
}

impl BRepFillOffsetWire {
    /// `UpdateDetromp(Detromp, Shape1, Shape2, Vertices, Params, Bisec, SOnE, EOnE,
    /// Trim)` (private): every vertex for the Intersection join; for the Arc join the
    /// ends of each interval of the bisector whose middle is outside the offset zone
    /// (all of them for a closed periodic bisector).
    pub(super) fn update_detromp(
        &self,
        detromp: &mut Detromp,
        shapes: (&Shape, &Shape),
        b: &BisectorPoints<'_>,
    ) -> Result<(), Failure> {
        if self.my_join_type == JoinType::Intersection {
            for v in b.vertices {
                append_both(detromp, shapes, v)?;
            }
            return Ok(());
        }
        let bis_handle: Handle<Geom2dTrimmedCurve> = b.bisec.value();
        let bis = bis_handle.get().ok_or_else(null_bisector)?;
        let force_add: bool = closed_periodic(bis)?;
        let mut ii: usize = 0;
        let mut u1: f64 = bis.first_parameter();
        let mut v1: Shape = Shape::default();
        if b.on_edge.0 {
            // The first point of the bisector is on the offset.
            v1 = b.vertices[0].clone();
            ii += 1;
        }
        while ii < b.vertices.len() && ii < b.params.len() {
            let (u2, v2) = (b.params[ii].x(), b.vertices[ii].clone());
            if !b.trim.is_inside(bis.value((u2 + u1) * 0.5)?)? || force_add {
                if !v1.is_null() {
                    append_both(detromp, shapes, &v1)?;
                }
                append_both(detromp, shapes, &v2)?;
            }
            (u1, v1) = (u2, v2);
            ii += 1;
        }
        self.close_detromp(detromp, shapes, b, (bis, u1, &v1), force_add)
    }

    /// The last interval: from the last vertex to the end of the bisector (an infinite
    /// end keeps the vertex) when the end is not on the offset.
    fn close_detromp(
        &self,
        detromp: &mut Detromp,
        shapes: (&Shape, &Shape),
        b: &BisectorPoints<'_>,
        (bis, u1, v1): (&Geom2dTrimmedCurve, f64, &Shape),
        force_add: bool,
    ) -> Result<(), Failure> {
        let u2: f64 = bis.last_parameter();
        if b.on_edge.1 {
            return Ok(());
        }
        let keep: bool = precision::is_infinite(u2)
            || !b.trim.is_inside(bis.value((u2 + u1) * 0.5)?)?
            || force_add;
        if keep && !v1.is_null() {
            append_both(detromp, shapes, v1)?;
        }
        Ok(())
    }
}

/// Whether the bisector is a trimmed periodic curve whose ends coincide.
fn closed_periodic(bis: &Geom2dTrimmedCurve) -> Result<bool, Failure> {
    let basis: Handle<dyn Geom2dCurve> = bis.basis_curve();
    if !basis.get().is_some_and(|c| c.is_periodic()) {
        return Ok(false);
    }
    let pf: Point2 = bis.value(bis.first_parameter())?;
    let pl: Point2 = bis.value(bis.last_parameter())?;
    Ok(pf.distance(pl) <= precision::confusion())
}

/// `StoreInMap(V1, V2, MapVV)`: V1 replaced by V2 (or by what replaces V2), the
/// vertices already replaced by V1 redirected.
fn store_in_map(v1: &Shape, v2: &Shape, map_vv: &mut MergedVertices) -> Result<(), Failure> {
    let new_v: Shape = map_vv.seek(v2).cloned().unwrap_or_else(|| v2.clone());
    if let Some(old) = map_vv.change_seek(v1) {
        *old = new_v.clone();
    }
    for i in 1..=map_vv.extent() {
        let value = map_vv.change_from_index(i)?;
        if value.is_same(v1) {
            *value = new_v.clone();
        }
    }
    map_vv.add(v1.clone(), new_v);
    Ok(())
}

/// `DoubleOrNotInside(LV, V)`: whether the vertex appears twice in the list, or not at
/// all.
fn double_or_not_inside(lv: &List<Shape>, v: &Shape) -> bool {
    let mut vu = false;
    for s in lv.iter() {
        if v.is_same(s) {
            if vu {
                return true;
            }
            vu = true;
        }
    }
    !vu
}

/// `IsInnerEdge(ProE, AllSpine, TrPar1, TrPar2)`: the range of an edge of the spine
/// that does not end at a free vertex (one edge only), `None` otherwise.
fn is_inner_edge(pro_e: &Shape, all_spine: &Face) -> Result<Option<(f64, f64)>, Failure> {
    if pro_e.shape_type()? != ShapeEnum::Edge {
        return Ok(None);
    }
    let an_edge: Edge = topods::edge(pro_e)?;
    let mut ve_map = rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(all_spine, ShapeEnum::Vertex, ShapeEnum::Edge, &mut ve_map)?;
    for i in 1..=ve_map.extent() {
        let le = ve_map.find_from_index(i)?;
        if le.len() == 1 && le.first().is_some_and(|f| an_edge.is_same(f)) {
            return Ok(None);
        }
    }
    Ok(Some(BRepTool::range(&an_edge)?))
}

/// The vertices and parameters of a parallel edge, sorted by parameter (bubble sort),
/// those out of the proofing removed and the doubles merged into MapVV.
fn sort_and_merge(
    e: &Edge,
    detromp: &List<Shape>,
    (the_ver, the_par): (&mut Vec<Shape>, &mut Vec<f64>),
    map_vv: &mut MergedVertices,
) -> Result<(), Failure> {
    let mut change = true;
    while change {
        change = false;
        for i in 1..the_par.len() {
            if the_par[i - 1] > the_par[i] {
                the_par.swap(i - 1, i);
                the_ver.swap(i - 1, i);
                change = true;
            }
        }
    }
    if BRepTool::degenerated(e)? {
        return Ok(());
    }
    let mut k: usize = 0;
    while k < the_ver.len() {
        if double_or_not_inside(detromp, &the_ver[k]) {
            the_ver.remove(k);
            the_par.remove(k);
        } else {
            k += 1;
        }
    }
    merge_doubles(the_ver, the_par, map_vv)
}

/// A vertex equal (or at the same parameter) to the next keeps one representation:
/// the last pair keeps the later one, another pair the earlier one.
fn merge_doubles(
    the_ver: &mut Vec<Shape>,
    the_par: &mut Vec<f64>,
    map_vv: &mut MergedVertices,
) -> Result<(), Failure> {
    let a_par_tol: f64 = 2.0 * precision::p_confusion();
    let mut k: usize = 0;
    while k + 1 < the_ver.len() {
        if !(the_ver[k].is_same(&the_ver[k + 1])
            || (the_par[k] - the_par[k + 1]).abs() <= a_par_tol)
        {
            k += 1;
            continue;
        }
        if k + 2 == the_ver.len() {
            store_in_map(&the_ver[k], &the_ver[k + 1], map_vv)?;
            the_ver.remove(k);
            the_par.remove(k);
        } else {
            store_in_map(&the_ver[k + 1], &the_ver[k], map_vv)?;
            the_ver.remove(k + 1);
            the_par.remove(k + 1);
        }
        // The native loop steps back (k--) before its increment: the same index is
        // tested again.
    }
    Ok(())
}

/// The inputs of TrimEdge for one parallel edge.
pub(super) struct ParallelVertices<'a> {
    /// The parallel edge.
    pub e: &'a Edge,
    /// Its spine element.
    pub pro_e: &'a Shape,
    /// The whole spine.
    pub all_spine: &'a Face,
    /// The proofing of the spine element.
    pub detromp: &'a List<Shape>,
    /// 1 for the first and -1 for the last edge of an open result, 0 otherwise.
    pub ind_of_e: i32,
}

/// `TrimEdge(E, ProE, AllSpine, D, Sv, MapverPar, S, MapVV, IndOfE)`: the parts of the
/// parallel edge between pairs of its kept vertices (from the start, or to the end, of
/// the extreme edges of an open result).
pub(super) fn trim_edge(
    p: &ParallelVertices<'_>,
    the_ver: &mut Vec<Shape>,
    the_par: &mut Vec<f64>,
    map_vv: &mut MergedVertices,
) -> Result<Vec<Shape>, Failure> {
    sort_and_merge(p.e, p.detromp, (the_ver, the_par), map_vv)?;
    if p.ind_of_e == 1 || p.ind_of_e == -1 {
        return Ok(vec![open_end_edge(p, the_ver, the_par)?.into_shape()]);
    }
    let b = BRepBuilder::default();
    let mut s: Vec<Shape> = Vec::new();
    let mut k: usize = 0;
    while k + 1 < the_ver.len() {
        let mut new_edge: Edge = topods::edge(&p.e.empty_copied()?)?;
        let (first, second) = if new_edge.orientation() == Orientation::Reversed {
            (Orientation::Reversed, Orientation::Forward)
        } else {
            (Orientation::Forward, Orientation::Reversed)
        };
        b.add(&mut new_edge, &the_ver[k].oriented(first))?;
        b.add(&mut new_edge, &the_ver[k + 1].oriented(second))?;
        b.range(&new_edge, the_par[k], the_par[k + 1], false)?;
        s.push(new_edge.into_shape());
        k += 2;
    }
    Ok(s)
}

/// A new vertex at the point of the p-curve at a parameter (on Z = 0).
fn pcurve_vertex(pcurve: &Handle<dyn Geom2dCurve>, u: f64) -> Result<Vertex, Failure> {
    let p2 = pcurve.get().ok_or_else(null_bisector)?.value(u)?;
    BRepLibMakeVertex::new(Point3::new(p2.x(), p2.y(), 0.))?.vertex()
}

/// The first (IndOfE 1) or last (-1) edge of an open result: from its free end (moved
/// to the end of the spine edge when it is inner) to its first kept vertex.
fn open_end_edge(
    p: &ParallelVertices<'_>,
    the_ver: &[Shape],
    the_par: &[f64],
) -> Result<Edge, Failure> {
    let b = BRepBuilder::default();
    let mut new_edge: Edge = topods::edge(&p.e.empty_copied()?)?;
    let (mut v1, mut v2) = topexp::vertices(p.e, false);
    let on_surface = BRepTool::curve_on_surface_by_index(p.e, 1)?;
    let (mut fpar, mut lpar) = (on_surface.first, on_surface.last);
    let inner = is_inner_edge(p.pro_e, p.all_spine)?;
    let first_ver: &Shape = the_ver.first().ok_or_else(no_vertex)?;
    let first_par: f64 = *the_par.first().ok_or_else(no_vertex)?;
    let forward: bool = new_edge.orientation() == Orientation::Forward;
    // Forward first edge and reversed last edge start at the free end; the others end
    // there.
    let starts_free: bool = (p.ind_of_e == 1) == forward;
    if starts_free {
        if let Some((tr1, _)) = inner {
            fpar = tr1;
            v1 = pcurve_vertex(&on_surface.pcurve, fpar)?;
        }
        let (o1, o2) = if forward {
            (Orientation::Forward, Orientation::Reversed)
        } else {
            (Orientation::Reversed, Orientation::Forward)
        };
        b.add(&mut new_edge, &v1.oriented(o1))?;
        b.add(&mut new_edge, &first_ver.oriented(o2))?;
        b.range(&new_edge, fpar, first_par, false)?;
        return Ok(new_edge);
    }
    if let Some((_, tr2)) = inner {
        lpar = tr2;
        v2 = pcurve_vertex(&on_surface.pcurve, lpar)?;
    }
    let (o1, o2) = if p.ind_of_e == 1 {
        (Orientation::Reversed, Orientation::Forward)
    } else {
        (Orientation::Forward, Orientation::Reversed)
    };
    b.add(&mut new_edge, &first_ver.oriented(o1))?;
    b.add(&mut new_edge, &v2.oriented(o2))?;
    b.range(&new_edge, first_par, lpar, false)?;
    Ok(new_edge)
}

/// Standard_NoSuchObject of a shape without proofing.
fn not_bound(s: &Shape) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("BRepFill_OffsetWire: no proofing for {s:?}; expected a bound spine element"),
        "NCollection_DataMap",
    )
}

/// Standard_NullObject of a null bisector or p-curve.
fn null_bisector() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_OffsetWire: null bisector or p-curve; expected a curve",
        "BRepFill_OffsetWire",
    )
}

/// Standard_NoSuchObject of an extreme edge without kept vertex.
fn no_vertex() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "BRepFill_OffsetWire: extreme edge of an open result without vertex; expected one",
        "NCollection_Sequence",
    )
}
