// Rust translation of BRepBuilderAPI_Sewing::FindFreeBoundaries (BRepBuilderAPI_Sewing.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The free boundaries: the edges of one face (or of any number of faces in
//! non-manifold mode, or of none for floating edges) with their faces, and the nodes of
//! their vertices.
use super::{BRepBuilderAPISewing, live_reshape};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The edges of the shapes to sew, each with the faces whose wires use it.
fn edge_faces(
    new_shapes: &IndexedMapOfShape,
    edges: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    let mut map_faces = MapOfShape::default();
    for i in 1..=new_shapes.extent() {
        for face in Explorer::new(new_shapes.find_key(i)?, ShapeEnum::Face, ShapeEnum::Shape)? {
            if !map_faces.add(face.clone()) {
                continue;
            }
            let mut wires = ShapeIterator::new(&face, true, true);
            while let Ok(wire) = wires.value() {
                if wire.shape_type()? == ShapeEnum::Wire {
                    let mut it = ShapeIterator::new(wire, true, true);
                    while let Ok(edge) = it.value() {
                        if let Some(faces) = edges.change_seek(edge) {
                            faces.append(face.clone());
                        }
                        it.next();
                    }
                }
                wires.next();
            }
        }
    }
    Ok(())
}

/// The copy of a seam met by one face only: the same vertices and ranges, the seam
/// pair on the face replaced by its first p-curve.
fn unsewn_seam(edge: &Shape, face: &Shape) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut new_edge = edge.empty_copied()?;
    let mut it = ShapeIterator::new(edge, true, true);
    while let Ok(vertex) = it.value() {
        b.add(&mut new_edge, vertex)?;
        it.next();
    }
    let (e, f) = (topods::edge(edge)?, topods::face(face)?);
    let old = BRepTool::curve_on_face(&e, &f)?;
    let typed = topods::edge(&new_edge)?;
    let null: Handle<dyn Geom2dCurve> = Handle::default();
    b.update_edge_pcurves_on_face(&typed, (&null, &null), &f, 0.0)?;
    b.update_edge_pcurve_on_face(&typed, &old.pcurve, &f, 0.0)?;
    let (first, last) = BRepTool::range(&e)?;
    b.range(&typed, first, last, false)?;
    b.range_on_face(&typed, &f, old.first, old.last)?;
    Ok(new_edge)
}

impl BRepBuilderAPISewing {
    /// The shapes to sew: the image of the context shape (it becomes the input when
    /// none was added), then the inputs, whose edges are collected.
    fn shapes_to_sew(
        &mut self,
        edges: &mut IndexedDataMapOfShapeListOfShape,
    ) -> Result<IndexedMapOfShape, Failure> {
        let mut new_shapes = IndexedMapOfShape::default();
        if !self.my_shape.is_null() {
            if self.my_old_shapes.is_empty() {
                let shape = self.my_shape.clone();
                self.add(&shape)?;
                self.my_shape.nullify();
            } else {
                let new_shape =
                    live_reshape(&self.my_re_shape)?.apply(&self.my_shape, ShapeEnum::Shape)?;
                if !new_shape.is_null() {
                    new_shapes.add(new_shape);
                }
            }
        }
        for i in 1..=self.my_old_shapes.extent() {
            let shape = self.my_old_shapes.find_from_index(i)?;
            if shape.is_null() {
                continue;
            }
            new_shapes.add(shape.clone());
            for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !edges.contains(&edge) {
                    edges.add(edge, List::default());
                }
            }
        }
        Ok(new_shapes)
    }

    /// `FindFreeBoundaries()`: fills `myBoundFaces` (boundary -> faces) with the edges
    /// of one face (a seam of one face is first replaced by a plain copy), of any
    /// number of faces in non-manifold mode, or of no face with the floating edges
    /// mode; degenerated and INTERNAL edges are left out. The vertices of each boundary
    /// become their own nodes (`myVertexNode`, `myVertexNodeFree` for floating edges).
    pub(crate) fn find_free_boundaries(&mut self) -> Result<(), Failure> {
        let mut edges = IndexedDataMapOfShapeListOfShape::default();
        let new_shapes = self.shapes_to_sew(&mut edges)?;
        edge_faces(&new_shapes, &mut edges)?;
        let reshape = self.my_re_shape.clone();
        for i in 1..=edges.extent() {
            let list_faces = edges.find_from_index(i)?;
            let nb_faces = list_faces.extent();
            let mut edge = edges.find_key(i)?.clone();
            if edge.orientation() == Orientation::Internal {
                continue;
            }
            if nb_faces == 1 {
                let face = list_faces.first().cloned().unwrap_or_default();
                if BRepTool::is_closed_on_face(&topods::edge(&edge)?, &topods::face(&face)?)? {
                    let new_edge = unsewn_seam(&edge, &face)?;
                    live_reshape(&reshape)?.replace(&edge, &new_edge)?;
                    edge = new_edge;
                }
            }
            let is_bound_float = self.floating_edges_mode() && nb_faces == 0;
            let is_bound =
                self.face_mode() && ((self.my_nonmanifold && nb_faces != 0) || nb_faces == 1);
            if is_bound || is_bound_float {
                self.add_boundary(&edge, list_faces.clone(), is_bound)?;
            }
        }
        Ok(())
    }

    /// A boundary and its faces, and its vertices as nodes (unless one is missing or
    /// INTERNAL); a degenerated edge is left out.
    fn add_boundary(
        &mut self,
        edge: &Shape,
        faces: List<Shape>,
        is_bound: bool,
    ) -> Result<(), Failure> {
        let typed: Edge = topods::edge(edge)?;
        if BRepTool::degenerated(&typed)? {
            return Ok(());
        }
        self.my_bound_faces.add(edge.clone(), faces);
        let (v_first, v_last) = topexp::vertices(&typed, false);
        if v_first.is_null() || v_last.is_null() {
            return Ok(());
        }
        if v_first.orientation() == Orientation::Internal
            || v_last.orientation() == Orientation::Internal
        {
            return Ok(());
        }
        let nodes = if is_bound {
            &mut self.my_vertex_node
        } else {
            &mut self.my_vertex_node_free
        };
        for v in [v_first.into_shape(), v_last.into_shape()] {
            if !nodes.contains(&v) {
                nodes.add(v.clone(), v);
            }
        }
        Ok(())
    }
}
