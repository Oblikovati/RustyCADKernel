// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ProximityDistTool.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_ProximityDistTool`: the largest of the distances from the vertices of a
//! first triangle set to a second one, measured along the perpendiculars when there
//! are some (the proximity distance). The border statuses of the proximity points are
//! in `status`, the status and projection types in `prj_state`.
mod prj_state;
mod status;

pub use prj_state::{BRepExtremaPrjState, BRepExtremaProxPntStatus};

use super::BRepExtremaTriangleSet;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bvh::tools::{
    PrjStateInTriangle, TriangleProjectionState, point_box_square_distance_by_corners,
    point_triangle_projection,
};
use rustycad_tkmath::bvh::{
    BvhArray3d, BvhBaseTraverse, BvhDistance, BvhTraverse, BvhVec3d, array, select_elements,
};

/// `pointBoxSquareMaxDistance`: the square distance from a point to the farthest corner
/// of a box, per axis from the side of the box center.
fn point_box_square_max_distance(point: &BvhVec3d, c_min: &BvhVec3d, c_max: &BvhVec3d) -> f64 {
    let mut dist: f64 = 0.0;
    for i in 0..3 {
        let mut d: f64 = if point[i] <= 0.5 * (c_max[i] + c_min[i]) {
            c_max[i] - point[i]
        } else {
            point[i] - c_min[i]
        };
        d *= d;
        dist += d;
    }
    dist
}

/// `BRepExtrema_ProximityDistTool` (docs/topalgo-design.md section 13): for each
/// sampled vertex of the first set (then each additional vertex), the distance to the
/// second set along a perpendicular to one of its triangles (or the plain minimum
/// distance when there is none); the proximity distance is the largest, with its two
/// points and their border statuses.
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaProximityDistTool;
/// let tool = BRepExtremaProximityDistTool::new();
/// assert_eq!(tool.proximity_distance(), -1.0);
/// ```
pub struct BRepExtremaProximityDistTool {
    base: BvhDistance<f64, BvhVec3d>,
    my_min_distance: f64,
    my_min_dist_point: BvhVec3d,
    my_extrema_point: BvhVec3d,
    my_prox_dist: f64,
    my_pnt1: BvhVec3d,
    my_pnt2: BvhVec3d,
    my_pnt_status1: BRepExtremaProxPntStatus,
    my_pnt_status2: BRepExtremaProxPntStatus,
    my_set1: Handle<BRepExtremaTriangleSet>,
    my_set2: Handle<BRepExtremaTriangleSet>,
    my_shape_list1: Vec<Shape>,
    my_shape_list2: Vec<Shape>,
    my_nb_samples1: i32,
    my_is_prox_vtx1_from_add_set: bool,
    my_add_vertices1: BvhArray3d,
    my_add_status1: Vec<BRepExtremaProxPntStatus>,
    my_prox_vtx_idx1: i32,
    my_prox_prj_state: BRepExtremaPrjState,
    my_ext_prj_state: BRepExtremaPrjState,
    my_min_prj_state: BRepExtremaPrjState,
}

impl Default for BRepExtremaProximityDistTool {
    fn default() -> Self {
        Self::new()
    }
}

/// The inputs of the loading constructor: the first set with its sample count, its
/// additional vertices and their statuses, the second set, and the shape lists.
pub struct BRepExtremaProximityInputs<'a> {
    /// `theSet1`.
    pub set1: &'a Handle<BRepExtremaTriangleSet>,
    /// `theNbSamples1`.
    pub nb_samples1: i32,
    /// `theAddVertices1`.
    pub add_vertices1: &'a BvhArray3d,
    /// `theAddStatus1`.
    pub add_status1: &'a [BRepExtremaProxPntStatus],
    /// `theSet2`.
    pub set2: &'a Handle<BRepExtremaTriangleSet>,
    /// `theShapeList1`.
    pub shape_list1: &'a [Shape],
    /// `theShapeList2`.
    pub shape_list2: &'a [Shape],
}

