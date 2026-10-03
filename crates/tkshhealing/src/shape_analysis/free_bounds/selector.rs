//! Adapt zero-based working wires to the public native endpoint selector.
use super::*;
use crate::shape_analysis::ShapeAnalysisBoxBndTreeSelector;
use rustycad_tkernel::ncollection::{FallibleTreeSelector, HArray1};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
pub(super) struct BoundsSelector<'a> {
    pub wires: &'a mut [Shape],
    pub consumed: Vec<bool>,
    pub selected: usize,
    pub status: i32,
    shared: bool,
    selector: ShapeAnalysisBoxBndTreeSelector,
}
impl<'a> BoundsSelector<'a> {
    pub fn new(wires: &'a mut [Shape], shared: bool) -> Result<Self, Failure> {
        let array = wire_array(wires)?;
        Ok(Self {
            consumed: vec![false; wires.len()],
            wires,
            selected: 0,
            status: 0,
            shared,
            selector: ShapeAnalysisBoxBndTreeSelector::new(&array, shared),
        })
    }
    pub fn reset(&mut self, ends: [Vertex; 2], tolerance: f64) -> Result<(), Failure> {
        self.selector.set_stop();
        let points = [BRepTool::pnt(&ends[0])?, BRepTool::pnt(&ends[1])?];
        let boxes = points.map(|point| endpoint_box(point, tolerance));
        self.selector.define_boxes(&boxes[0], &boxes[1]);
        self.selector.define_pnt(points[0], points[1]);
        self.selector
            .define_vertexes(ends[0].clone(), ends[1].clone());
        self.status = 0;
        if !self.shared {
            self.selector.set_tolerance(tolerance);
        }
        Ok(())
    }
    fn synchronize_selection(&mut self) {
        self.selected = self.selector.nb().saturating_sub(1) as usize;
        let flags = [
            ShapeExtendStatus::Done1,
            ShapeExtendStatus::Done2,
            ShapeExtendStatus::Done3,
            ShapeExtendStatus::Done4,
        ];
        self.status = flags
            .iter()
            .position(|flag| self.selector.last_check_status(*flag))
            .map_or_else(
                || {
                    if self.selector.last_check_status(ShapeExtendStatus::Fail2) {
                        -2
                    } else {
                        0
                    }
                },
                |index| index as i32 + 1,
            );
    }
}
impl FallibleTreeSelector<usize, BoundingBox3> for BoundsSelector<'_> {
    type Error = Failure;
    fn reject(&self, bounds: &BoundingBox3) -> bool {
        self.selector.reject(bounds)
    }
    fn accept(&mut self, index: &usize) -> Result<bool, Failure> {
        if self.consumed[*index] {
            return Ok(false);
        }
        let accepted = self.selector.accept((*index + 1) as i32)?;
        self.synchronize_selection();
        Ok(accepted)
    }
    fn stop(&self) -> bool {
        self.selector.stop()
    }
}
fn wire_array(wires: &[Shape]) -> Result<Handle<HArray1<Shape>>, Failure> {
    let count = i32::try_from(wires.len()).map_err(|_| wire_count_failure(wires.len()))?;
    let array =
        HArray1::<Shape>::with_bounds(1, count).map_err(|_| wire_count_failure(wires.len()))?;
    // Slots are reversed only after consumption; shared TShapes preserve all live endpoint changes.
    array.change_array1().as_mut_slice().clone_from_slice(wires);
    Ok(Handle::new(array))
}
fn wire_count_failure(count: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("wire count {count}; expected 1..=i32::MAX"),
        "",
    )
}
pub(super) fn endpoint_box(point: Point3, tolerance: f64) -> BoundingBox3 {
    let mut bounds = BoundingBox3::default();
    bounds.set_point(point);
    bounds.set_gap(tolerance);
    bounds
}
