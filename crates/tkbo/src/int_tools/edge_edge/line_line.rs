//! Analytic edge intersection preserves OCCT 3d097a0 overlap clipping and shared-vertex exclusion.
use crate::int_tools::{CommonPrt, Range, Tools};
use rustycad_tkbrep::topods::{Edge, ShapeIterator};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    elclib,
    gp::{Line3, Point3, Vector3},
};
pub(super) struct LinePair<'a> {
    pub edges: (&'a Edge, &'a Edge),
    pub lines: (Line3, Line3),
    pub ranges: (Range, Range),
    pub tolerances: (f64, f64),
    pub tolerance: f64,
}
impl LinePair<'_> {
    #[allow(
        clippy::map_clone,
        reason = "OCCT sequence insertion invokes a nontrivial copy constructor"
    )]
    pub(super) fn compute(&self) -> Result<Option<CommonPrt>, Failure> {
        let angle = self.lines.0.direction().angle(&self.lines.1.direction());
        let parallel = angle < precision::angular();
        let squared = self.tolerance * self.tolerance;
        if parallel && self.lines.0.square_distance(self.lines.1.location()) > squared {
            return Ok(None);
        }
        let endpoints = (
            elclib::line_value(self.ranges.0.first(), self.lines.0.position()),
            elclib::line_value(self.ranges.0.last(), self.lines.0.position()),
        );
        let (coincident, same_side) = if parallel {
            (true, false)
        } else {
            self.endpoint_sides(endpoints, squared)
        };
        if !coincident && same_side {
            return Ok(None);
        }
        let part = if coincident {
            self.overlap(endpoints)
        } else {
            self.crossing(angle)?
        };
        // NCollection_Sequence::Append copy-constructs CommonPrt, resetting AllNullFlag.
        Ok(part.map(|value| value.clone()))
    }
    fn endpoint_sides(&self, endpoints: (Point3, Point3), squared: f64) -> (bool, bool) {
        let mut origin = self.lines.1.location();
        let range = self.ranges.1;
        if !precision::is_infinite(range.first()) && !precision::is_infinite(range.last()) {
            origin =
                elclib::line_value((range.first() + range.last()) / 2., self.lines.1.position());
        }
        let direction = Vector3::from(self.lines.1.direction());
        let first = Vector3::between(origin, endpoints.0).crossed(direction);
        let last = Vector3::between(origin, endpoints.1).crossed(direction);
        (
            first.square_magnitude() <= squared && last.square_magnitude() <= squared,
            first.dot(last) > 0.,
        )
    }
    fn common_part(&self, kind: ShapeEnum) -> CommonPrt {
        let mut part = CommonPrt::default();
        part.set_edge1(self.edges.0);
        part.set_edge2(self.edges.1);
        part.set_type(kind);
        part
    }
    fn overlap(&self, endpoints: (Point3, Point3)) -> Option<CommonPrt> {
        let mut first = elclib::line_parameter(self.lines.1.position(), endpoints.0);
        let mut last = elclib::line_parameter(self.lines.1.position(), endpoints.1);
        let (lower, upper) = self.ranges.1.range();
        if (first > upper && last > upper) || (first < lower && last < lower) {
            return None;
        }
        if first > last {
            std::mem::swap(&mut first, &mut last);
        }
        let mut part = self.common_part(ShapeEnum::Edge);
        self.overlap_ranges(&mut part, first, last);
        Some(part)
    }
    fn overlap_ranges(&self, part: &mut CommonPrt, first: f64, last: f64) {
        let (lower, upper) = self.ranges.1.range();
        if first < lower {
            part.set_range1_bounds(
                self.ranges.0.first() + (lower - first),
                self.ranges.0.last(),
            );
            part.append_range2_bounds(lower, last);
            return;
        }
        if last <= upper {
            part.set_range1(self.ranges.0);
            part.set_all_null_flag(true);
            part.append_range2_bounds(first, last);
        } else {
            part.set_range1_bounds(self.ranges.0.first(), self.ranges.0.last() - (last - upper));
            part.append_range2_bounds(first, upper);
        }
    }
    fn crossing(&self, angle: f64) -> Result<Option<CommonPrt>, Failure> {
        let displacement = Vector3::between(self.lines.0.location(), self.lines.1.location());
        let first_direction = self.lines.0.direction().xyz();
        let second_direction = self.lines.1.direction().xyz();
        let cross = first_direction.crossed(second_direction);
        let distance = displacement.dot(Vector3::from(cross.normalized()?));
        if distance.abs() > self.tolerance || self.shared_vertex()? {
            return Ok(None);
        }
        let second = (first_direction * displacement.dot(Vector3::from(first_direction))
            - *displacement.xyz())
        .dot(second_direction)
            / cross.square_modulus();
        if outside(second, self.ranges.1) {
            return Ok(None);
        }
        let point = elclib::line_value(second, self.lines.1.position());
        let first =
            Vector3::between(self.lines.0.location(), point).dot(Vector3::from(first_direction));
        if outside(first, self.ranges.0) {
            return Ok(None);
        }
        let other = elclib::line_value(first, self.lines.0.position());
        if other.square_distance(&point) > self.tolerance * self.tolerance {
            return Ok(None);
        }
        Ok(Some(self.vertex_part(first, second, angle)))
    }
    fn vertex_part(&self, first: f64, second: f64, angle: f64) -> CommonPrt {
        let mut part = self.common_part(ShapeEnum::Vertex);
        let first_step = Tools::compute_int_range(self.tolerances.0, self.tolerances.1, angle);
        let second_step = Tools::compute_int_range(self.tolerances.1, self.tolerances.0, angle);
        part.set_range1_bounds(first - first_step, first + first_step);
        part.append_range2_bounds(second - second_step, second + second_step);
        part.set_vertex_parameter1(first);
        part.set_vertex_parameter2(second);
        part
    }
    fn shared_vertex(&self) -> Result<bool, Failure> {
        let mut first = ShapeIterator::new(self.edges.0, true, true);
        while first.more() {
            let mut second = ShapeIterator::new(self.edges.1, true, true);
            while second.more() {
                if first.value()?.is_same(second.value()?) {
                    return Ok(true);
                }
                second.next();
            }
            first.next();
        }
        Ok(false)
    }
}
fn outside(parameter: f64, range: Range) -> bool {
    parameter < range.first() || parameter > range.last()
}