impl BRepExtremaProximityDistTool {
    /// `BRepExtrema_ProximityDistTool()`: no sets; proximity distance -1, statuses
    /// unknown; `BRepExtremaProximityDistTool::new()`.
    pub fn new() -> Self {
        Self {
            base: BvhDistance::default(),
            my_min_distance: f64::MAX,
            my_min_dist_point: BvhVec3d::default(),
            my_extrema_point: BvhVec3d::default(),
            my_prox_dist: -1.0,
            my_pnt1: BvhVec3d::default(),
            my_pnt2: BvhVec3d::default(),
            my_pnt_status1: BRepExtremaProxPntStatus::Unknown,
            my_pnt_status2: BRepExtremaProxPntStatus::Unknown,
            my_set1: Handle::default(),
            my_set2: Handle::default(),
            my_shape_list1: Vec::new(),
            my_shape_list2: Vec::new(),
            my_nb_samples1: 0,
            my_is_prox_vtx1_from_add_set: false,
            my_add_vertices1: BvhArray3d::default(),
            my_add_status1: Vec::new(),
            my_prox_vtx_idx1: -1,
            my_prox_prj_state: BRepExtremaPrjState::default(),
            my_ext_prj_state: BRepExtremaPrjState::default(),
            my_min_prj_state: BRepExtremaPrjState::default(),
        }
    }

    /// `BRepExtrema_ProximityDistTool(theSet1, theNbSamples1, theAddVertices1,
    /// theAddStatus1, theSet2, theShapeList1, theShapeList2)`: loads everything;
    /// `BRepExtremaProximityDistTool::with_inputs(&inputs)`.
    pub fn with_inputs(inputs: &BRepExtremaProximityInputs<'_>) -> Self {
        let mut tool: Self = Self::new();
        tool.my_nb_samples1 = inputs.nb_samples1;
        tool.load_triangle_sets(inputs.set1, inputs.set2);
        tool.load_shape_lists(inputs.shape_list1, inputs.shape_list2);
        tool.load_additional_points_first_set(inputs.add_vertices1, inputs.add_status1);
        tool
    }

    /// `LoadTriangleSets(theSet1, theSet2)`; `tool.load_triangle_sets(&set1, &set2)`.
    pub fn load_triangle_sets(
        &mut self,
        set1: &Handle<BRepExtremaTriangleSet>,
        set2: &Handle<BRepExtremaTriangleSet>,
    ) {
        self.my_set1 = set1.clone();
        self.my_set2 = set2.clone();
    }

    /// `LoadShapeLists(theShapeList1, theShapeList2)`: the sub-shapes of each set, by
    /// shape index; `tool.load_shape_lists(&list1, &list2)`.
    pub fn load_shape_lists(&mut self, shape_list1: &[Shape], shape_list2: &[Shape]) {
        self.my_shape_list1 = shape_list1.to_vec();
        self.my_shape_list2 = shape_list2.to_vec();
    }

    /// `LoadAdditionalPointsFirstSet(theAddVertices1, theAddStatus1)`: the additional
    /// vertices of the first set and their statuses;
    /// `tool.load_additional_points_first_set(&vertices, &statuses)`.
    pub fn load_additional_points_first_set(
        &mut self,
        add_vertices1: &BvhArray3d,
        add_status1: &[BRepExtremaProxPntStatus],
    ) {
        self.my_add_vertices1 = add_vertices1.clone();
        self.my_add_status1 = add_status1.to_vec();
    }

