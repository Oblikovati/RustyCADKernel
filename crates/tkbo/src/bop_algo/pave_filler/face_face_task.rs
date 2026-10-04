//! Private BOPAlgo_FaceFace wrapper used by the PaveFiller task phase.
use super::super::{AlertIntersectionFailed, Options, ParallelAlgo, ParallelAlgoBase};
use crate::int_tools::{Context, FaceFace};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::{
    message::{AlertHandle, ProgressScope},
    standard::{Failure, Handle},
};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
use std::sync::Arc;

/// Face-face task state from BOPAlgo_PaveFiller_6.cxx; `task.perform_parallel()?`.
pub(crate) struct BOPAlgoFaceFaceTask<'a> {
    parallel: ParallelAlgoBase<'a>,
    intersector: FaceFace,
    indices: [i32; 2],
    faces: [Face; 2],
    boxes: [BoundingBox3; 2],
    tolerance: f64,
    transform: Transform3,
}

impl Default for BOPAlgoFaceFaceTask<'_> {
    fn default() -> Self {
        Self {
            parallel: ParallelAlgoBase::default(),
            intersector: FaceFace::default(),
            indices: [-1; 2],
            faces: std::array::from_fn(|_| Face::default()),
            boxes: std::array::from_fn(|_| BoundingBox3::default()),
            tolerance: 1e-7,
            transform: Transform3::default(),
        }
    }
}

impl<'a> BOPAlgoFaceFaceTask<'a> {
    pub(crate) fn set_run_parallel(&mut self, enabled: bool) {
        self.parallel.options_mut().set_run_parallel(enabled);
    }

    pub(crate) fn set_indices(&mut self, first: i32, second: i32) {
        self.indices = [first, second];
    }

    pub(crate) fn indices(&self) -> [i32; 2] {
        self.indices
    }

    pub(crate) fn set_faces(&mut self, first: &Face, second: &Face) {
        self.faces = [first.clone(), second.clone()];
    }

    pub(crate) fn set_boxes(&mut self, first: &BoundingBox3, second: &BoundingBox3) {
        self.boxes = [first.clone(), second.clone()];
    }

    pub(crate) fn set_tol_ff(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }

    pub(crate) fn tol_ff(&self) -> f64 {
        self.tolerance
    }

    #[cfg(test)]
    pub(crate) fn trsf(&self) -> &Transform3 {
        &self.transform
    }

    pub(crate) fn set_parameters(&mut self, spatial: bool, first: bool, second: bool, tol: f64) {
        self.intersector.set_parameters(spatial, first, second, tol);
    }

    pub(crate) fn set_fuzzy_value(&mut self, fuzzy: f64) {
        self.intersector.set_fuzzy_value(fuzzy);
    }

    pub(crate) fn set_context(&mut self, context: &Handle<Context>) {
        self.intersector.set_context(context);
    }

    pub(crate) fn face1(&self) -> &Face {
        &self.faces[0]
    }

    pub(crate) fn face2(&self) -> &Face {
        &self.faces[1]
    }

    pub(crate) fn intersector(&self) -> &FaceFace {
        &self.intersector
    }

    pub(crate) fn intersector_mut(&mut self) -> &mut FaceFace {
        &mut self.intersector
    }

    pub(crate) fn apply_trsf(&mut self) -> Result<(), Failure> {
        self.intersector.apply_result_transform(self.transform)
    }

    fn execute_face_intersection(&mut self) {
        match transform_boxes_to_origin(&self.boxes) {
            Ok(Some(transform)) => self.perform_on_translated_faces(transform),
            Ok(None) => self.perform_on_input_faces(),
            Err(_) => self.parallel.options().add_error(&failure_alert()),
        }
    }

    fn perform_on_input_faces(&mut self) {
        if self
            .intersector
            .perform(&self.faces[0], &self.faces[1], self.run_parallel())
            .is_err()
        {
            self.parallel.options().add_error(&failure_alert());
        }
    }

    fn perform_on_translated_faces(&mut self, transform: Transform3) {
        let location = Location::new(transform);
        let mut first = self.faces[0].clone();
        let mut second = self.faces[1].clone();
        if first.move_by(&location, false).is_err() || second.move_by(&location, false).is_err() {
            self.parallel.options().add_error(&failure_alert());
            return;
        };
        let Ok(inverse) = transform.inverted() else {
            self.parallel.options().add_error(&failure_alert());
            return;
        };
        self.transform = inverse;
        if self
            .intersector
            .perform(&first, &second, self.run_parallel())
            .is_err()
        {
            self.parallel.options().add_error(&failure_alert());
        }
    }

    fn run_parallel(&self) -> bool {
        self.parallel_options().run_parallel()
    }
}

impl<'a> ParallelAlgo<'a> for BOPAlgoFaceFaceTask<'a> {
    fn parallel_options(&self) -> &Options {
        self.parallel.options()
    }

    fn parallel_base(&self) -> &ParallelAlgoBase<'a> {
        &self.parallel
    }

    fn parallel_base_mut(&mut self) -> &mut ParallelAlgoBase<'a> {
        &mut self.parallel
    }

    fn perform_parallel(&mut self) -> Result<(), Failure> {
        let scope = ProgressScope::unnamed(self.parallel.progress_range(), 1.0, false);
        if self.parallel.options().user_break(&scope) {
            return Ok(());
        }
        self.execute_face_intersection();
        Ok(())
    }

    fn fill_parallel_constants(&self, _whole: f64, _steps: &mut super::super::PISteps) {}

    fn fill_parallel_steps(&self, _steps: &mut super::super::PISteps) {}
}

fn transform_boxes_to_origin(boxes: &[BoundingBox3; 2]) -> Result<Option<Transform3>, Failure> {
    let mut union = boxes[0].clone();
    union.add_box(&boxes[1]);
    let minimum = union.corner_min()?;
    let maximum = union.corner_max()?;
    let center = midpoint(minimum, maximum);
    let origin = Point3::default();
    let distance = center.distance(&origin);
    if distance < 1e5 || union.square_extent().sqrt() / distance > 1e-5 {
        return Ok(None);
    }
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::between(minimum, origin));
    Ok(Some(transform))
}

fn midpoint(first: Point3, second: Point3) -> Point3 {
    Point3::new(
        (first.x() + second.x()) * 0.5,
        (first.y() + second.y()) * 0.5,
        (first.z() + second.z()) * 0.5,
    )
}

fn failure_alert() -> AlertHandle {
    Arc::new(AlertIntersectionFailed) as AlertHandle
}
