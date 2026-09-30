// IntTools_CommonPrt.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Common edge portions, preserving the distinct native copy operations.
use super::Range;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
/// `IntTools_CommonPrt`; `CommonPrt::default().ranges2().is_empty()`.
#[derive(Debug)]
pub struct CommonPrt {
    edge1: Edge,
    edge2: Edge,
    shape_type: ShapeEnum,
    range1: Range,
    vertex_parameter1: f64,
    vertex_parameter2: f64,
    ranges2: Vec<Range>,
    all_null_flag: bool,
    bounding_points: (Point3, Point3),
}
impl Default for CommonPrt {
    fn default() -> Self {
        Self {
            edge1: Edge::default(),
            edge2: Edge::default(),
            shape_type: ShapeEnum::Shape,
            range1: Range::default(),
            vertex_parameter1: 0.0,
            vertex_parameter2: 0.0,
            ranges2: Vec::new(),
            all_null_flag: false,
            bounding_points: (Point3::default(), Point3::default()),
        }
    }
}
impl Clone for CommonPrt {
    fn clone(&self) -> Self {
        let mut result = Self::default();
        self.copy_to(&mut result);
        result.bounding_points = self.bounding_points;
        // Native copy construction resets AllNullFlag, unlike Assign (3d097a0).
        result
    }
}
impl CommonPrt {
    /// Copies fields and appends ranges, retaining native assignment semantics; `part.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        other.copy_to(self);
        self.bounding_points = other.bounding_points;
        self.all_null_flag = other.all_null_flag;
        self
    }
    /// Copies core fields and appends ranges, leaving target flag/points untouched; `part.copy_to(&mut other)`.
    pub fn copy_to(&self, target: &mut Self) {
        target.edge1 = self.edge1.clone();
        target.edge2 = self.edge2.clone();
        target.shape_type = self.shape_type;
        target.range1 = self.range1;
        target.vertex_parameter1 = self.vertex_parameter1;
        target.vertex_parameter2 = self.vertex_parameter2;
        target.ranges2.extend_from_slice(&self.ranges2);
    }
    /// Replaces edge1; `part.set_edge1(&edge)`.
    pub fn set_edge1(&mut self, value: &Edge) {
        self.edge1 = value.clone();
    }
    /// Reads edge1; `part.edge1()`.
    pub fn edge1(&self) -> &Edge {
        &self.edge1
    }
    /// Replaces edge2; `part.set_edge2(&edge)`.
    pub fn set_edge2(&mut self, value: &Edge) {
        self.edge2 = value.clone();
    }
    /// Reads edge2; `part.edge2()`.
    pub fn edge2(&self) -> &Edge {
        &self.edge2
    }
    /// Replaces shape_type; `part.set_type(ShapeEnum::Edge)`.
    pub fn set_type(&mut self, value: ShapeEnum) {
        self.shape_type = value;
    }
    /// Reads shape_type; `part.shape_type()`.
    pub fn shape_type(&self) -> ShapeEnum {
        self.shape_type
    }
    /// Replaces vertex_parameter1; `part.set_vertex_parameter1(0.5)`.
    pub fn set_vertex_parameter1(&mut self, value: f64) {
        self.vertex_parameter1 = value;
    }
    /// Reads vertex_parameter1; `part.vertex_parameter1()`.
    pub fn vertex_parameter1(&self) -> f64 {
        self.vertex_parameter1
    }
    /// Replaces vertex_parameter2; `part.set_vertex_parameter2(0.75)`.
    pub fn set_vertex_parameter2(&mut self, value: f64) {
        self.vertex_parameter2 = value;
    }
    /// Reads vertex_parameter2; `part.vertex_parameter2()`.
    pub fn vertex_parameter2(&self) -> f64 {
        self.vertex_parameter2
    }
    /// Replaces all_null_flag; `part.set_all_null_flag(true)`.
    pub fn set_all_null_flag(&mut self, value: bool) {
        self.all_null_flag = value;
    }
    /// Reads all_null_flag; `part.all_null_flag()`.
    pub fn all_null_flag(&self) -> bool {
        self.all_null_flag
    }
    /// Replaces the first edge's interval; `part.set_range1(Range::new(0.0, 1.0))`.
    pub fn set_range1(&mut self, range: Range) {
        self.range1 = range;
    }
    /// Replaces the first interval endpoints; `part.set_range1_bounds(0.0, 1.0)`.
    pub fn set_range1_bounds(&mut self, first: f64, last: f64) {
        self.range1 = Range::new(first, last);
    }
    /// Borrows the first interval; `part.range1()`.
    pub fn range1(&self) -> &Range {
        &self.range1
    }
    /// Reads the first interval endpoints; `part.range1_bounds()`.
    pub fn range1_bounds(&self) -> (f64, f64) {
        self.range1.range()
    }
    /// Appends a second-edge interval; `part.append_range2(Range::new(0.0, 1.0))`.
    pub fn append_range2(&mut self, range: Range) {
        self.ranges2.push(range);
    }
    /// Appends second-edge endpoints; `part.append_range2_bounds(0.0, 1.0)`.
    pub fn append_range2_bounds(&mut self, first: f64, last: f64) {
        self.ranges2.push(Range::new(first, last));
    }
    /// Borrows all second-edge intervals; `part.ranges2()`.
    pub fn ranges2(&self) -> &[Range] {
        &self.ranges2
    }
    /// Mutates the second-edge intervals; `part.change_ranges2().clear()`.
    pub fn change_ranges2(&mut self) -> &mut Vec<Range> {
        &mut self.ranges2
    }
    /// Replaces both bounding points; `part.set_bounding_points(p, q)`.
    pub fn set_bounding_points(&mut self, first: Point3, last: Point3) {
        self.bounding_points = (first, last);
    }
    /// Reads both bounding points; `part.bounding_points()`.
    pub fn bounding_points(&self) -> (Point3, Point3) {
        self.bounding_points
    }
}
