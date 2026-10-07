// Rust translation of the drop branches of ShapeFix_Wireframe::MergeSmallEdges
// (ShapeFix_Wireframe.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871).

//! A small edge that cannot be joined: in drop mode it is removed when its neighbours
//! stay connected; otherwise only a closed edge alone on its face is removed (with the
//! `gka` protection against removing circles).
use super::MergeRun;
use super::choose::Choice;
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

impl MergeRun<'_, '_> {
    /// `aModeDrop`: the edge `index` is removed from the wire when the wire stays
    /// connected without it; the neighbours reconnected and renamed in the maps (DONE3).
    pub(super) fn drop_middle(&mut self, index: i32, choice: &Choice) -> Result<i32, Failure> {
        let temp = Handle::new(ShapeExtendWireData::new());
        temp.add_wire(&self.wire_fixer.wire()?, 0)?;
        temp.remove(index)?;
        let analysis = ShapeAnalysisWire::new();
        analysis.load_wire_data(&temp);
        let count = analysis.nb_edges();
        let new_index = if index <= count { index } else { 1 };
        analysis.check_connected_at(new_index, self.precision())?;
        if analysis.last_check_status(ShapeExtendStatus::Fail) {
            return Ok(index + 1);
        }
        self.wire_data()?.remove(index)?;
        let tmp1 = temp.edge(new_index)?;
        let tmp2 = temp.edge(if new_index == 1 { count } else { new_index - 1 })?;
        let removed = if choice.take_next {
            &choice.edge1
        } else {
            &choice.edge2
        };
        self.reconnect_after_drop(new_index, removed, [tmp1, tmp2])?;
        self.add_status(ShapeExtendStatus::Done3);
        Ok(index)
    }

    /// FixConnected and FixDegenerated at the gap left by the dropped edge, the removal
    /// of the edge and the renaming of its neighbours `tmpedge1` and `tmpedge2`.
    fn reconnect_after_drop(
        &mut self,
        new_index: i32,
        removed: &Edge,
        neighbours: [Edge; 2],
    ) -> Result<(), Failure> {
        let faces = neighbours.clone().map(|edge| self.faces_of(&edge));
        let count = self.wire_data()?.nb_edges();
        let fix_index = if new_index <= count { new_index } else { 1 };
        self.wire_fixer
            .fix_connected_at(fix_index, self.precision(), true)?;
        self.wire_fixer.fix_degenerated_at(fix_index)?;
        let mut renamed = Vec::with_capacity(2);
        for edge in &neighbours {
            renamed.push(topods::edge(&ReShape::apply(
                &*self.context,
                edge,
                ShapeEnum::Shape,
            )?)?);
        }
        ReShape::remove(&*self.context, removed)?;
        self.warn(removed, "FixWireframe.FixSmallEdges.MSG0");
        self.maps.small.remove(removed);
        for edge in [removed, &neighbours[0], &neighbours[1]] {
            self.maps.edge_faces.unbind(edge);
        }
        for (new, faces) in renamed.iter().zip(&faces) {
            self.maps
                .edge_faces
                .bind(new.clone().into_shape(), faces.clone());
        }
        for ((old, new), faces) in neighbours.iter().zip(&renamed).zip(&faces) {
            self.rename_small(old, new, faces)?;
        }
        Ok(())
    }

    /// A small neighbour renamed by the reconnection is renamed in the small-edge map and
    /// in the small-edge lists of its faces (moved to the end, as the native in-loop
    /// Remove and Append).
    fn rename_small(&mut self, old: &Edge, new: &Edge, faces: &List<Shape>) -> Result<(), Failure> {
        if !self.maps.small.contains(old) {
            return Ok(());
        }
        self.maps.small.remove(old);
        self.maps.small.add(new.clone().into_shape());
        for face in faces.iter() {
            let edges = self.maps.face_small.change_find(face)?;
            let matches = edges.iter().filter(|edge| edge.is_same(old)).count();
            if matches > 0 && new.is_same(old) {
                // The native loop removes and appends the same edge forever.
                return Err(Failure::with_type(
                    "Standard_ProgramError",
                    &format!(
                        "ShapeFix_Wireframe::MergeSmallEdges: edge renamed to itself in the small-edge list of a face ({matches} occurrences); expected a new edge"
                    ),
                    "",
                ));
            }
            edges.retain(|edge| !edge.is_same(old));
            (0..matches).for_each(|_| {
                edges.append(new.clone().into_shape());
            });
        }
        Ok(())
    }

    /// Without drop mode (`gka protection against removing circles`): a closed edge that
    /// is the only one of its face is removed (DONE2), unless its middle point is farther
    /// from its start than its end.
    pub(super) fn remove_closed(&mut self, index: i32, choice: &Choice) -> Result<i32, Failure> {
        let candidate = if choice.take_next {
            &choice.edge1
        } else {
            &choice.edge2
        };
        if is_circle(candidate)? {
            return Ok(index + 1);
        }
        if choice.faces.extent() != 1 {
            return Ok(index + 1);
        }
        let sae = ShapeAnalysisEdge::new();
        if !sae
            .first_vertex(candidate)
            .is_same(&sae.last_vertex(candidate))
        {
            return Ok(index + 1);
        }
        self.wire_data()?.remove(index)?;
        ReShape::remove(&*self.context, candidate)?;
        self.warn(candidate, "FixWireframe.FixSmallEdges.MSG0");
        self.maps.small.remove(candidate);
        self.maps.edge_faces.unbind(candidate);
        self.add_status(ShapeExtendStatus::Done2);
        Ok(index)
    }
}

/// The middle point of the 3D curve is farther from its start than its end.
fn is_circle(edge: &Edge) -> Result<bool, Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().curve_3d(edge, false)?;
    let Some(c3d) = curve.get() else {
        return Ok(false);
    };
    let (p1, p2, p3) = (c3d.d0(first)?, c3d.d0(last)?, c3d.d0((first + last) * 0.5)?);
    Ok(p1.distance(&p3) > p1.distance(&p2))
}
