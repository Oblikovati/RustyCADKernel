//! Applies BOPAlgo_FaceFace's precision translation to only spatial result points/curves.
use super::super::PntOnFace;
use super::*;
use rustycad_tkmath::gp::Transform3;

impl FaceFace {
    /// Reverses the wrapper's temporary precision translation after a successful intersector run.
    pub(crate) fn apply_result_transform(&mut self, transform: Transform3) -> Result<(), Failure> {
        if !self.done {
            return Ok(());
        }
        self.transform_spatial_curves(transform)?;
        self.transform_intersection_points(transform);
        Ok(())
    }

    fn transform_spatial_curves(&self, transform: Transform3) -> Result<(), Failure> {
        for curve in &self.curves {
            curve
                .curve()
                .get()
                .expect("completed curve has geometry")
                .transform(transform)?;
        }
        Ok(())
    }

    fn transform_intersection_points(&mut self, transform: Transform3) {
        for pair in &mut self.points {
            let first = transformed_point(pair.p1(), transform);
            let second = transformed_point(pair.p2(), transform);
            pair.set_p1(&first);
            pair.set_p2(&second);
        }
    }
}

fn transformed_point(point: &PntOnFace, transform: Transform3) -> PntOnFace {
    let mut result = point.clone();
    result.set_pnt(point.pnt().transformed(transform));
    result
}
