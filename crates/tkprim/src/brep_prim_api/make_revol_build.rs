// Rust translation of BRepPrimAPI_MakeRevol::Build of BRepPrimAPI_MakeRevol.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The body of `BRepPrimAPI_MakeRevol::Build`: the per-face copies of the shared
//! degenerated edges.
use super::make_revol::BRepPrimAPIMakeRevol;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tktopalgo::brep_lib;

/// A `NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>,
/// TopTools_ShapeMapHasher>` of the body (a local list is a `Vec`).
type ShapeListMap = DataMap<Shape, Vec<Shape>, ShapeMapHasher>;

/// `anEdge.EmptyCopied()` holding the vertices of `edge` (taken forward, without
/// cumulating the orientation), with the orientation of `edge`.
fn copied_edge(edge: &Shape) -> Result<Shape, Failure> {
    let mut copy = edge.empty_copied()?;
    copy.set_orientation(Orientation::Forward);
    let builder = BRepBuilder::default();
    let mut vertices = ShapeIterator::new(&edge.oriented(Orientation::Forward), false, true);
    while vertices.more() {
        builder.add(&mut copy, vertices.value()?)?;
        vertices.next();
    }
    copy.set_orientation(edge.orientation());
    Ok(copy)
}

/// Whether the edge `shape` is degenerated (`BRep_TEdge::Degenerated`).
fn is_degenerated(shape: &Shape) -> Result<bool, Failure> {
    BRepTool::degenerated(&Edge::unchecked(shape.clone()))
}

/// For each face, the degenerated edges it holds, in explorer order.
fn degenerated_by_face(shape: &Shape) -> Result<ShapeListMap, Failure> {
    let mut by_face = ShapeListMap::default();
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let face = faces.current().clone();
        let mut edges = Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while edges.more() {
            if is_degenerated(edges.current())? {
                let list = match by_face.change_seek(&face) {
                    Some(list) => list,
                    None => by_face.bound(face.clone(), Vec::new()),
                };
                list.push(edges.current().clone());
            }
            edges.next()?;
        }
        faces.next()?;
    }
    Ok(by_face)
}

/// The substitution of the degenerated `edge` of a face by the first of its remaining
/// copies with the same orientation, unless the face holds the recorded occurrence
/// itself; the chosen entry leaves `copies`. Returns whether the edge is replaced.
fn substitute_edge(edge: &Shape, copies: &mut Vec<Shape>, substitution: &BRepToolsReShape) -> Result<bool, Failure> {
    for index in 0..copies.len() {
        if copies[index].is_equal(edge) {
            copies.remove(index);
            return Ok(false);
        }
        if copies[index].orientation() == edge.orientation() {
            substitution.replace(edge, &copies[index])?;
            copies.remove(index);
            return Ok(true);
        }
    }
    Ok(false)
}

impl BRepPrimAPIMakeRevol {
    /// The body of `Build` after its `myIsBuild` guard.
    pub(super) fn build_revolution(&mut self) -> Result<(), Failure> {
        self.base.my_shape = self.my_revol.shape()?;
        brep_lib::update_inner_tolerances(&self.base.my_shape)?;
        self.base.set_done();
        self.my_is_build = true;
        self.my_hist = Handle::default();
        self.my_degenerated.clear();
        let mut copies = self.record_degenerated()?;
        if self.my_degenerated.is_empty() {
            return Ok(());
        }
        self.substitute_degenerated(&mut copies)
    }

    /// Records every occurrence of a degenerated edge: the first one of each edge, then
    /// a copy for each further occurrence, all into `myDegenerated`; returns the
    /// occurrences by edge.
    fn record_degenerated(&mut self) -> Result<ShapeListMap, Failure> {
        let mut copies = ShapeListMap::default();
        let mut edges = Explorer::new(&self.base.my_shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while edges.more() {
            let edge = edges.current().clone();
            edges.next()?;
            if !is_degenerated(&edge)? {
                continue;
            }
            let recorded = match copies.change_seek(&edge) {
                Some(list) => {
                    let copy = copied_edge(&edge)?;
                    list.push(copy.clone());
                    copy
                }
                None => {
                    copies.bound(edge.clone(), vec![edge.clone()]);
                    edge
                }
            };
            self.my_degenerated.append(recorded);
        }
        Ok(copies)
    }

    /// Gives each face with degenerated edges its own copies, recording the
    /// substitutions in `myHist`.
    fn substitute_degenerated(&mut self, copies: &mut ShapeListMap) -> Result<(), Failure> {
        let by_face = degenerated_by_face(&self.base.my_shape)?;
        let substitution = BRepToolsReShape::new();
        let face_substitution = BRepToolsReShape::new();
        let mut cursor = by_face.cursor();
        while cursor.more() {
            substitution.clear();
            let mut is_replaced = false;
            let face = cursor.key()?;
            for edge in cursor.value()? {
                let Some(edge_copies) = copies.change_seek(edge) else {
                    continue;
                };
                // The flag stays set once an edge of the face is replaced.
                if substitute_edge(edge, edge_copies, &substitution)? {
                    is_replaced = true;
                }
            }
            if is_replaced {
                self.replace_face(face, &substitution, &face_substitution)?;
            }
            cursor.next();
        }
        Ok(())
    }

    /// Replaces `face` by its substituted copy in the shape and merges both
    /// substitutions into `myHist`.
    fn replace_face(&mut self, face: &Shape, substitution: &BRepToolsReShape, face_substitution: &BRepToolsReShape) -> Result<(), Failure> {
        let new_face = substitution.apply(face, ShapeEnum::Shape)?;
        face_substitution.replace(face, &new_face)?;
        match self.my_hist.get() {
            None => self.my_hist = substitution.history()?,
            Some(history) => history.merge(&*substitution.history()?)?,
        }
        self.base.my_shape = face_substitution.apply(&self.base.my_shape, ShapeEnum::Shape)?;
        let history = self.my_hist.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "BRepPrimAPI_MakeRevol::Build: null history; expected the history of the substitution")
        })?;
        history.merge(&*face_substitution.history()?)?;
        history.replace_modified(face, &new_face)?;
        face_substitution.clear();
        Ok(())
    }
}
