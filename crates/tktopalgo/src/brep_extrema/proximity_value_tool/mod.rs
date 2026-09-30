// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ProximityValueTool.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_ProximityValueTool`: the proximity value of two triangulated shapes
//! (two edges or two faces), the smaller of the proximity distances from each to the
//! other, after refining the coarser mesh. The refinement is in `refine`, the
//! inspector of its cell filter in `vertex_inspector`.
mod refine;
mod vertex_inspector;

pub use vertex_inspector::BRepExtremaVertexInspector;

use super::{
    BRepExtremaProxPntStatus, BRepExtremaProximityDistTool, BRepExtremaProximityInputs,
    BRepExtremaTriangleSet,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::CellFilter;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bvh::{BvhArray3d, BvhVec3d};
use rustycad_tkmath::gp::Point3;

/// The per-shape state of the tool: the set, the list and first shape, the refinement
/// data and the additional vertices.
#[derive(Default)]
struct ProximityShape {
    set: Handle<BRepExtremaTriangleSet>,
    list: Vec<Shape>,
    shape: Shape,
    add_vertices: BvhArray3d,
    add_status: Vec<BRepExtremaProxPntStatus>,
    is_init: bool,
    is_refinement_required: bool,
    nb_nodes: i32,
    step: f64,
    shape_type: Option<ShapeEnum>,
    nb_samples: i32,
}

/// The result of one direction: the distance and the points and statuses on the first
/// and the second shape of that direction.
struct DirectedProximity {
    distance: f64,
    points: (BvhVec3d, BvhVec3d),
    statuses: (BRepExtremaProxPntStatus, BRepExtremaProxPntStatus),
}

/// `BRepExtrema_ProximityValueTool` (docs/topalgo-design.md section 13): the proximity
/// value of two edges or two faces (the first shapes of the lists), the minimal
/// diameter of a tube containing both edges or thickness of a shell containing both
/// faces, with its points and their statuses.
///
/// The refinement flags and the additional vertices are never reset: a later
/// `Perform` adds more vertices to those of the earlier ones (native quirk).
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaProximityValueTool;
/// let tool = BRepExtremaProximityValueTool::new();
/// assert!(!tool.is_done());
/// assert_eq!(tool.distance(), f64::MAX);
/// ```
pub struct BRepExtremaProximityValueTool {
    shape1: ProximityShape,
    shape2: ProximityShape,
    my_cells: CellFilter<BRepExtremaVertexInspector>,
    my_inspector: BRepExtremaVertexInspector,
    my_distance: f64,
    my_is_done: bool,
    my_pnt1: Point3,
    my_pnt2: Point3,
    my_pnt_status1: BRepExtremaProxPntStatus,
    my_pnt_status2: BRepExtremaProxPntStatus,
}

impl Default for BRepExtremaProximityValueTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaProximityValueTool {
    /// `BRepExtrema_ProximityValueTool()`: nothing loaded, distance DBL_MAX;
    /// `BRepExtremaProximityValueTool::new()`.
    pub fn new() -> Self {
        Self {
            shape1: ProximityShape::default(),
            shape2: ProximityShape::default(),
            my_cells: CellFilter::new(0.0),
            my_inspector: BRepExtremaVertexInspector::new(),
            my_distance: f64::MAX,
            my_is_done: false,
            my_pnt1: Point3::default(),
            my_pnt2: Point3::default(),
            my_pnt_status1: BRepExtremaProxPntStatus::default(),
            my_pnt_status2: BRepExtremaProxPntStatus::default(),
        }
    }

    /// `BRepExtrema_ProximityValueTool(theSet1, theSet2, theShapeList1, theShapeList2)`:
    /// loads the lists, then the sets;
    /// `BRepExtremaProximityValueTool::with_sets(&s1, &s2, &l1, &l2)?`.
    pub fn with_sets(
        set1: &Handle<BRepExtremaTriangleSet>,
        set2: &Handle<BRepExtremaTriangleSet>,
        shape_list1: &[Shape],
        shape_list2: &[Shape],
    ) -> Result<Self, Failure> {
        let mut tool: Self = Self::new();
        tool.load_shape_lists(shape_list1, shape_list2)?;
        tool.load_triangle_sets(set1, set2);
        Ok(tool)
    }

    /// `LoadTriangleSets(theSet1, theSet2)`: the sets of the next computation;
    /// `tool.load_triangle_sets(&set1, &set2)`.
    pub fn load_triangle_sets(
        &mut self,
        set1: &Handle<BRepExtremaTriangleSet>,
        set2: &Handle<BRepExtremaTriangleSet>,
    ) {
        self.shape1.set = set1.clone();
        self.shape2.set = set2.clone();
        self.mark_dirty();
    }

    /// `LoadShapeLists(theShapeList1, theShapeList2)`: the lists and, from the first
    /// shape of each, its type, node count and refinement step (a shape is initialized
    /// when it is a triangulated face or an edge with a 3D polygon and a step of at
    /// least Confusion). Standard_OutOfRange for an empty list (an unchecked read
    /// natively); `tool.load_shape_lists(&list1, &list2)?`.
    pub fn load_shape_lists(
        &mut self,
        shape_list1: &[Shape],
        shape_list2: &[Shape],
    ) -> Result<(), Failure> {
        for (state, list) in [
            (&mut self.shape1, shape_list1),
            (&mut self.shape2, shape_list2),
        ] {
            state.list = list.to_vec();
            state.shape = list
                .first()
                .ok_or_else(|| {
                    Failure::out_of_range(
                        "BRepExtrema_ProximityValueTool::LoadShapeLists: an empty shape list",
                    )
                })?
                .clone();
        }
        self.shape1.is_init = refine::info_for_refinement(&mut self.shape1)?;
        self.shape2.is_init = refine::info_for_refinement(&mut self.shape2)?;
        self.mark_dirty();
        Ok(())
    }

    /// `SetNbSamplePoints(theSamples1, theSamples2)` (the native defaults are 0): the
    /// sample counts of the vertices of each shape (all the vertices for 0 or less);
    /// `tool.set_nb_sample_points(0, 0)`.
    pub fn set_nb_sample_points(&mut self, samples1: i32, samples2: i32) {
        self.shape1.nb_samples = samples1;
        self.shape2.nb_samples = samples2;
        self.mark_dirty();
    }

    /// `computeProximityDist`: the proximity distance from the vertices of `from` (and
    /// its additional vertices) to the triangles of `to`; `None` when not done.
    fn compute_proximity_dist(
        from: &ProximityShape,
        to: &ProximityShape,
    ) -> Result<Option<DirectedProximity>, Failure> {
        let inputs = BRepExtremaProximityInputs {
            set1: &from.set,
            nb_samples1: from.nb_samples,
            add_vertices1: &from.add_vertices,
            add_status1: &from.add_status,
            set2: &to.set,
            shape_list1: &from.list,
            shape_list2: &to.list,
        };
        let mut tool = BRepExtremaProximityDistTool::with_inputs(&inputs);
        tool.perform()?;
        if !tool.is_done() {
            return Ok(None);
        }
        Ok(Some(DirectedProximity {
            distance: tool.proximity_distance(),
            points: tool.proximity_points(),
            statuses: tool.proximity_points_status(),
        }))
    }

    /// `Perform(theTolerance)`: for two initialized shapes of one type, the refinement
    /// of the coarser one, then the proximity distances in both directions; the smaller
    /// one (the second on a tie) is the proximity value, returned in `tolerance`
    /// (unchanged when not done); `tool.perform(&mut tolerance)?`.
    pub fn perform(&mut self, tolerance: &mut f64) -> Result<(), Failure> {
        if !self.shape1.is_init
            || !self.shape2.is_init
            || self.shape1.shape_type != self.shape2.shape_type
        {
            return Ok(());
        }
        if !self.shapes_additional_vertices()? {
            return Ok(());
        }
        let Some(first) = Self::compute_proximity_dist(&self.shape1, &self.shape2)? else {
            return Ok(());
        };
        let Some(second) = Self::compute_proximity_dist(&self.shape2, &self.shape1)? else {
            return Ok(());
        };
        self.set_proximity(&first, &second);
        *tolerance = self.my_distance;
        Ok(())
    }

    /// The smaller of both directed proximities (the second on a tie) becomes the
    /// result; the tool is done.
    fn set_proximity(&mut self, first: &DirectedProximity, second: &DirectedProximity) {
        // The second direction measures from the second shape: its points are swapped.
        let (distance, (p1, p2), (s1, s2)) = if first.distance < second.distance {
            (first.distance, first.points, first.statuses)
        } else {
            (
                second.distance,
                (second.points.1, second.points.0),
                (second.statuses.1, second.statuses.0),
            )
        };
        self.my_distance = distance;
        self.my_pnt1 = Point3::new(p1.x, p1.y, p1.z);
        self.my_pnt2 = Point3::new(p2.x, p2.y, p2.z);
        self.my_pnt_status1 = s1;
        self.my_pnt_status2 = s2;
        self.my_is_done = true;
    }

    /// `IsDone()`: true when a proximity value was computed since the last change;
    /// `tool.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `MarkDirty()`: the results are outdated; `tool.mark_dirty()`.
    pub fn mark_dirty(&mut self) {
        self.my_is_done = false;
    }

    /// `Distance()`: the proximity value (DBL_MAX before any); `tool.distance()`.
    pub fn distance(&self) -> f64 {
        self.my_distance
    }

    /// `ProximityPoints(thePoint1, thePoint2)`: the points of the proximity value;
    /// `let (p1, p2) = tool.proximity_points();`.
    pub fn proximity_points(&self) -> (Point3, Point3) {
        (self.my_pnt1, self.my_pnt2)
    }

    /// `ProximityPointsStatus(thePointStatus1, thePointStatus2)`: their statuses;
    /// `let (s1, s2) = tool.proximity_points_status();`.
    pub fn proximity_points_status(&self) -> (BRepExtremaProxPntStatus, BRepExtremaProxPntStatus) {
        (self.my_pnt_status1, self.my_pnt_status2)
    }
}
