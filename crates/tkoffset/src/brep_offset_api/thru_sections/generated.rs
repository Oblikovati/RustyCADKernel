// Rust translation of BRepOffsetAPI_ThruSections::Generated
// (ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_ThruSections.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The history of a loft: the faces grown from a section edge and the longitudinal
//! edges grown from a section vertex.
use super::BRepOffsetAPIThruSections;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;

impl BRepOffsetAPIThruSections {
    /// The list of `Generated(S)`: nothing for a shape that is neither an indexed
    /// section edge nor an indexed section vertex.
    pub(super) fn generated_shapes(&self, shape: &Shape) -> Result<List<Shape>, Failure> {
        let mut generated = List::default();
        let all_faces = faces_of(&self.base.my_shape)?;
        match shape.shape_type()? {
            ShapeEnum::Edge => self.faces_of_edge(shape, &all_faces, &mut generated)?,
            ShapeEnum::Vertex => self.edges_of_vertex(shape, &all_faces, &mut generated)?,
            _ => {}
        }
        Ok(generated)
    }

    /// The faces at the indices of a section edge, then (ruled) at the same indices
    /// in each next band of faces.
    fn faces_of_edge(
        &self,
        shape: &Shape,
        all_faces: &[Shape],
        generated: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let Ok(indices) = self.my_edge_new_indices.find(shape) else {
            return Ok(());
        };
        append_faces_at(all_faces, indices, 0, generated);
        if self.my_is_ruled {
            for band in 1..self.my_wires.len().saturating_sub(1) {
                let offset = band as i32 * self.my_nb_edges_in_section;
                append_faces_at(all_faces, indices, offset, generated);
            }
        }
        Ok(())
    }

    /// The longitudinal edges from a section vertex: the bunch of edges of a punctual
    /// end section, else the edge along the face of its index (and, ruled, the chain to
    /// the last section).
    fn edges_of_vertex(
        &self,
        shape: &Shape,
        all_faces: &[Shape],
        generated: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let Ok(&index) = self.my_vertex_index.find(shape) else {
            return Ok(());
        };
        let is_degen = self.degenerated_end_of(shape)?;
        if is_degen[0] || is_degen[1] {
            return self.edges_of_punctual_end(shape, is_degen, generated);
        }
        self.edges_of_indexed_vertex(index, all_faces, generated)
    }

    /// Which punctual end section (first, then last) has the vertex.
    fn degenerated_end_of(&self, shape: &Shape) -> Result<[bool; 2], Failure> {
        let mut is_degen = [false, false];
        let ends = [
            (self.my_degen1, 0),
            (self.my_degen2, self.my_wires.len().saturating_sub(1)),
        ];
        for (i, (degenerated, section)) in ends.into_iter().enumerate() {
            if !degenerated {
                continue;
            }
            let explorer =
                Explorer::new(&self.my_wires[section], ShapeEnum::Vertex, ShapeEnum::Shape)?;
            if shape.is_same(explorer.current()) {
                is_degen[i] = true;
                break;
            }
        }
        Ok(is_degen)
    }

    /// The vertex as built by the generator (ruled or immutable input), or itself.
    fn result_of(&self, shape: &Shape) -> Result<Shape, Failure> {
        match &self.my_bf_generator {
            Some(generator) if self.my_is_ruled || !self.my_mutable_input => {
                generator.result_shape(shape)
            }
            _ => Ok(shape.clone()),
        }
    }

