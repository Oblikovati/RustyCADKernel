//! `BOPAlgo_EdgeEdge`, the edge/edge task class of `BOPAlgo_PaveFiller_3.cxx`.
use crate::bop_algo::tools::{TRSF_TO_POINT_CRITERIA, trsf_to_point};
use crate::bop_ds::PaveBlockHandle;
use crate::int_tools::{CommonPrt, EdgeEdge};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

/// `BOPAlgo_EdgeEdge`: an IntTools_EdgeEdge between two pave blocks, run near the origin
/// when the boxes lie far away.
pub(in crate::bop_algo::pave_filler) struct EdgeEdgeTask {
    pub(in crate::bop_algo::pave_filler) block1: PaveBlockHandle,
    pub(in crate::bop_algo::pave_filler) block2: PaveBlockHandle,
    edges: [Edge; 2],
    ranges: [(f64, f64); 2],
    boxes: [BoundingBox3; 2],
    fuzzy: f64,
    quick: bool,
    done: bool,
    failed: bool,
    pub(in crate::bop_algo::pave_filler) parts: Vec<CommonPrt>,
}

impl EdgeEdgeTask {
    pub(in crate::bop_algo::pave_filler) fn new(
        blocks: [PaveBlockHandle; 2],
        edges: [Edge; 2],
        ranges: [(f64, f64); 2],
    ) -> Self {
        let [block1, block2] = blocks;
        Self {
            block1,
            block2,
            edges,
            ranges,
            boxes: [BoundingBox3::default(), BoundingBox3::default()],
            fuzzy: precision::confusion(),
            quick: false,
            done: false,
            failed: false,
            parts: Vec::new(),
        }
    }

    pub(in crate::bop_algo::pave_filler) fn configure(
        &mut self,
        boxes: [BoundingBox3; 2],
        fuzzy: f64,
        quick: bool,
    ) {
        self.boxes = boxes;
        self.fuzzy = fuzzy;
        self.quick = quick;
    }

    /// Native `BOPAlgo_EdgeEdge::Perform`; exceptions become the task's error.
    pub(in crate::bop_algo::pave_filler) fn perform(&mut self) {
        if self.run().is_err() {
            self.failed = true;
        }
    }

    /// `!IsDone() || HasErrors()`.
    pub(in crate::bop_algo::pave_filler) fn failed(&self) -> bool {
        !self.done || self.failed
    }

    fn run(&mut self) -> Result<(), Failure> {
        let translation = trsf_to_point(
            &self.boxes[0],
            &self.boxes[1],
            Point3::default(),
            TRSF_TO_POINT_CRITERIA,
        )?;
        let mut edges = self.edges.clone();
        if let Some(transform) = translation {
            let location = Location::new(transform);
            for edge in &mut edges {
                *edge = Edge::unchecked(edge.moved(&location, false)?);
            }
        }
        let mut intersector = EdgeEdge::new();
        intersector.set_edge1_range(&edges[0], self.ranges[0].0, self.ranges[0].1);
        intersector.set_edge2_range(&edges[1], self.ranges[1].0, self.ranges[1].1);
        intersector.set_fuzzy_value(self.fuzzy);
        intersector.use_quick_coincidence_check(self.quick);
        let result = intersector.perform();
        self.done = intersector.is_done();
        self.parts = intersector.common_parts().to_vec();
        if translation.is_some() {
            for part in &mut self.parts {
                part.set_edge1(&self.edges[0]);
                part.set_edge2(&self.edges[1]);
            }
        }
        result
    }
}
