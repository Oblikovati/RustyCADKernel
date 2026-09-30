//! Shared scalar equation from Contap_ArcFunction and Contap_SurfFunction.
use super::{ContapTFunction, geometry::vector_of_dir};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
pub(super) struct ContourEquation {
    pub kind: ContapTFunction,
    pub direction: Direction3,
    pub eye: Point3,
    pub cosine: f64,
    pub scale: f64,
}
impl ContourEquation {
    pub fn normal_value(&self, point: Point3, normal: Vector3) -> Option<f64> {
        let direction = vector_of_dir(self.direction);
        match self.kind {
            ContapTFunction::ContourStd => Some(normal.dot(direction) / self.scale),
            ContapTFunction::ContourPrs => {
                Some(normal.dot(Vector3::between(self.eye, point)) / self.scale)
            }
            ContapTFunction::DraftStd => {
                Some((normal.dot(direction) - self.cosine * normal.magnitude()) / self.scale)
            }
            ContapTFunction::DraftPrs => None,
        }
    }
    pub fn normal_gradient(
        &self,
        point: Point3,
        normal: Vector3,
        du: Vector3,
        dv: Vector3,
    ) -> Result<Option<(f64, f64)>, Failure> {
        let direction = vector_of_dir(self.direction);
        Ok(match self.kind {
            ContapTFunction::ContourStd => Some((
                du.dot(direction) / self.scale,
                dv.dot(direction) / self.scale,
            )),
            ContapTFunction::ContourPrs => {
                let eye_point = Vector3::between(self.eye, point);
                Some((
                    du.dot(eye_point) / self.scale,
                    dv.dot(eye_point) / self.scale,
                ))
            }
            ContapTFunction::DraftStd => Some(self.draft_gradient(normal, du, dv)?),
            ContapTFunction::DraftPrs => None,
        })
    }
    fn draft_gradient(
        &self,
        mut normal: Vector3,
        du: Vector3,
        dv: Vector3,
    ) -> Result<(f64, f64), Failure> {
        normal.normalize()?;
        let direction = vector_of_dir(self.direction);
        Ok((
            (du.dot(direction) - self.cosine * du.dot(normal)) / self.scale,
            (dv.dot(direction) - self.cosine * dv.dot(normal)) / self.scale,
        ))
    }
}
