//! `BuildDraftFace` and `HasMultiConnected` (BOPAlgo_Builder_2.cxx): a face rebuilt on the
//! splits of its boundary edges.
use super::{BOPAlgoBuilder, required_context};
use crate::bop_tools::algo_tools3d::split_seam_on_face;
use crate::bop_tools::is_split_to_reverse_with_warning;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The edges at each vertex met so far (`aVerticesCounter`).
type VertexEdges = DataMap<Shape, Vec<Shape>, ShapeMapHasher>;

/// The checks shared by the wires of one draft face.
struct DraftChecks {
    vertex_edges: VertexEdges,
    edges: Map<Shape, ShapeMapHasher>,
}

impl BOPAlgoBuilder {
    /// The face on the same surface with each wire rebuilt on the splits of its edges, or
    /// `None` when an internal edge, a multi-connected vertex or unified edges may split the
    /// face (`BuildDraftFace`).
    ///
    /// Native quirk kept: the splits are oriented in place inside the image lists.
    pub(super) fn build_draft_face(&mut self, face: &Face) -> Result<Option<Face>, Failure> {
        let builder = BRepBuilder::default();
        let (surface, location) = BRepTool::surface_with_location(face)?;
        let mut draft = Face::default();
        builder.make_face_on_located_surface(
            &mut draft,
            &surface,
            &location,
            BRepTool::face_tolerance(face)?,
        )?;
        let mut checks = DraftChecks {
            vertex_edges: DataMap::with_hasher(ShapeMapHasher, 1),
            edges: Map::with_hasher(ShapeMapHasher, 1),
        };
        let mut draft = draft.into_shape();
        for wire in ShapeIterator::new(&face.oriented(Orientation::Forward), true, true) {
            if wire.shape_type()? != ShapeEnum::Wire {
                continue;
            }
            let Some(new_wire) = self.draft_wire(&wire, face, &mut checks)? else {
                return Ok(None);
            };
            if let Some(new_wire) = new_wire {
                builder.add(&mut draft, &new_wire)?;
            }
        }
        if face.orientation() == Orientation::Reversed {
            draft.reverse();
        }
        Ok(Some(Face::unchecked(draft)))
    }

    /// The wire rebuilt on the splits (`Some(None)` for an empty wire), or `None` when the
    /// face must go to the face builder.
    fn draft_wire(
        &mut self,
        wire: &Shape,
        face: &Face,
        checks: &mut DraftChecks,
    ) -> Result<Option<Option<Shape>>, Failure> {
        let edges: Vec<Shape> =
            ShapeIterator::new(&wire.oriented(Orientation::Forward), true, true).collect();
        if edges.is_empty() {
            return Ok(Some(None));
        }
        let builder = BRepBuilder::default();
        let mut new_wire = Wire::default();
        builder.make_wire(&mut new_wire);
        let mut new_wire = new_wire.into_shape();
        for edge in edges {
            if !self.draft_edge(&Edge::unchecked(edge), face, checks, &mut new_wire)? {
                return Ok(None);
            }
        }
        new_wire.set_orientation(wire.orientation());
        new_wire.set_closed(BRepTool::is_closed(&new_wire)?)?;
        Ok(Some(Some(new_wire)))
    }

    /// Adds the edge or its splits to the new wire; false when the face must go to the face
    /// builder.
    fn draft_edge(
        &mut self,
        edge: &Edge,
        face: &Face,
        checks: &mut DraftChecks,
        new_wire: &mut Shape,
    ) -> Result<bool, Failure> {
        let orientation = edge.orientation();
        if orientation == Orientation::Internal {
            // Internal edges could split the face on halves.
            return Ok(false);
        }
        let degenerated = BRepTool::degenerated(edge)?;
        let closed = BRepTool::is_closed_on_face(edge, face)?;
        let builder = BRepBuilder::default();
        let Some(count) = self.images.seek(edge).map(|images| images.len()) else {
            if !accept_edge(edge, degenerated, closed, checks)? {
                return Ok(false);
            }
            builder.add(new_wire, edge)?;
            return Ok(true);
        };
        for position in 0..count {
            // The checks see the split as stored, before it is oriented in place.
            let split = Edge::unchecked(self.image_at(edge, position)?);
            if !accept_edge(&split, degenerated, closed, checks)? {
                return Ok(false);
            }
            self.image_slot(edge, position)?
                .set_orientation(orientation);
            if !degenerated {
                self.orient_split_in_place(edge, position, face, closed)?;
            }
            let split = self.image_at(edge, position)?;
            builder.add(new_wire, &split)?;
        }
        Ok(true)
    }

    /// Gives a split of a seam its second p-curve and reverses it in place when it goes
    /// against the original edge.
    fn orient_split_in_place(
        &mut self,
        edge: &Edge,
        position: usize,
        face: &Face,
        closed: bool,
    ) -> Result<(), Failure> {
        let split = Edge::unchecked(self.image_at(edge, position)?);
        if closed && !BRepTool::is_closed_on_face(&split, face)? {
            split_seam_on_face(&split, face)?;
        }
        let context = required_context(&self.context)?;
        if is_split_to_reverse_with_warning(&split, edge, context, Some(self.options().report()))? {
            self.image_slot(edge, position)?.reverse();
        }
        Ok(())
    }

    fn image_at(&self, edge: &Edge, position: usize) -> Result<Shape, Failure> {
        self.images
            .find(edge)?
            .iter()
            .nth(position)
            .cloned()
            .ok_or_else(|| missing_split(position))
    }

    fn image_slot(&mut self, edge: &Edge, position: usize) -> Result<&mut Shape, Failure> {
        self.images
            .change_find(edge)?
            .iter_mut()
            .nth(position)
            .ok_or_else(|| missing_split(position))
    }
}

/// The multi-connexity and unification checks of one edge of the draft face.
fn accept_edge(
    edge: &Edge,
    degenerated: bool,
    closed: bool,
    checks: &mut DraftChecks,
) -> Result<bool, Failure> {
    if !degenerated && has_multi_connected(edge, &mut checks.vertex_edges) {
        return Ok(false);
    }
    // The edges of the face must not have been unified during the intersection.
    Ok(closed || checks.edges.add(edge.clone().into_shape()))
}

/// Whether a vertex of the edge now has more than two edges (`HasMultiConnected`).
fn has_multi_connected(edge: &Edge, vertex_edges: &mut VertexEdges) -> bool {
    for vertex in ShapeIterator::new(edge, true, true) {
        let Some(edges) = vertex_edges.change_seek(&vertex) else {
            vertex_edges.bound(vertex, vec![edge.clone().into_shape()]);
            continue;
        };
        if !edges.iter().any(|known| known == &**edge) {
            edges.push(edge.clone().into_shape());
        }
        if edges.len() > 2 {
            return true;
        }
    }
    false
}

fn missing_split(position: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BuildDraftFace: no split at position {position}; expected an image index"),
        "",
    )
}