    /// `goThroughtSet1(theVertices1, theIsAdditionalSet)`: the distance of every sampled
    /// vertex (a step of the size over the sample count); a larger one becomes the
    /// proximity. Stops at the first vertex without a distance while none is known.
    fn go_through_set1(
        &mut self,
        vertices1: &BvhArray3d,
        is_additional_set: bool,
    ) -> Result<(), Failure> {
        let vtx_size: i32 = array::size(vertices1);
        let vtx_step: i32 = (if self.my_nb_samples1 <= 0 {
            1
        } else {
            vtx_size / self.my_nb_samples1
        })
        .max(1);
        let mut vtx_idx: i32 = 0;
        while vtx_idx < vtx_size {
            self.base.set_distance(f64::MAX);
            self.my_min_distance = f64::MAX;
            self.base.set_done(false);
            let vertex: BvhVec3d = *array::value(vertices1, vtx_idx)?;
            self.base.set_object(vertex);
            self.compute_distance()?;
            if !self.base.is_done() && self.my_prox_dist < 0.0 {
                return Ok(());
            }
            if self.base.is_done() && self.base.distance() > self.my_prox_dist {
                self.my_pnt1 = vertex;
                self.my_pnt2 = self.my_extrema_point;
                self.my_prox_dist = self.base.distance();
                self.my_prox_vtx_idx1 = vtx_idx;
                self.my_is_prox_vtx1_from_add_set = is_additional_set;
                self.my_prox_prj_state = self.my_ext_prj_state;
            }
            vtx_idx += vtx_step;
        }
        Ok(())
    }

    /// `Perform()`: the vertices of the first set, then the additional ones, against the
    /// second set; done when a proximity distance was found, then the statuses of its
    /// points. Standard_NullObject before the sets are loaded; `tool.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        let set1 = self.my_set1.get().ok_or_else(|| {
            null_object("BRepExtrema_ProximityDistTool::Perform: the first set is null")
        })?;
        let vertices1: BvhArray3d = set1.vertices();
        self.go_through_set1(&vertices1, false)?;
        let add_vertices1: BvhArray3d = self.my_add_vertices1.clone();
        self.go_through_set1(&add_vertices1, true)?;
        self.base.set_done(self.my_prox_dist > -1.0);
        if self.base.is_done() {
            self.define_status_prox_pnt()?;
        }
        Ok(())
    }

    /// `ComputeDistance()` (protected): the selection through the second set; without a
    /// perpendicular distance the smallest plain distance is taken (when there is one).
    /// Returns the distance; `tool.compute_distance()?`.
    pub fn compute_distance(&mut self) -> Result<f64, Failure> {
        let selected: i32 = self.select()?;
        self.base.set_done(selected > 0);
        if !self.base.is_done() {
            if self.my_min_distance < f64::MAX {
                self.my_extrema_point = self.my_min_dist_point;
                self.my_ext_prj_state = self.my_min_prj_state;
                self.base.set_done(true);
            }
            self.base.set_distance(self.my_min_distance);
        }
        Ok(self.base.distance())
    }

    /// `BVH_Traverse::Select()` over the tree of the second set (`SetBVHSet(mySet2)`).
    fn select(&mut self) -> Result<i32, Failure> {
        let set2 = self.my_set2.clone();
        let set2 = set2
            .get()
            .ok_or_else(|| null_object("BRepExtrema_ProximityDistTool: the second set is null"))?;
        let tree = set2.take_bvh()?;
        let selected = select_elements(self, Some(&tree));
        set2.restore_bvh(tree);
        selected
    }

    /// `IsDone()` of `BVH_Distance`: the flag of the last distance, then of `Perform`;
    /// `tool.is_done()`.
    pub fn is_done(&self) -> bool {
        self.base.is_done()
    }

    /// `ProximityPoints(thePoint1, thePoint2)`: the points of the proximity distance;
    /// `let (p1, p2) = tool.proximity_points();`.
    pub fn proximity_points(&self) -> (BvhVec3d, BvhVec3d) {
        (self.my_pnt1, self.my_pnt2)
    }

    /// `ProximityPointsStatus(thePointStatus1, thePointStatus2)`: their statuses;
    /// `let (s1, s2) = tool.proximity_points_status();`.
    pub fn proximity_points_status(&self) -> (BRepExtremaProxPntStatus, BRepExtremaProxPntStatus) {
        (self.my_pnt_status1, self.my_pnt_status2)
    }

    /// `ProximityDistance()`: the proximity distance (-1 when none was found);
    /// `tool.proximity_distance()`.
    pub fn proximity_distance(&self) -> f64 {
        self.my_prox_dist
    }

    /// The perpendicular part of `Accept`: true when the direction from the triangle
    /// point to the object is not along the triangle normal (the segment normal in the
    /// plane of the object for a degenerate triangle).
    fn is_off_perpendicular(
        &self,
        [a, b, c]: &[BvhVec3d; 3],
        direct: &BvhVec3d,
        sq_distance: f64,
    ) -> bool {
        let ab: BvhVec3d = *b - *a;
        let norm: BvhVec3d = if b.is_equal(c) {
            // A degenerate triangle (a segment).
            let ap: BvhVec3d = *self.base.object() - *a;
            BvhVec3d::cross(&BvhVec3d::cross(&ap, &ab), &ab)
        } else {
            BvhVec3d::cross(&ab, &(*c - *a))
        };
        let norm_sq_len: f64 = norm.dot(&norm);
        let cross_cross: BvhVec3d = BvhVec3d::cross(direct, &norm);
        let cross_cross_sq_len: f64 = cross_cross.dot(&cross_cross);
        cross_cross_sq_len > precision::square_confusion() * sq_distance * norm_sq_len
    }
}

