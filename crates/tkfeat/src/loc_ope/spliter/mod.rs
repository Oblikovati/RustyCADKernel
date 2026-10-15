// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Spliter.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_Spliter`: the split of a shape by the wires lying on it: the vertices of the
//! wires substituted to the shape vertices they touch, the edges cut at the others, the
//! faces cut by the rebuilt wires, then the faces on the left of the wires found.
mod steps;

use super::{BuildWires, SplitShape, WiresOnShape};
use crate::errors::{construction_error, not_done, null_object};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;

/// The shapes and their descendants (`myMap`).
type Descendants = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `LocOpe_Spliter`: the shape split by projected wires.
///
/// ```
/// use rustycad_tkfeat::loc_ope::Spliter;
/// let spliter = Spliter::new();
/// assert!(!spliter.is_done());
/// ```
#[derive(Clone, Default)]
pub struct Spliter {
    shape: Shape,
    done: bool,
    res: Shape,
    map: Descendants,
    d_left: List<Shape>,
    left: List<Shape>,
}

/// `Put(S, theMap)`: binds `s` and its sub-shapes to themselves.
fn put(s: &Shape, the_map: &mut Descendants) {
    if the_map.is_bound(s) {
        return;
    }
    let mut l = List::default();
    l.append(s.clone());
    the_map.bind(s.clone(), l);
    for child in ShapeIterator::new(s, true, true) {
        put(&child, the_map);
    }
}

/// `RebuildWires(ledge, PW)`: the edges chained into wires.
fn rebuild_wires(ledge: &List<Shape>, pw: &WiresOnShape) -> Result<List<Shape>, Failure> {
    let the_build = BuildWires::with_edges(ledge, pw)?;
    if !the_build.is_done() {
        return Err(construction_error(
            "LocOpe_Spliter::RebuildWires",
            "the wires were not built",
        ));
    }
    Ok(the_build.result()?.clone())
}

/// The located 3D curve of an edge and its range.
fn located_curve(
    e: &Edge,
) -> Result<
    (
        rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
        f64,
        f64,
    ),
    Failure,
> {
    let located = BRepTool::curve_with_location(e)?;
    let mut c = located.curve;
    if !located.location.is_identity() {
        let curve = c
            .get()
            .ok_or_else(|| null_object("LocOpe_Spliter::Select", "edge without 3D curve"))?;
        c = curve.transformed(located.location.transformation())?.cast();
    }
    Ok((c, located.first, located.last))
}

/// `Select(Ebase, lsubs)`: keeps the edge of `lsubs` nearest to the middle of `ebase`
/// (none when no projection exists).
fn select(ebase: &Edge, lsubs: &mut Vec<Shape>) -> Result<(), Failure> {
    // Choix d`un point
    let (c, f, l) = located_curve(ebase)?;
    let pt = c
        .get()
        .ok_or_else(|| null_object("LocOpe_Spliter::Select", "edge without 3D curve"))?
        .value((f + l) / 2.)?;
    let mut proj = ProjectPointOnCurve::new();
    let (mut dmin, mut imin) = (f64::MAX, 0);
    for (i, s) in lsubs.iter().enumerate() {
        let (c, f, l) = located_curve(&topods::edge(s)?)?;
        proj.init_range(pt, &c, f, l)?;
        if proj.nb_points() > 0 && proj.lower_distance()? < dmin {
            imin = i + 1;
            dmin = proj.lower_distance()?;
        }
    }
    if imin == 0 {
        lsubs.clear();
    } else {
        let kept = lsubs[imin - 1].clone();
        *lsubs = vec![kept];
    }
    Ok(())
}

