// Rust translation of BRepBuilderAPI_Sewing::SameRange, SameParameter and
// SameParameterEdge(edge, seqEdges, seqForward, mapMerged, locReShape)
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The merge of a sequence of sections into one edge, and the guarded GeomLib and
//! BRepLib calls it uses.
use super::{BRepBuilderAPISewing, live_reshape};
use crate::brep_lib;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_lib;

/// The native default tolerance of `BRepLib::SameParameter(E)`.
const SAME_PARAMETER_TOLERANCE: f64 = 1.0e-5;

impl BRepBuilderAPISewing {
    /// `SameRange(CurvePtr, FirstOnCurve, LastOnCurve, RequestedFirst,
    /// RequestedLast)`: `GeomLib::SameRange` at Precision::PConfusion(); an exception
    /// is caught and the curve assigned so far (null at first) is returned.
    pub(crate) fn same_range_pcurve(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        on_curve: (f64, f64),
        requested: (f64, f64),
    ) -> Handle<dyn Geom2dCurve> {
        let mut new_curve: Handle<dyn Geom2dCurve> = Handle::default();
        // The native try/catch keeps what GeomLib assigned before it raised.
        let _ = geom_lib::same_range(
            precision::p_confusion(),
            curve,
            on_curve,
            requested,
            &mut new_curve,
        );
        new_curve
    }

    /// `SameParameter(edge)`: `BRepLib::SameParameter(edge)` (tolerance 1e-5), an
    /// exception being caught.
    pub(crate) fn same_parameter_guarded(&self, edge: &Edge) {
        // The native try/catch ignores the failure; the edge keeps what was done.
        let _ = brep_lib::same_parameter_edge(edge, SAME_PARAMETER_TOLERANCE);
    }

    /// The images of a section by the context and by `loc_reshape` (the second one is
    /// kept only when the two re-shapers differ, as natively).
    fn section_image(
        &self,
        edge: &Shape,
        loc_reshape: &Handle<dyn ReShape>,
    ) -> Result<Edge, Failure> {
        let mut section =
            topods::edge(&live_reshape(&self.my_re_shape)?.apply(edge, ShapeEnum::Shape)?)?;
        let local = live_reshape(loc_reshape)?.apply(&section, ShapeEnum::Shape)?;
        if *loc_reshape != self.my_re_shape {
            section = topods::edge(&local)?;
        }
        Ok(section)
    }

    /// The faces of the boundary of a section, once each, appended to `faces`.
    fn append_bound_faces(
        &self,
        faces: &mut List<Shape>,
        merged: &mut MapOfShape,
        bound_faces: &List<Shape>,
    ) {
        for face in bound_faces.iter() {
            if merged.add(face.clone()) {
                faces.append(face.clone());
            }
        }
    }

    /// The presentation edge of the analysis mode: an empty copy of the reference with
    /// the nodes of its vertices.
    fn presentation_edge(&self, edge1: &Edge) -> Result<Edge, Failure> {
        let (mut v1, mut v2) = topexp::vertices(edge1, false);
        if let Some(node) = self.my_vertex_node.seek(&v1) {
            v1 = topods::vertex(node)?;
        }
        if let Some(node) = self.my_vertex_node.seek(&v2) {
            v2 = topods::vertex(node)?;
        }
        let mut new_edge = edge1.clone();
        new_edge.empty_copy()?;
        let builder = BRepBuilder::default();
        let mut an_edge = new_edge.oriented(Orientation::Forward);
        builder.add(&mut an_edge, &v1.oriented(Orientation::Forward))?;
        builder.add(&mut an_edge, &v2.oriented(Orientation::Reversed))?;
        Ok(new_edge)
    }

    /// Merges one candidate section into the merged edge (its faces joined to the merged
    /// faces, the orientation flipped with the section); false when the section is
    /// floating or the pair does not merge.
    fn merge_into(
        &self,
        state: &mut SequenceMerge,
        oedge2: &Shape,
        forward: bool,
        loc_reshape: &Handle<dyn ReShape>,
    ) -> Result<bool, Failure> {
        let edge2 = self.section_image(oedge2, loc_reshape)?;
        let orientation = forward == state.is_forward;
        // Skip floating edge.
        let Some(list_faces2) = self.my_bound_faces.seek(&self.bound_of(oedge2)) else {
            return Ok(false);
        };
        let mut which_sec: i32 = 1;
        let pair = (&state.edge, &edge2, &state.faces, list_faces2);
        let new_edge = self.same_parameter_edge_of_pair(pair, orientation, &mut which_sec, true)?;
        if new_edge.is_null() {
            return Ok(false);
        }
        self.append_bound_faces(&mut state.faces, &mut state.merged_faces, list_faces2);
        if !orientation && which_sec != 1 {
            state.is_forward = !state.is_forward;
        }
        state.edge = new_edge;
        Ok(true)
    }

    /// `SameParameterEdge(edge, seqEdges, seqForward, mapMerged, locReShape)`: merges
    /// the candidate sections into the reference one (only the first in manifold
    /// mode), each merged section recorded in `map_merged`; the result is oriented as
    /// the reference, or null when nothing was merged.
    pub(crate) fn same_parameter_edge_of_sequence(
        &self,
        edge: &Shape,
        seq_edges: &Sequence<Shape>,
        seq_forward: &Sequence<bool>,
        map_merged: &mut MapOfShape,
        loc_reshape: &Handle<dyn ReShape>,
    ) -> Result<Edge, Failure> {
        let mut state = SequenceMerge {
            edge: self.section_image(edge, loc_reshape)?,
            faces: List::default(),
            merged_faces: MapOfShape::default(),
            is_forward: true,
        };
        if self.my_sewing {
            if let Some(faces) = self.my_bound_faces.seek(&self.bound_of(edge)) {
                self.append_bound_faces(&mut state.faces, &mut state.merged_faces, faces);
            }
        } else {
            state.edge = self.presentation_edge(&state.edge)?;
        }
        let mut is_done = false;
        for i in 1..=seq_edges.length() as usize {
            let oedge2 = seq_edges.value(i)?;
            if self.my_sewing
                && !self.merge_into(&mut state, oedge2, *seq_forward.value(i)?, loc_reshape)?
            {
                continue;
            }
            map_merged.add(oedge2.clone());
            is_done = true;
            if !self.my_nonmanifold {
                break;
            }
        }
        if !is_done {
            state.edge.nullify();
            return Ok(state.edge);
        }
        state.edge.set_orientation(if state.is_forward {
            Orientation::Forward
        } else {
            Orientation::Reversed
        });
        Ok(state.edge)
    }
}

/// The state of `SameParameterEdge(edge, seqEdges, ...)`: the merged edge, the faces of
/// the merged sections (each once) and the orientation of the result.
struct SequenceMerge {
    edge: Edge,
    faces: List<Shape>,
    merged_faces: MapOfShape,
    is_forward: bool,
}
