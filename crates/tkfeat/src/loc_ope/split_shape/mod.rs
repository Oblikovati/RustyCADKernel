// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_SplitShape.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_SplitShape`: edges cut at vertices and faces cut by wires, the containing
//! shape rebuilt on the pieces.
mod closed_wire;
mod open_wire;
mod statics;
mod wire_list;

use crate::errors::no_such_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisEdge;

/// The shapes and their descendants (`myMap`).
pub(crate) type DescendantMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `LocOpe_SplitShape`: the cut of the edges and faces of a shape.
///
/// ```
/// use rustycad_tkfeat::loc_ope::SplitShape;
/// use rustycad_tkbrep::topods::Shape;
/// let split = SplitShape::new();
/// assert!(split.shape().is_null());
/// ```
#[derive(Clone, Default)]
pub struct SplitShape {
    done: bool,
    shape: Shape,
    map: DescendantMap,
    dbl_e: MapOfShape,
    left: List<Shape>,
}

/// The shapes of a list, in order.
pub(crate) fn to_vec(l: &List<Shape>) -> Vec<Shape> {
    l.iter().cloned().collect()
}

/// A list of shapes.
pub(crate) fn to_list(v: Vec<Shape>) -> List<Shape> {
    v.into_iter().collect()
}

/// `updateToleraces`: raises the tolerance of the pieces of the split edges of the face
/// to their same-parameter deviation on it.
fn update_tolerances(the_face: &Face, the_map: &DescendantMap) -> Result<(), Failure> {
    for e in Explorer::new(the_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let Some(l_edges) = the_map.seek(&e) else {
            continue;
        };
        if l_edges.len() <= 1 {
            continue;
        }
        let mut sae = ShapeAnalysisEdge::default();
        for cur in l_edges.iter() {
            let cur_e = topods::edge(cur)?;
            let (done, maxdev) = sae.check_same_parameter_on_face(&cur_e, the_face, 23)?;
            if done {
                BRepBuilder::default().update_edge_tolerance(&cur_e, maxdev)?;
            }
        }
    }
    Ok(())
}

/// An edge copy of `edg` (FORWARD) bounded by the vertices `v1` (FORWARD) and `v2`
/// (REVERSED) at the parameters `p1` and `p2`, with the tolerances `t1` and `t2`.
fn bounded_copy(
    edg: &Edge,
    (v1, p1, t1): (&Vertex, f64, f64),
    (v2, p2, t2): (&Vertex, f64, f64),
) -> Result<Edge, Failure> {
    let b = BRepBuilder::default();
    let mut e = topods::edge(&edg.empty_copied()?)?;
    e.set_orientation(Orientation::Forward);
    let v1 = topods::vertex(&v1.oriented(Orientation::Forward))?;
    b.add(&mut e, &v1)?;
    b.update_vertex_on_edge(&v1, p1, &e, t1)?;
    let v2 = topods::vertex(&v2.oriented(Orientation::Reversed))?;
    b.add(&mut e, &v2)?;
    b.update_vertex_on_edge(&v2, p2, &e, t2)?;
    Ok(e)
}

impl SplitShape {
    /// `LocOpe_SplitShape()`; `SplitShape::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_SplitShape(S)`: initialized on `s`; `SplitShape::with_shape(&s)?`.
    pub fn with_shape(s: &Shape) -> Result<Self, Failure> {
        let mut split = Self::new();
        split.init(s)?;
        Ok(split)
    }

    /// `Init(S)`: every sub-shape of `s` bound without descendant (the vertices to
    /// themselves); `split.init(&s)?`.
    pub fn init(&mut self, s: &Shape) -> Result<(), Failure> {
        self.done = false;
        self.shape = s.clone();
        self.dbl_e.clear(false);
        self.map.clear(false);
        let shape = self.shape.clone();
        self.put(&shape)
    }

    /// `Put(S)`: binds `s` and its sub-shapes.
    fn put(&mut self, s: &Shape) -> Result<(), Failure> {
        if self.map.is_bound(s) {
            return Ok(());
        }
        self.map.bind(s.clone(), List::default());
        if s.shape_type()? != ShapeEnum::Vertex {
            for child in ShapeIterator::new(s, true, true) {
                self.put(&child)?;
            }
        } else if let Some(l) = self.map.change_seek(s) {
            l.append(s.clone());
        }
        Ok(())
    }

    /// The descendants of `s`, which must be bound (Standard_NoSuchObject otherwise, as
    /// the native operator() raises).
    pub(crate) fn list_of(&self, s: &Shape) -> Result<&List<Shape>, Failure> {
        self.map
            .seek(s)
            .ok_or_else(|| no_such_object("LocOpe_SplitShape", "shape not in the split shape"))
    }

    /// The descendants of `s`, to write.
    pub(crate) fn list_mut(&mut self, s: &Shape) -> Result<&mut List<Shape>, Failure> {
        self.map
            .change_seek(s)
            .ok_or_else(|| no_such_object("LocOpe_SplitShape", "shape not in the split shape"))
    }

