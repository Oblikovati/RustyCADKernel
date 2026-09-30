use super::*;
use crate::{
    brep_mesh::{GeomTool, IntersectionFlag},
    imesh_data::MEMORY_BLOCK_SIZE_HUGE,
};
use rustycad_tkernel::ncollection::{DynamicArray, IncrementalAllocatorObject, TreeSelector};
use rustycad_tkmath::{
    bnd::BoundingBox2,
    gp::{Point2, Vector2, Xy, resolution},
};
pub(super) struct SegmentSelector {
    segments: Handle<FaceSegments>,
    current: FaceSegment,
    self_index: i32,
    bounds: BoundingBox2,
    max_loop_size: f64,
    pub indices: DynamicArray<i32>,
    pub failure: Option<Failure>,
}
impl SegmentSelector {
    pub fn new(tolerance: f64) -> Self {
        let allocator = IncrementalAllocatorObject::new(MEMORY_BLOCK_SIZE_HUGE);
        Self {
            segments: Handle::default(),
            current: FaceSegment::default(),
            self_index: -1,
            bounds: BoundingBox2::default(),
            max_loop_size: std::f64::consts::PI * tolerance * tolerance,
            indices: DynamicArray::with_allocator(256, Some(allocator.resource().clone())),
            failure: None,
        }
    }
    pub fn set_segments(&mut self, segments: &Handle<FaceSegments>) {
        self.segments = segments.clone();
    }
    pub fn reset(&mut self, segment: &FaceSegment, self_index: i32) -> Result<(), Failure> {
        self.indices.clear(false);
        self.self_index = self_index;
        self.current = segment.clone();
        self.bounds = segment.bounds()?;
        Ok(())
    }
    fn accepts(&mut self, index: i32) -> Result<bool, Failure> {
        let other = self
            .segments
            .read()
            .value(index)
            .map_err(array_failure)?
            .clone();
        let (first, last) = self.current.endpoints()?;
        let (start, end) = other.endpoints()?;
        let Some(intersection) = segment_crossing([first, last, start, end]) else {
            return Ok(false);
        };
        let angle = Vector2::between(first, last).angle(Vector2::between(start, end))?;
        if angle.abs() < 5.0 * std::f64::consts::PI / 180.0 {
            return Ok(false);
        }
        if self.self_index != -1 && self.loop_area(index, intersection)?.abs() < self.max_loop_size
        {
            return Ok(false);
        }
        self.indices.append(index);
        Ok(true)
    }
    fn loop_area(&self, index: i32, intersection: Point2) -> Result<f64, Failure> {
        let mut previous = Xy::default();
        let mut sum = 0.0;
        for current_index in self.self_index..index {
            let segment = self
                .segments
                .read()
                .value(current_index)
                .map_err(array_failure)?
                .clone();
            let last = segment.second_point()?;
            let current = last.xy() - intersection.xy();
            if current.square_modulus() < resolution() {
                continue;
            }
            if previous.square_modulus() > resolution() {
                sum += previous.crossed(current);
            }
            previous = current;
        }
        Ok(sum / 2.0)
    }
}
impl TreeSelector<i32, BoundingBox2> for SegmentSelector {
    fn reject(&self, bounds: &BoundingBox2) -> bool {
        self.bounds.is_out_box(bounds)
    }
    fn accept(&mut self, index: &i32) -> bool {
        match self.accepts(*index) {
            Ok(accepted) => accepted,
            Err(error) => {
                self.failure = Some(error);
                false
            }
        }
    }
    fn stop(&self) -> bool {
        self.failure.is_some()
    }
}

fn segment_crossing(points: [Point2; 4]) -> Option<Point2> {
    let mut intersection = Point2::default();
    let flag = GeomTool::int_seg_seg(
        points[0].xy(),
        points[1].xy(),
        points[2].xy(),
        points[3].xy(),
        false,
        false,
        &mut intersection,
    );
    (flag == IntersectionFlag::Cross).then_some(intersection)
}
