//! Native BoxBndTree selection: traversal order and tail preference are observable.
use super::*;
use rustycad_tkernel::ncollection::TreeSelector;
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
pub(super) struct BoundsSelector<'a> {
    pub wires: &'a mut [Shape],
    pub consumed: Vec<bool>,
    pub selected: usize,
    pub status: i32,
    shared: bool,
    stop: bool,
    ends: [Vertex; 2],
    points: [Point3; 2],
    boxes: [BoundingBox3; 2],
    indices: [usize; 2],
    tolerance: f64,
    minimum: f64,
    pub error: Option<Failure>,
}
impl<'a> BoundsSelector<'a> {
    pub fn new(wires: &'a mut [Shape], shared: bool) -> Self {
        Self {
            consumed: vec![false; wires.len()],
            wires,
            selected: 0,
            status: 0,
            shared,
            stop: false,
            ends: Default::default(),
            points: Default::default(),
            boxes: Default::default(),
            indices: [0; 2],
            tolerance: 1e-7,
            minimum: 1e-7,
            error: None,
        }
    }
    pub fn reset(&mut self, ends: [Vertex; 2], tolerance: f64) -> Result<(), Failure> {
        self.stop = false;
        self.indices = [0; 2];
        self.points = [BRepTool::pnt(&ends[0])?, BRepTool::pnt(&ends[1])?];
        self.boxes = self.points.map(|point| endpoint_box(point, tolerance));
        self.ends = ends;
        self.status = 0;
        if !self.shared {
            self.tolerance = tolerance;
            self.minimum = tolerance;
        }
        Ok(())
    }
    fn accept_wire(&mut self, index: usize) -> Result<bool, Failure> {
        if self.consumed[index] {
            return Ok(false);
        }
        let (first, last) =
            rustycad_tkbrep::topexp::wire_vertices(&topods::wire(&self.wires[index])?);
        if self.shared {
            return Ok(self.accept_shared(index, &[first, last]));
        }
        let points = [BRepTool::pnt(&first)?, BRepTool::pnt(&last)?];
        Ok(self.accept_nearest(index, points))
    }
    fn accept_shared(&mut self, index: usize, ends: &[Vertex; 2]) -> bool {
        let pairs = [(1, 0), (1, 1), (0, 1), (0, 0)];
        let Some(kind) = pairs
            .iter()
            .position(|&(own, other)| self.ends[own].is_same(&ends[other]))
        else {
            self.status = -2;
            self.stop = false;
            return false;
        };
        self.status = kind as i32 + 1;
        self.indices[if kind < 2 { 1 } else { 0 }] = index + 1;
        self.selected = index;
        if self.indices[1] != 0 {
            self.stop = true;
        }
        true
    }
    fn endpoint_distances(&self, points: [Point3; 2]) -> [f64; 4] {
        [
            points[0].distance(&self.points[1]),
            points[1].distance(&self.points[1]),
            points[1].distance(&self.points[0]),
            points[0].distance(&self.points[0]),
        ]
    }
    fn accept_nearest(&mut self, index: usize, points: [Point3; 2]) -> bool {
        let distances = self.endpoint_distances(points);
        let tail = usize::from(distances[0] > distances[1]);
        let head = 2 + usize::from(distances[2] > distances[3]);
        let kind = if distances[tail] > distances[head] {
            head
        } else {
            tail
        };
        let minimum = if distances[head] < distances[tail] {
            distances[head]
        } else {
            distances[tail]
        };
        if minimum > self.minimum {
            return false;
        }
        self.accept_distance(index, kind, minimum)
    }
    fn accept_distance(&mut self, index: usize, kind: usize, minimum: f64) -> bool {
        self.indices[if kind < 2 { 1 } else { 0 }] = index + 1;
        // The upstream positive-difference condition cannot fire after the preceding rejection.
        self.minimum = minimum;
        if minimum > self.tolerance {
            self.status = -2;
            return false;
        }
        self.selected = if self.indices[1] != 0 {
            self.indices[1] - 1
        } else {
            self.indices[0] - 1
        };
        if minimum == 0. && kind < 2 {
            self.stop = true;
        }
        if self.selected == index {
            self.status = kind as i32 + 1;
        }
        true
    }
}
impl TreeSelector<usize, BoundingBox3> for BoundsSelector<'_> {
    fn reject(&self, bounds: &BoundingBox3) -> bool {
        self.boxes[0].is_out_box(bounds) && self.boxes[1].is_out_box(bounds)
    }
    fn accept(&mut self, index: &usize) -> bool {
        match self.accept_wire(*index) {
            Ok(accepted) => accepted,
            Err(error) => {
                self.error = Some(error);
                self.stop = true;
                false
            }
        }
    }
    fn stop(&self) -> bool {
        self.stop
    }
}
pub(super) fn endpoint_box(point: Point3, tolerance: f64) -> BoundingBox3 {
    let mut bounds = BoundingBox3::default();
    bounds.set_point(point);
    bounds.set_gap(tolerance);
    bounds
}