    /// `CanSplit(E)`: not rebuilt yet, `e` is bound and no rebuilt wire holds it;
    /// `split.can_split(&e)?`.
    pub fn can_split(&self, e: &Edge) -> Result<bool, Failure> {
        if self.done || self.map.is_empty() || !self.map.is_bound(e) {
            return Ok(false);
        }
        // On verifie que l`edge n`appartient pas a un wire deja reconstruit
        for (key, value) in self.map.items() {
            if key.shape_type()? != ShapeEnum::Wire || value.is_empty() {
                continue;
            }
            let mut edges = Explorer::new(key, ShapeEnum::Edge, ShapeEnum::Shape)?;
            if edges.any(|x| x.is_same(e)) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Add(V, P, E)`: cuts the piece of `e` holding the parameter `p` strictly inside
    /// at the vertex `v` (FORWARD or REVERSED), or adds an INTERNAL or EXTERNAL `v` to a
    /// copy of the piece; `split.add_vertex(&v, p, &e)?`.
    pub fn add_vertex(&mut self, v: &Vertex, p: f64, e: &Edge) -> Result<(), Failure> {
        if !self.can_split(e)? {
            return Ok(());
        }
        let mut le = to_vec(self.list_of(e)?);
        if le.is_empty() {
            le.push(e.as_ref().clone());
        }
        let mut found = None;
        for (k, s) in le.iter().enumerate() {
            let (f, l) = BRepTool::range(&topods::edge(s)?)?;
            if p > f + precision::p_confusion() && p < l - precision::p_confusion() {
                found = Some(k);
                break;
            }
        }
        let Some(k) = found else {
            // The edge appended to an empty list stays there, as natively.
            *self.list_mut(e)? = to_list(le);
            return Ok(());
        };
        let edg = topods::edge(&le.remove(k))?;
        if v.orientation() == Orientation::Forward || v.orientation() == Orientation::Reversed {
            let (e1, e2) = Self::cut_at(&edg, v, p)?;
            let mut new_list: Vec<Shape> = le[..k].to_vec();
            new_list.push(e1.into_shape());
            new_list.push(e2.into_shape());
            new_list.extend_from_slice(&le[k..]);
            self.map.unbind(e);
            self.map.bind(e.as_ref().clone(), to_list(new_list));
        } else {
            le.push(Self::with_internal_vertex(&edg, v, p)?.into_shape());
            *self.list_mut(e)? = to_list(le);
        }
        Ok(())
    }

    /// The two pieces of `edg` at the vertex `v` of parameter `p`; on a degenerated edge
    /// the end vertices get the larger of their tolerance and the one of `v`.
    fn cut_at(edg: &Edge, v: &Vertex, p: f64) -> Result<(Edge, Edge), Failure> {
        let edg = topods::edge(&edg.oriented(Orientation::Forward))?;
        let (cur_v1, cur_v2) = topexp::vertices(&edg, false);
        let par1 = BRepTool::parameter(&cur_v1, &edg)?;
        let par2 = BRepTool::parameter(&cur_v2, &edg)?;
        let tol_split_v = BRepTool::vertex_tolerance(v)?;
        // for degenerated edges tolerance of vertices should be set to maximal value
        // from tolerance of the vertex of the edge and tolerance of splitting vertex
        let degenerated = BRepTool::degenerated(&edg)?;
        let tol_v1 = if degenerated {
            BRepTool::vertex_tolerance(&cur_v1)?.max(tol_split_v)
        } else {
            BRepTool::vertex_tolerance(&cur_v1)?
        };
        let e1 = bounded_copy(&edg, (&cur_v1, par1, tol_v1), (v, p, tol_split_v))?;
        let tol_v2 = if degenerated {
            tol_v1
        } else {
            BRepTool::vertex_tolerance(&cur_v2)?
        };
        let e2 = bounded_copy(&edg, (v, p, tol_split_v), (&cur_v2, par2, tol_v2))?;
        Ok((e1, e2))
    }

    /// A copy of `edg` with its vertices and the INTERNAL or EXTERNAL vertex `v` at `p`.
    fn with_internal_vertex(edg: &Edge, v: &Vertex, p: f64) -> Result<Edge, Failure> {
        let b = BRepBuilder::default();
        let mut e1 = topods::edge(&edg.empty_copied()?)?;
        for x in Explorer::new(edg, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            let vtx = topods::vertex(&x)?;
            let f = BRepTool::parameter(&vtx, edg)?;
            b.add(&mut e1, &vtx)?;
            b.update_vertex_on_edge(&vtx, f, &e1, BRepTool::vertex_tolerance(&vtx)?)?;
        }
        b.add(&mut e1, v)?;
        b.update_vertex_on_edge(v, p, &e1, BRepTool::vertex_tolerance(v)?)?;
        Ok(e1)
    }

    /// Codes the CN regularity of the edges of `w` on `f` (JAG 10.11.95 Codage des
    /// regularites).
    fn code_regularities(w: &Shape, f: &Face) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edg = topods::edge(&e)?;
            if !BRepTool::has_continuity_on_faces(&edg, f, f)? {
                b.continuity_on_faces(&edg, f, f, Continuity::CN)?;
            }
        }
        Ok(())
    }

    /// `Add(W, F)`: cuts the descendant of `f` holding the wire `w`, open or closed on
    /// `f`; false when it cannot (a failure inside the cut gives false, as the native
    /// catch does); `split.add_wire(&w, &f)?`.
    pub fn add_wire(&mut self, w: &Wire, f: &Face) -> Result<bool, Failure> {
        if self.done {
            return Ok(false);
        }
        if self.list_of(f)?.is_empty() {
            self.rebuild(f)?;
        }
        // The native catch of Standard_Failure gives false; the descendants changed
        // before the failure stay changed ("some faces may be lost").
        let added = match super::closed_wire(w, f) {
            Err(_) => return Ok(false),
            Ok(false) => self.add_open_wire(w, f),
            Ok(true) => self.add_closed_wire(w, f),
        };
        if !matches!(added, Ok(true)) {
            return Ok(false);
        }
        Self::code_regularities(w, f)?;
        Ok(true)
    }

    /// `Shape()`: the original shape; `split.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// `DescendantShapes(S)`: the descendants of `s`, the shape being rebuilt on the
    /// first call; `split.descendant_shapes(&s)?`.
    pub fn descendant_shapes(&mut self, s: &Shape) -> Result<&List<Shape>, Failure> {
        if !self.done {
            let shape = self.shape.clone();
            self.rebuild(&shape)?;
            self.done = true;
        }
        self.list_of(s)
    }

    /// `LeftOf(W, F)`: the descendants of `f` (with its orientation in the shape) holding
    /// an edge of `w` with the same orientation; Standard_NoSuchObject when `f` is not a
    /// face of the shape; `split.left_of(&w, &f)?`.
    pub fn left_of(&mut self, w: &Wire, f: &Face) -> Result<&List<Shape>, Failure> {
        if self.shape.is_null() {
            return Err(no_such_object("LocOpe_SplitShape::LeftOf", "no shape"));
        }
        let the_face = Explorer::new(&self.shape, ShapeEnum::Face, ShapeEnum::Shape)?
            .find(|x| x.is_same(f))
            .ok_or_else(|| {
                no_such_object("LocOpe_SplitShape::LeftOf", "the face is not in the shape")
            })?;
        self.left.clear();
        let or_face = the_face.orientation();
        let descendants = to_vec(self.list_of(&the_face)?);
        for edg in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
            for d in &descendants {
                let fac = d.oriented(or_face);
                let mut edges = Explorer::new(&fac, ShapeEnum::Edge, ShapeEnum::Shape)?;
                if !edges.any(|x| x.is_same(&edg) && x.orientation() == edg.orientation()) {
                    continue;
                }
                if !self.left.iter().any(|x| x.is_same(&fac)) {
                    // la face n`est pas deja presente
                    self.left.append(fac);
                }
                break;
            }
        }
        Ok(&self.left)
    }

    /// `Rebuild(S)`: the descendants of `s` (itself when no sub-shape was cut, a copy
    /// on the descendants of its sub-shapes otherwise); true when rebuilt.
    fn rebuild(&mut self, s: &Shape) -> Result<bool, Failure> {
        if s.shape_type()? == ShapeEnum::Face {
            update_tolerances(&topods::face(s)?, &self.map)?;
        }
        if let Some(first) = self.list_of(s)?.first() {
            return Ok(!first.is_same(s));
        }
        let mut rebuild = false;
        for child in ShapeIterator::new(s, true, true) {
            rebuild = self.rebuild(&child)? || rebuild;
        }
        let result = if rebuild {
            self.rebuilt_copy(s)?
        } else {
            s.clone()
        };
        self.list_mut(s)?.append(result);
        Ok(rebuild)
    }

    /// The copy of `s` on the descendants of its sub-shapes, a wire or a shell closed
    /// as it is.
    fn rebuilt_copy(&self, s: &Shape) -> Result<Shape, Failure> {
        let b = BRepBuilder::default();
        let mut result = s.empty_copied()?;
        for child in ShapeIterator::new(s, true, true) {
            let orient = child.orientation();
            for d in self.list_of(&child)?.iter() {
                b.add(&mut result, &d.oriented(orient))?;
            }
        }
        // Assign "Closed" flag for Wires and Shells only
        if matches!(result.shape_type()?, ShapeEnum::Wire | ShapeEnum::Shell) {
            result.set_closed(BRepTool::is_closed(&result)?)?;
        }
        Ok(result)
    }
}
