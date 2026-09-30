use super::*;
use rustycad_tkmath::gp::{Xy, resolution};
/// Native intersection discriminants; `IntersectionFlag::Cross as i32 == 1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum IntersectionFlag {
    /// Disjoint or excluded contact.
    NoIntersection,
    /// Interior crossing.
    Cross,
    /// Shared endpoint.
    EndPointTouch,
    /// One endpoint lies on the other segment.
    PointOnSegment,
    /// Collinear overlap.
    Glued,
    /// Coincident supporting lines.
    Same,
}
impl GeomTool<'_> {
    /// Intersect unbounded supporting lines; outputs remain untouched for parallel lines; `GeomTool::int_lin_lin(a,b,c,d,&mut point,&mut parameters)`.
    pub fn int_lin_lin(
        start1: Xy,
        end1: Xy,
        start2: Xy,
        end2: Xy,
        point: &mut Xy,
        parameters: &mut [f64; 2],
    ) -> IntersectionFlag {
        let first = end1 - start1;
        let second = end2 - start2;
        let offset = start2 - start1;
        let denominator = first.crossed(second);
        let numerator = offset.crossed(second);
        if denominator.abs() < resolution() {
            return if numerator.abs() < resolution() {
                IntersectionFlag::Same
            } else {
                IntersectionFlag::NoIntersection
            };
        }
        parameters[0] = numerator / denominator;
        *point = start1 + first * parameters[0];
        parameters[1] = offset.crossed(first) / denominator;
        IntersectionFlag::Cross
    }
    /// Classify bounded segments with native endpoint rules; `GeomTool::int_seg_seg(a,b,c,d,false,false,&mut point)`.
    pub fn int_seg_seg(
        start1: Xy,
        end1: Xy,
        start2: Xy,
        end2: Xy,
        end_touch: bool,
        point_on_segment: bool,
        point: &mut Point2,
    ) -> IntersectionFlag {
        let points = [start1, end1, start2, end2];
        let hash = [
            classify(start1, end1, start2),
            classify(start1, end1, end2),
            classify(start2, end2, start1),
            classify(start2, end2, end1),
        ];
        let sum = hash.iter().sum();
        if let Some(flag) = classify_contact(points, hash, sum, end_touch, point_on_segment, point)
        {
            return flag;
        }
        line_segment_intersection(points, sum, point)
    }
}
fn classify_contact(
    points: [Xy; 4],
    hash: [i32; 4],
    sum: i32,
    end_touch: bool,
    point_on_segment: bool,
    point: &mut Point2,
) -> Option<IntersectionFlag> {
    if hash[0] < 0 || hash[1] < 0 {
        return Some(shared_endpoint_flag(sum, end_touch));
    }
    if sum == 1 {
        if !point_on_segment {
            return Some(IntersectionFlag::NoIntersection);
        }
        // Native selects the corresponding endpoint of the opposite segment (OCC preserved).
        let index = hash.iter().position(|value| *value == 1).unwrap_or(3);
        *point = Point2::new(points[index].x(), points[index].y());
        return Some(IntersectionFlag::PointOnSegment);
    }
    if sum == 2 {
        return Some(IntersectionFlag::Glued);
    }
    None
}
fn shared_endpoint_flag(sum: i32, end_touch: bool) -> IntersectionFlag {
    if sum == -1 {
        return IntersectionFlag::Glued;
    }
    if end_touch {
        return IntersectionFlag::EndPointTouch;
    }
    IntersectionFlag::NoIntersection
}
fn line_segment_intersection(points: [Xy; 4], sum: i32, point: &mut Point2) -> IntersectionFlag {
    let mut parameters = [0.0; 2];
    let mut xy = point.xy();
    let flag = GeomTool::int_lin_lin(
        points[0],
        points[1],
        points[2],
        points[3],
        &mut xy,
        &mut parameters,
    );
    *point = Point2::new(xy.x(), xy.y());
    bounded_line_flag(flag, sum, parameters)
}
fn bounded_line_flag(flag: IntersectionFlag, sum: i32, parameters: [f64; 2]) -> IntersectionFlag {
    if flag == IntersectionFlag::NoIntersection {
        return flag;
    }
    if flag == IntersectionFlag::Same {
        return coincident_flag(sum);
    }
    let precision = precision::p_confusion();
    if parameters
        .into_iter()
        .any(|parameter| parameter < precision || parameter > 1.0 - precision)
    {
        return IntersectionFlag::NoIntersection;
    }
    IntersectionFlag::Cross
}
fn coincident_flag(sum: i32) -> IntersectionFlag {
    if sum < -2 {
        return IntersectionFlag::Same;
    }
    if sum == -1 {
        return IntersectionFlag::Glued;
    }
    IntersectionFlag::NoIntersection
}
fn classify(first: Xy, last: Xy, point: Xy) -> i32 {
    let segment = last - first;
    let offset = point - first;
    let distance = segment.crossed(offset).abs();
    let tolerance = precision::p_confusion();
    if distance > tolerance
        && distance * distance / segment.square_modulus() > tolerance * tolerance
    {
        return 0;
    }
    let product = segment.multiplied_components(offset);
    if product.x() < 0.0 || product.y() < 0.0 || segment.square_modulus() < offset.square_modulus()
    {
        return 0;
    }
    if point.is_equal(&first, tolerance) || point.is_equal(&last, tolerance) {
        return -1;
    }
    1
}
