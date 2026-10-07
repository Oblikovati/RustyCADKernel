//! `BOPAlgo_EdgeFace`, the edge/face task class of `BOPAlgo_PaveFiller_5.cxx`.
use crate::bop_algo::tools::{TRSF_TO_POINT_CRITERIA, trsf_to_point};
use crate::bop_ds::PaveBlockHandle;
use crate::int_tools::{CommonPrt, Context, EdgeFace, Range};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;
use std::sync::Arc;

/// `BOPAlgo_EdgeFace`: an IntTools_EdgeFace on one pave block, run near the origin when the
/// boxes lie far away.
pub(in crate::bop_algo::pave_filler) struct EdgeFaceTask {
    pub(in crate::bop_algo::pave_filler) edge_index: i32,
    pub(in crate::bop_algo::pave_filler) face_index: i32,
    pub(in crate::bop_algo::pave_filler) block: PaveBlockHandle,
    pub(in crate::bop_algo::pave_filler) edge: Edge,
    pub(in crate::bop_algo::pave_filler) face: Face,
    boxes: [BoundingBox3; 2],
    fuzzy: f64,
    quick: bool,
    range: Range,
    pub(in crate::bop_algo::pave_filler) new_shrunk_range: Range,
    done: bool,
    failed: bool,
    pub(in crate::bop_algo::pave_filler) minimal_distance: f64,
    pub(in crate::bop_algo::pave_filler) parts: Vec<CommonPrt>,
}

impl EdgeFaceTask {
    pub(in crate::bop_algo::pave_filler) fn new(
        indices: (i32, i32),
        block: &PaveBlockHandle,
        edge: &Edge,
        face: &Face,
    ) -> Self {
        Self {
            edge_index: indices.0,
            face_index: indices.1,
            block: Arc::clone(block),
            edge: edge.clone(),
            face: face.clone(),
            boxes: [BoundingBox3::default(), BoundingBox3::default()],
            fuzzy: precision::confusion(),
            quick: false,
            range: Range::new(0., 0.),
            new_shrunk_range: Range::new(0., 0.),
            done: false,
            failed: false,
            minimal_distance: f64::MAX,
            parts: Vec::new(),
        }
    }

    pub(in crate::bop_algo::pave_filler) fn configure(
        &mut self,
        boxes: [BoundingBox3; 2],
        fuzzy: f64,
        quick: bool,
        range: Range,
    ) {
        self.boxes = boxes;
        self.fuzzy = fuzzy;
        self.quick = quick;
        self.range = range;
    }

    /// Native `BOPAlgo_EdgeFace::Perform`; exceptions become the task's error.
    pub(in crate::bop_algo::pave_filler) fn perform(&mut self, context: &Handle<Context>) {
        if self.run(context).is_err() {
            self.failed = true;
        }
    }

    pub(in crate::bop_algo::pave_filler) fn failed(&self) -> bool {
        !self.done || self.failed
    }

    fn run(&mut self, context: &Handle<Context>) -> Result<(), Failure> {
        let translation = trsf_to_point(
            &self.boxes[0],
            &self.boxes[1],
            Point3::default(),
            TRSF_TO_POINT_CRITERIA,
        )?;
        let (mut edge, mut face) = (self.edge.clone(), self.face.clone());
        if let Some(transform) = translation {
            let location = Location::new(transform);
            edge = Edge::unchecked(edge.moved(&location, false)?);
            face = Face::unchecked(face.moved(&location, false)?);
        }
        let mut intersector = EdgeFace::new();
        intersector.set_edge(&edge);
        intersector.set_face(&face);
        intersector.set_range(&self.range);
        intersector.set_fuzzy_value(self.fuzzy);
        intersector.use_quick_coincidence_check(self.quick);
        intersector.set_context(context);
        let result = intersector.perform();
        self.done = intersector.is_done();
        self.minimal_distance = intersector.minimal_distance();
        self.parts = intersector.common_parts().iter().cloned().collect();
        if translation.is_some() {
            for part in &mut self.parts {
                part.set_edge1(&self.edge);
            }
        }
        result
    }
}