    /// The edges of the result leaving (first end) or reaching (last end) the punctual
    /// section vertex, and (ruled) the chain of each through the inner sections.
    fn edges_of_punctual_end(
        &self,
        shape: &Shape,
        is_degen: [bool; 2],
        generated: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut ve_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.base.my_shape,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut ve_map,
        )?;
        let new_shape = self.result_of(shape)?;
        let mut emap = IndexedMapOfShape::default();
        for edge in ve_map.find_from_key(&new_shape)?.iter() {
            let edge = topods::edge(edge)?;
            if BRepTool::degenerated(&edge)? {
                continue;
            }
            let (v0, v1) = topexp::vertices(&edge, false);
            if (is_degen[0] && new_shape.is_same(&v0)) || (is_degen[1] && new_shape.is_same(&v1)) {
                emap.add(edge.into_shape());
            }
        }
        for j in 1..=emap.extent() {
            let edge = topods::edge(emap.find_key(j)?)?;
            generated.append(edge.as_ref().clone());
            if self.my_is_ruled {
                self.append_chain_from_end(edge, is_degen[0], &ve_map, generated)?;
            }
        }
        Ok(())
    }

    /// The chain of longitudinal edges following `edge` through the inner sections
    /// (from the first section when `from_first`, else back from the last).
    fn append_chain_from_end(
        &self,
        mut edge: Edge,
        from_first: bool,
        ve_map: &IndexedDataMapOfShapeListOfShape,
        generated: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let count = self.my_wires.len();
        for (i, k) in (2..count).zip((1..count).rev()) {
            let section = if from_first { i } else { k };
            let vertex = if from_first {
                topexp::last_vertex(&edge, false)
            } else {
                topexp::first_vertex(&edge, false)
            };
            let mut section_edges = IndexedMapOfShape::default();
            let section_wire = self.result_of(&self.my_wires[section - 1])?;
            topexp::map_shapes_of_type(&section_wire, ShapeEnum::Edge, &mut section_edges)?;
            let next_edge = next_longitudinal_edge(
                ve_map.find_from_key(vertex.as_ref())?,
                &edge,
                &section_edges,
            )?;
            generated.append(next_edge.as_ref().clone());
            edge = next_edge;
        }
        Ok(())
    }

    /// The longitudinal edge of a vertex of a non-punctual section: an edge of the
    /// face at its index, then (ruled) its continuation through each band of faces.
    fn edges_of_indexed_vertex(
        &self,
        index: i32,
        all_faces: &[Shape],
        generated: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let vindex = if index > 0 { 0 } else { 1 };
        let mut eindex = index.abs();
        let first_face = forward_face(all_faces, eindex)?;
        let mut ve_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            first_face.as_ref(),
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut ve_map,
        )?;
        let mut edge = self.first_longitudinal_edge(&first_face, eindex, vindex, &ve_map)?;
        generated.append(edge.as_ref().clone());
        if !self.my_is_ruled {
            return Ok(());
        }
        for _ in 2..self.my_wires.len() {
            let first_vertex = topexp::last_vertex(&edge, false);
            let first_edge = other_edge(ve_map.find_from_key(first_vertex.as_ref())?, &edge)?;
            eindex += self.my_nb_edges_in_section;
            let face = forward_face(all_faces, eindex)?;
            ve_map.clear(true);
            topexp::map_shapes_and_ancestors(
                face.as_ref(),
                ShapeEnum::Vertex,
                ShapeEnum::Edge,
                &mut ve_map,
            )?;
            edge = other_edge(ve_map.find_from_key(first_vertex.as_ref())?, &first_edge)?;
            generated.append(edge.as_ref().clone());
        }
        Ok(())
    }

    /// The longitudinal edge of the first face: its first or third edge on a plane of
    /// a punctual first section, else the edge at the section vertex of the right sense.
    fn first_longitudinal_edge(
        &self,
        first_face: &Face,
        eindex: i32,
        vindex: usize,
        ve_map: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Edge, Failure> {
        let mut explorer = Explorer::new(first_face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
        let surface = BRepAdaptorSurface::with_face(first_face, false)?;
        if self.my_degen1 && surface.get_type()? == SurfaceType::Plane {
            if vindex == 0 {
                explorer.next()?;
                explorer.next()?;
            }
            return topods::edge(explorer.current());
        }
        let (first_edge, section_vertex) = self.section_edge_at(eindex)?;
        let (v0, v1) = topexp::vertices(&first_edge, false);
        let starts_at_first = v0.is_same(&section_vertex);
        let first_vertex: Vertex = if (vindex == 0) == starts_at_first {
            v0
        } else {
            v1
        };
        let orientation = if vindex == 0 {
            Orientation::Reversed
        } else {
            Orientation::Forward
        };
        let mut found = Edge::default();
        for edge in ve_map.find_from_key(first_vertex.as_ref())?.iter() {
            found = topods::edge(edge)?;
            if !found.is_same(explorer.current())
                && !BRepTool::degenerated(&found)?
                && found.orientation() == orientation
            {
                break;
            }
        }
        Ok(found)
    }

    /// The edge at the 1-based index in the first section (as built by the generator
    /// when it applies) and the wire explorer vertex there (null edges past the end).
    fn section_edge_at(&self, eindex: i32) -> Result<(Edge, Vertex), Failure> {
        let mut explorer = WireExplorer::with_wire(&topods::wire(&self.my_wires[0])?)?;
        let mut first_edge = Edge::default();
        let mut i = 1;
        while explorer.more() {
            first_edge = explorer.current().clone();
            if i == eindex {
                let edge = topods::edge(&self.result_of(first_edge.as_ref())?)?;
                return Ok((edge, explorer.current_vertex().clone()));
            }
            explorer.next()?;
            i += 1;
        }
        Ok((first_edge, Vertex::default()))
    }
}

/// The faces of the shape in explorer order.
fn faces_of(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut faces = Vec::new();
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while explorer.more() {
        faces.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(faces)
}

/// Appends the faces at the 1-based indices shifted by `offset`, skipping indices
/// past the last face.
fn append_faces_at(all_faces: &[Shape], indices: &[i32], offset: i32, generated: &mut List<Shape>) {
    for index in indices {
        let position = index + offset;
        if position < 1 || all_faces.len() < position as usize {
            continue;
        }
        generated.append(all_faces[position as usize - 1].clone());
    }
}

/// The face at the 1-based index, FORWARD; Standard_OutOfRange past the faces.
fn forward_face(all_faces: &[Shape], index: i32) -> Result<Face, Failure> {
    let face = usize::try_from(index - 1)
        .ok()
        .and_then(|i| all_faces.get(i))
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "ThruSections Generated: face {index}; expected 1..={}",
                    all_faces.len()
                ),
            )
        })?;
    topods::face(&face.oriented(Orientation::Forward))
}

/// The edge of a two-edge list that is not `edge` (the first one otherwise).
fn other_edge(edges: &List<Shape>, edge: &Shape) -> Result<Edge, Failure> {
    let (first, last) = (edges.first(), edges.last());
    let chosen = match first {
        Some(first) if edge.is_same(first) => last,
        _ => first,
    };
    topods::edge(chosen.ok_or_else(|| {
        Failure::with_message(
            "Standard_NoSuchObject",
            "ThruSections Generated: a vertex without edge; expected its edges",
        )
    })?)
}

/// The last edge of the list, or the first that is neither `edge` nor in the section.
fn next_longitudinal_edge(
    edges: &List<Shape>,
    edge: &Edge,
    section_edges: &IndexedMapOfShape,
) -> Result<Edge, Failure> {
    let mut next = Edge::default();
    for candidate in edges.iter() {
        next = topods::edge(candidate)?;
        if !next.is_same(edge) && !section_edges.contains(&next) {
            break;
        }
    }
    Ok(next)
}
