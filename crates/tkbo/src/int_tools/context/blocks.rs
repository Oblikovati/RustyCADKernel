use super::*;
use crate::int_tools::{Curve, Tools};
impl Context {
    /// Tests the asymmetric interior point against the face; `context.is_valid_block_for_face(0., 1., &curve, &face, 1e-7)?`.
    pub fn is_valid_block_for_face(
        &self,
        first: f64,
        last: f64,
        curve: &Curve,
        face: &Face,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let parameter = Tools::intermediate_point(first, last);
        let point = curve.curve().eval_d0(parameter)?;
        self.is_valid_point_for_face(point, face, tolerance)
    }
    /// Uses each available pcurve before spatial projection; `context.is_valid_block_for_faces(0., 1., &curve, &f1, &f2, 1e-7)?`.
    pub fn is_valid_block_for_faces(
        &self,
        first: f64,
        last: f64,
        curve: &Curve,
        f1: &Face,
        f2: &Face,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let parameter = Tools::intermediate_point(first, last);
        let point = curve.curve().eval_d0(parameter)?;
        for (pcurve, face) in [(curve.first_curve2d(), f1), (curve.second_curve2d(), f2)] {
            let valid = if pcurve.is_null() {
                self.is_valid_point_for_face(point, face, tolerance)?
            } else {
                self.is_point_in_on_face(face, pcurve.eval_d0(parameter)?)?
            };
            if !valid {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