/// The descendant of `e2`'s pieces matching the wire edge `e1` by its vertices (or, for
/// a closed `e1`, by its tangent at the start), oriented as `e1` runs.
fn matching_pieces(e1: &Edge, pieces: &List<Shape>) -> Result<Vec<Shape>, Failure> {
    let (vf1, vl1) = topexp::vertices(e1, false);
    let mut lsubs = Vec::new();
    for s in pieces.iter() {
        let e2 = topods::edge(s)?;
        let (vf2, vl2) = topexp::vertices(&e2, false);
        if !vl1.is_same(&vf1) {
            if vf1.is_same(&vf2) && vl1.is_same(&vl2) {
                lsubs.push(e2.oriented(Orientation::Forward));
            } else if vf1.is_same(&vl2) && vl1.is_same(&vf2) {
                lsubs.push(e2.oriented(Orientation::Reversed));
            }
        } else if vf2.is_same(&vl2) && vl2.is_same(&vl1) {
            // discrimination sur les tangentes (tout au meme point)
            let d1 = |e: &Edge| -> Result<rustycad_tkmath::gp::Vector3, Failure> {
                let located = BRepTool::curve_with_location(e)?;
                let curve = located.curve.get().ok_or_else(|| {
                    null_object("LocOpe_Spliter::Perform", "edge without 3D curve")
                })?;
                Ok(curve
                    .d1(located.first)?
                    .1
                    .transformed(located.location.transformation()))
            };
            let orient = if d1(e1)?.dot(d1(&e2)?) > 0. {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            lsubs.push(e2.oriented(orient));
        }
    }
    Ok(lsubs)
}

impl Spliter {
    /// `LocOpe_Spliter()`; `Spliter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_Spliter(S)`: the algorithm on `s`; `Spliter::with_shape(&s)`.
    pub fn with_shape(s: &Shape) -> Self {
        Self {
            shape: s.clone(),
            ..Self::default()
        }
    }

    /// `Init(S)`; `spliter.init(&s)`.
    pub fn init(&mut self, s: &Shape) {
        self.shape = s.clone();
        self.done = false;
    }

    /// `IsDone()`; `spliter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `ResultingShape()`; StdFail_NotDone before Perform; `spliter.resulting_shape()?`.
    pub fn resulting_shape(&self) -> Result<&Shape, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Spliter::ResultingShape"));
        }
        Ok(&self.res)
    }

    /// `Shape()`: the initial shape; `spliter.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// `DirectLeft()`: the faces on the left of the wires; `spliter.direct_left()?`.
    pub fn direct_left(&self) -> Result<&List<Shape>, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Spliter::DirectLeft"));
        }
        Ok(&self.d_left)
    }

    /// `Left()`: the faces of the left part, grown from `DirectLeft` across the edges
    /// that are not wire edges; `spliter.left()?`.
    pub fn left(&self) -> Result<&List<Shape>, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Spliter::Left"));
        }
        Ok(&self.left)
    }

    /// `DescendantShapes(S)`: the descendants of `s` (none for an unknown shape);
    /// `spliter.descendant_shapes(&s)?`.
    pub fn descendant_shapes(&self, s: &Shape) -> Result<List<Shape>, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Spliter::DescendantShapes"));
        }
        Ok(self.map.seek(s).cloned().unwrap_or_default())
    }

    fn first_of(&self, s: &Shape) -> Result<Shape, Failure> {
        self.map
            .seek(s)
            .and_then(|l| l.first().cloned())
            .ok_or_else(|| null_object("LocOpe_Spliter::Perform", "shape without descendant"))
    }

    /// `Perform(PW)`: splits the shape by the wires of `pw`; Standard_NullObject without
    /// shape; `spliter.perform(&mut pw)?`.
    pub fn perform(&mut self, pw: &mut WiresOnShape) -> Result<(), Failure> {
        if self.shape.is_null() {
            return Err(null_object("LocOpe_Spliter::Perform", "no shape to split"));
        }
        self.done = false;
        self.map.clear(false);
        self.res.nullify();
        let shape = self.shape.clone();
        put(&shape, &mut self.map);
        let mut map_v = MapOfShape::default();
        let mut map_e = MapOfShape::default();
        self.substitute_vertices(pw, &mut map_v, &mut map_e)?;
        self.res = self.first_of(&shape)?;
        let mut the_c_face = SplitShape::with_shape(&self.res)?;
        let mut edg_on_edg = DataMap::default();
        let (mut map_fe, faces_with_section) =
            self.prepare_faces(pw, &mut map_v, &mut the_c_face, &mut edg_on_edg)?;
        // Rebuilds wires on each face of the shape
        for i in 1..=map_fe.extent() {
            let fac = topods::face(map_fe.find_key(i)?)?;
            let ledges = rebuild_wires(map_fe.find_from_index(i)?, pw)?;
            *map_fe.change_from_index(i)? = ledges.clone();
            if faces_with_section.contains(&fac) {
                the_c_face.add_wires(&ledges, &fac)?;
            } else {
                for w in ledges.iter() {
                    the_c_face.add_wire(&topods::wire(w)?, &fac)?;
                }
            }
        }
        // Mise a jour des descendants
        let keys: Vec<(Shape, Shape)> = self
            .map
            .items()
            .map(|(k, v)| (k.clone(), v.first().cloned().unwrap_or_default()))
            .collect();
        for (sori, scib) in keys {
            let desc = the_c_face.descendant_shapes(&scib)?.clone();
            if let Some(l) = self.map.change_seek(&sori) {
                *l = desc;
            }
        }
        let Some(res) = self.single_result()? else {
            return Ok(());
        };
        self.res = res;
        self.substitute_edges(&edg_on_edg)?;
        self.merge_degenerated_vertices()?;
        self.d_left.clear();
        self.left.clear();
        let others = self.direct_left_faces(&map_e)?;
        let map_v: MapOfShape = others.into_iter().collect_map();
        self.grow_left(&map_e, map_v)?;
        self.done = true;
        Ok(())
    }
}

/// Collects shapes into a TopTools map.
trait CollectMap {
    fn collect_map(self) -> MapOfShape;
}

impl<I: Iterator<Item = Shape>> CollectMap for I {
    fn collect_map(self) -> MapOfShape {
        let mut map = MapOfShape::default();
        for s in self {
            map.add(s);
        }
        map
    }
}
