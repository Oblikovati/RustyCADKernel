// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ShapeProximity.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_ShapeProximity`: the overlapping faces of two triangulated shapes within
//! a tolerance, or (with an infinite tolerance) their proximity value.
use super::overlap_tool::BRepExtremaOverlapMap;
use super::{
    BRepExtremaOverlapTool, BRepExtremaProxPntStatus, BRepExtremaProximityValueTool,
    BRepExtremaTriangleSet,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

/// `initSubShapes(theShape, theSubshapesList, theTriangleSet)`: the faces then the edges
/// of the shape, and the triangle set initialized on them (created when null).
fn init_sub_shapes(
    shape: &Shape,
    sub_shapes: &mut Vec<Shape>,
    set: &mut Handle<BRepExtremaTriangleSet>,
) -> Result<bool, Failure> {
    sub_shapes.clear();
    for kind in [ShapeEnum::Face, ShapeEnum::Edge] {
        for sub_shape in Explorer::new(shape, kind, ShapeEnum::Shape)? {
            sub_shapes.push(sub_shape);
        }
    }
    if set.is_null() {
        *set = Handle::new(BRepExtremaTriangleSet::new());
    }
    set.init(sub_shapes)
}

/// `BRepExtrema_ShapeProximity` (docs/topalgo-design.md section 13): with a finite
/// tolerance, the faces of the two shapes whose triangles are nearer than it (the
/// overlap tool); with Precision::Infinite(), the proximity value of the first faces or
/// edges of the shapes (the value tool), which becomes the tolerance.
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaShapeProximity;
/// let tool = BRepExtremaShapeProximity::new(0.1);
/// assert_eq!(tool.tolerance(), 0.1);
/// assert!(!tool.is_done());
/// ```
pub struct BRepExtremaShapeProximity {
    my_tolerance: f64,
    my_is_init_s1: bool,
    my_is_init_s2: bool,
    my_shape_list1: Vec<Shape>,
    my_shape_list2: Vec<Shape>,
    my_element_set1: Handle<BRepExtremaTriangleSet>,
    my_element_set2: Handle<BRepExtremaTriangleSet>,
    my_nb_samples1: i32,
    my_nb_samples2: i32,
    my_prox_point1: Point3,
    my_prox_point2: Point3,
    my_prox_pnt_status1: BRepExtremaProxPntStatus,
    my_prox_pnt_status2: BRepExtremaProxPntStatus,
    my_overlap_tool: BRepExtremaOverlapTool,
    my_prox_val_tool: BRepExtremaProximityValueTool,
}

impl Default for BRepExtremaShapeProximity {
    /// The native default tolerance is Precision::Infinite().
    fn default() -> Self {
        Self::new(precision::infinite())
    }
}

impl BRepExtremaShapeProximity {
    /// `BRepExtrema_ShapeProximity(theTolerance)`: empty sets, no shape;
    /// `BRepExtremaShapeProximity::new(precision::infinite())`.
    pub fn new(tolerance: f64) -> Self {
        Self {
            my_tolerance: tolerance,
            my_is_init_s1: false,
            my_is_init_s2: false,
            my_shape_list1: Vec::new(),
            my_shape_list2: Vec::new(),
            my_element_set1: Handle::new(BRepExtremaTriangleSet::new()),
            my_element_set2: Handle::new(BRepExtremaTriangleSet::new()),
            my_nb_samples1: 0,
            my_nb_samples2: 0,
            my_prox_point1: Point3::default(),
            my_prox_point2: Point3::default(),
            my_prox_pnt_status1: BRepExtremaProxPntStatus::default(),
            my_prox_pnt_status2: BRepExtremaProxPntStatus::default(),
            my_overlap_tool: BRepExtremaOverlapTool::new(),
            my_prox_val_tool: BRepExtremaProximityValueTool::new(),
        }
    }

    /// `BRepExtrema_ShapeProximity(theShape1, theShape2, theTolerance)`: loads both
    /// shapes; `BRepExtremaShapeProximity::with_shapes(&s1, &s2, 0.1)?`.
    pub fn with_shapes(shape1: &Shape, shape2: &Shape, tolerance: f64) -> Result<Self, Failure> {
        let mut tool: Self = Self::new(tolerance);
        tool.load_shape1(shape1)?;
        tool.load_shape2(shape2)?;
        Ok(tool)
    }

    /// `Tolerance()`: the overlap distance (the proximity value after an infinite
    /// computation); `tool.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.my_tolerance
    }

    /// `SetTolerance(theTolerance)`; `tool.set_tolerance(0.5)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.my_tolerance = tolerance;
    }

    /// `Proximity()`: the proximity value, i.e. the tolerance; `tool.proximity()`.
    pub fn proximity(&self) -> f64 {
        self.tolerance()
    }

    /// The results of the tool of the current mode are outdated.
    fn mark_mode_dirty(&mut self) {
        if self.my_tolerance == precision::infinite() {
            self.my_prox_val_tool.mark_dirty();
        } else {
            self.my_overlap_tool.mark_dirty();
        }
    }

    /// `LoadShape1(theShape1)`: the faces then the edges of the shape and their mesh
    /// elements; returns the result of `Init` (always true); `tool.load_shape1(&s)?`.
    pub fn load_shape1(&mut self, shape1: &Shape) -> Result<bool, Failure> {
        self.my_is_init_s1 =
            init_sub_shapes(shape1, &mut self.my_shape_list1, &mut self.my_element_set1)?;
        self.mark_mode_dirty();
        Ok(self.my_is_init_s1)
    }

    /// `LoadShape2(theShape2)`: as `LoadShape1` for the second shape;
    /// `tool.load_shape2(&s)?`.
    pub fn load_shape2(&mut self, shape2: &Shape) -> Result<bool, Failure> {
        self.my_is_init_s2 =
            init_sub_shapes(shape2, &mut self.my_shape_list2, &mut self.my_element_set2)?;
        self.mark_mode_dirty();
        Ok(self.my_is_init_s2)
    }

    /// `SetNbSamples1(theNbSamples)`: the sample count of the first shape (0: all the
    /// nodes); `tool.set_nb_samples1(10)`.
    pub fn set_nb_samples1(&mut self, nb_samples: i32) {
        self.my_nb_samples1 = nb_samples;
    }

    /// `SetNbSamples2(theNbSamples)`; `tool.set_nb_samples2(10)`.
    pub fn set_nb_samples2(&mut self, nb_samples: i32) {
        self.my_nb_samples2 = nb_samples;
    }

    /// `Perform()`: with an infinite tolerance, the proximity value (unless done) whose
    /// value becomes the tolerance, with its points and statuses; otherwise the overlap
    /// test (unless done). Nothing before both shapes are loaded; `tool.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        if self.my_tolerance == precision::infinite() {
            if !self.my_is_init_s1 || !self.my_is_init_s2 || self.my_prox_val_tool.is_done() {
                return Ok(());
            }
            self.my_prox_val_tool
                .load_triangle_sets(&self.my_element_set1, &self.my_element_set2);
            self.my_prox_val_tool
                .load_shape_lists(&self.my_shape_list1, &self.my_shape_list2)?;
            self.my_prox_val_tool
                .set_nb_sample_points(self.my_nb_samples1, self.my_nb_samples2);
            self.my_prox_val_tool.perform(&mut self.my_tolerance)?;
            (self.my_prox_point1, self.my_prox_point2) = self.my_prox_val_tool.proximity_points();
            (self.my_prox_pnt_status1, self.my_prox_pnt_status2) =
                self.my_prox_val_tool.proximity_points_status();
            return Ok(());
        }
        if !self.my_is_init_s1 || !self.my_is_init_s2 || self.my_overlap_tool.is_done() {
            return Ok(());
        }
        self.my_overlap_tool
            .load_triangle_sets(&self.my_element_set1, &self.my_element_set2);
        self.my_overlap_tool.perform(self.my_tolerance)
    }

    /// `IsDone()`: either tool is done; `tool.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_overlap_tool.is_done() || self.my_prox_val_tool.is_done()
    }

    /// `OverlapSubShapes1()`: the overlapped faces of the second shape for each face of
    /// the first (indices from 0); `tool.overlap_sub_shapes1().extent()`.
    pub fn overlap_sub_shapes1(&self) -> &BRepExtremaOverlapMap {
        self.my_overlap_tool.overlap_sub_shapes1()
    }

    /// `OverlapSubShapes2()`; `tool.overlap_sub_shapes2().extent()`.
    pub fn overlap_sub_shapes2(&self) -> &BRepExtremaOverlapMap {
        self.my_overlap_tool.overlap_sub_shapes2()
    }

    /// The sub-shape `id` of a list; Standard_OutOfRange outside it (an unchecked read
    /// natively).
    fn sub_shape(list: &[Shape], id: i32, member: &str) -> Result<Shape, Failure> {
        usize::try_from(id)
            .ok()
            .and_then(|i| list.get(i))
            .cloned()
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "BRepExtrema_ShapeProximity::{member}: index {id} outside [0, {})",
                    list.len()
                ))
            })
    }

    /// `GetSubShape1(theID)`: the sub-shape of the first shape of index `id` (from 0);
    /// `tool.get_sub_shape1(0)?`.
    pub fn get_sub_shape1(&self, id: i32) -> Result<Shape, Failure> {
        Self::sub_shape(&self.my_shape_list1, id, "GetSubShape1")
    }

    /// `GetSubShape2(theID)`; `tool.get_sub_shape2(0)?`.
    pub fn get_sub_shape2(&self, id: i32) -> Result<Shape, Failure> {
        Self::sub_shape(&self.my_shape_list2, id, "GetSubShape2")
    }

    /// `ElementSet1()`: the mesh elements of the first shape; `tool.element_set1()`.
    pub fn element_set1(&self) -> &Handle<BRepExtremaTriangleSet> {
        &self.my_element_set1
    }

    /// `ElementSet2()`; `tool.element_set2()`.
    pub fn element_set2(&self) -> &Handle<BRepExtremaTriangleSet> {
        &self.my_element_set2
    }

    /// `ProximityPoint1()`: the point of the proximity value on the first shape;
    /// `tool.proximity_point1()`.
    pub fn proximity_point1(&self) -> &Point3 {
        &self.my_prox_point1
    }

    /// `ProximityPoint2()`; `tool.proximity_point2()`.
    pub fn proximity_point2(&self) -> &Point3 {
        &self.my_prox_point2
    }

    /// `ProxPntStatus1()`: the status of the first point; `tool.prox_pnt_status1()`.
    pub fn prox_pnt_status1(&self) -> BRepExtremaProxPntStatus {
        self.my_prox_pnt_status1
    }

    /// `ProxPntStatus2()`; `tool.prox_pnt_status2()`.
    pub fn prox_pnt_status2(&self) -> BRepExtremaProxPntStatus {
        self.my_prox_pnt_status2
    }
}