impl BvhBaseTraverse<f64> for BRepExtremaProximityDistTool {
    fn is_metric_better(&self, left: &f64, right: &f64) -> bool {
        self.base.is_metric_better(left, right)
    }
    fn reject_metric(&self, metric: &f64) -> bool {
        self.base.reject_metric(metric)
    }
    fn stop(&self) -> bool {
        self.base.stop()
    }
}

impl BvhTraverse<BvhVec3d, f64> for BRepExtremaProximityDistTool {
    /// `RejectNode(theCornerMin, theCornerMax, theMetric)`: the metric is the distance
    /// to the box; rejected when it exceeds the current distance or when the farthest
    /// corner is nearer than the proximity distance.
    fn reject_node(&self, corner_min: &BvhVec3d, corner_max: &BvhVec3d, metric: &mut f64) -> bool {
        let object: &BvhVec3d = self.base.object();
        *metric = point_box_square_distance_by_corners(object, corner_min, corner_max).sqrt();
        let max_metric: f64 = point_box_square_max_distance(object, corner_min, corner_max).sqrt();
        *metric > self.base.distance() || max_metric < self.my_prox_dist
    }

    /// `Accept(theTrgIdx, metric)`: the projection of the object on the triangle; a
    /// distance off the perpendicular only lowers the plain minimum (false); a
    /// perpendicular one lowers the current distance by more than Confusion (true).
    fn accept(&mut self, trg_idx: i32, _metric: &f64) -> Result<bool, Failure> {
        let set2 = self.my_set2.clone();
        let vertices: [BvhVec3d; 3] = set2.get_vertices(trg_idx)?;
        let mut projection = TriangleProjectionState {
            state: PrjStateInTriangle::Vertex,
            first_node: -1,
            last_node: -1,
        };
        let [a, b, c] = &vertices;
        let nearest_pnt: BvhVec3d =
            point_triangle_projection(self.base.object(), a, b, c, Some(&mut projection));
        let prj_state = BRepExtremaPrjState::new(
            trg_idx,
            projection.state,
            projection.first_node,
            projection.last_node,
        );
        let direct: BvhVec3d = *self.base.object() - nearest_pnt;
        let sq_distance: f64 = direct.dot(&direct);
        // The point belongs to the triangle when it is within Confusion.
        if sq_distance > precision::square_confusion()
            && self.is_off_perpendicular(&vertices, &direct, sq_distance)
        {
            if self.my_min_distance - sq_distance.sqrt() > precision::confusion() {
                self.my_min_distance = sq_distance.sqrt();
                self.my_min_dist_point = nearest_pnt;
                self.my_min_prj_state = prj_state;
            }
            return Ok(false);
        }
        // The distance is under a perpendicular.
        if self.base.distance() - sq_distance.sqrt() > precision::confusion() {
            self.base.set_distance(sq_distance.sqrt());
            self.my_extrema_point = nearest_pnt;
            self.my_ext_prj_state = prj_state;
            return Ok(true);
        }
        Ok(false)
    }
}
