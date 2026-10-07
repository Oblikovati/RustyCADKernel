// Rust translation of the join branch of ShapeFix_Wireframe::MergeSmallEdges
// (ShapeFix_Wireframe.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871).

//! The join of the chosen pair: the seam-crossing protection on the other faces, the
//! joined edge, its vertex copies and non-manifold vertices, and the map updates.
use super::super::join_edges::join_edges;
use super::choose::{Choice, Positions};
use super::{MergeRun, WireScope};
use crate::shape_analysis::{
    ShapeAnalysisEdge, ShapeAnalysisTransferParametersProj, ShapeAnalysisWire,
};
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_fix::ShapeFixWire;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{self, Builder, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl MergeRun<'_, '_> {
    /// `same_set && !isLimAngle`: the pair is joined (FAIL1 and the next index when it
    /// cannot be).
    pub(super) fn join_pair(
        &mut self,
        at: Positions,
        choice: Choice,
        scope: &WireScope,
    ) -> Result<i32, Failure> {
        let mut joined = Edge::default();
        let mut replace_first = true;
        if self.need_join(&choice, scope)? {
            let mut faces = Vec::new();
            for face in choice.faces.iter() {
                faces.push(ReShape::apply(&*self.context, face, ShapeEnum::Shape)?);
            }
            let result = join_edges(&choice.edge1, &choice.edge2, &faces);
            (joined, replace_first) = (result.edge, result.replace_first);
        }
        if joined.is_null() {
            self.add_status(ShapeExtendStatus::Fail1);
            return Ok(at.index + 1);
        }
        self.record_new_vertices(&joined)?;
        self.keep_nm_vertices(&choice.edge1, &mut joined)?;
        self.keep_nm_vertices(&choice.edge2, &mut joined)?;
        self.install_joined(at, &choice, &joined, replace_first, scope)?;
        self.add_status(ShapeExtendStatus::Done1);
        Ok(at.index)
    }

    /// `isNeedJoin` (gka protection against crossing seem on second face): on the first
    /// other face wire holding both edges, they must follow each other in reverse order.
    fn need_join(&self, choice: &Choice, scope: &WireScope) -> Result<bool, Failure> {
        for face in choice.faces.iter() {
            if face.is_same(&scope.current) {
                continue;
            }
            let applied = ReShape::apply(&*self.context, face, ShapeEnum::Shape)?;
            if wire_order(&applied, choice)? == Some(false) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `theNewVertices`: the vertices of the joined edge get empty copies.
    fn record_new_vertices(&mut self, joined: &Edge) -> Result<(), Failure> {
        let sae = ShapeAnalysisEdge::new();
        let (old1, old2) = (sae.first_vertex(joined), sae.last_vertex(joined));
        self.bind_copy(&old1)?;
        if !old1.is_same(&old2) {
            self.bind_copy(&old2)?;
        }
        Ok(())
    }

    fn bind_copy(&mut self, vertex: &Vertex) -> Result<(), Failure> {
        if !self.new_vertices.is_bound(vertex) {
            let copy = topods::vertex(&vertex.empty_copied()?)?.into_shape();
            self.new_vertices.bind(vertex.clone().into_shape(), copy);
        }
        Ok(())
    }

    /// To keep NM vertices belonging initial edges.
    fn keep_nm_vertices(&self, source: &Edge, joined: &mut Edge) -> Result<(), Failure> {
        for child in ShapeIterator::new(source, false, true) {
            if !matches!(
                child.orientation(),
                Orientation::Internal | Orientation::External
            ) {
                continue;
            }
            let old = topods::vertex(&child)?;
            let copy =
                ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(&old, joined, source)?;
            let mut shape = joined.clone().into_shape();
            Builder.add(&mut shape, &copy)?;
            *joined = topods::edge(&shape)?;
            ReShape::replace(&*self.context, &old, &copy)?;
        }
        Ok(())
    }

    /// The context and wire changes of a join, then the map updates.
    fn install_joined(
        &mut self,
        at: Positions,
        choice: &Choice,
        joined: &Edge,
        replace_first: bool,
        scope: &WireScope,
    ) -> Result<(), Failure> {
        let data = self.wire_data()?;
        let analysis = ShapeAnalysisWire::new();
        match &scope.face {
            Some(face) => analysis.init_wire_data(&data, face, self.precision())?,
            None => {
                analysis.load_wire_data(&data);
                analysis.set_precision(self.precision());
            }
        }
        let (kept, removed) = if replace_first {
            (&choice.edge1, &choice.edge2)
        } else {
            (&choice.edge2, &choice.edge1)
        };
        ReShape::replace(&*self.context, kept, joined)?;
        ReShape::remove(&*self.context, removed)?;
        self.warn(removed, "FixWireframe.FixSmallEdges.MSG0");
        let neighbour = if choice.take_next { at.next } else { at.prev };
        data.set(joined, neighbour)?;
        let new_small = analysis.check_small_at(neighbour, self.precision())?;
        data.remove(at.index)?;
        self.update_joined_maps(choice, joined, new_small)
    }

    /// Process changes in maps.
    fn update_joined_maps(
        &mut self,
        choice: &Choice,
        joined: &Edge,
        new_small: bool,
    ) -> Result<(), Failure> {
        let faces = choice.faces.clone();
        self.maps.edge_faces.unbind(&choice.edge1);
        self.maps.edge_faces.unbind(&choice.edge2);
        self.maps
            .edge_faces
            .bind(joined.clone().into_shape(), faces.clone());
        self.maps.small.remove(&choice.edge1);
        self.maps.small.remove(&choice.edge2);
        if new_small {
            self.maps.small.add(joined.clone().into_shape());
        }
        for face in faces.iter() {
            let Ok(edges) = self.maps.face_small.change_find(face) else {
                continue;
            };
            if new_small {
                edges.append(joined.clone().into_shape());
            }
            edges.retain(|edge| !edge.is_same(&choice.edge1) && !edge.is_same(&choice.edge2));
            if edges.extent() == 0 {
                self.maps.face_small.unbind(face);
            }
        }
        Ok(())
    }
}

/// On the first wire of the face holding both edges (after FixReorder): whether the
/// second edge comes just before the first one; `None` when no wire holds both.
fn wire_order(face: &Shape, choice: &Choice) -> Result<Option<bool>, Failure> {
    for child in ShapeIterator::new(face, true, true) {
        if child.shape_type()? != ShapeEnum::Wire {
            continue;
        }
        let fixer = ShapeFixWire::new();
        fixer.load(&topods::wire(&child)?)?;
        fixer.fix_reorder(false)?;
        let data = fixer.wire_data()?;
        let (ind1, ind2) = (data.index(&choice.edge1)?, data.index(&choice.edge2)?);
        if ind1 != 0 && ind2 != 0 {
            return Ok(Some(
                ind1 - ind2 == 1 || (ind2 == data.nb_edges() && ind1 == 1),
            ));
        }
    }
    Ok(None)
}
