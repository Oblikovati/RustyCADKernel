//! `Graphic3d_TransformPersScaledAbove`: a zoom persistence that stops shrinking the object
//! below a minimal scale.
use super::transform_pers::{Graphic3dTransformPersBase, Graphic3dTransformPersSource};
use super::{Graphic3dCamera, Graphic3dTransModeFlags, graphic3d_transform_pers_persistent_scale};
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkmath::gp::Point3;

/// Zoom persistence with a lower scale bound;
/// `Graphic3dTransformPersScaledAbove::new(2.0, &anchor)?`.
pub struct Graphic3dTransformPersScaledAbove {
    transient: StandardTransient,
    base: Graphic3dTransformPersBase,
    scale: f64,
}
impl Graphic3dTransformPersSource for Graphic3dTransformPersScaledAbove {
    fn transform_pers_base(&self) -> &Graphic3dTransformPersBase {
        &self.base
    }
    /// The base persistent scale, raised to the minimal scale.
    fn persistent_scale(&self, camera: &Graphic3dCamera, width: i32, height: i32) -> f64 {
        let scale = graphic3d_transform_pers_persistent_scale(&self.base, camera, width, height);
        if scale < self.scale {
            self.scale
        } else {
            scale
        }
    }
}
rustycad_tkernel::impl_transient!(Graphic3dTransformPersScaledAbove,
    "Graphic3d_TransformPersScaledAbove", dyn Graphic3dTransformPersSource, transient,
    interfaces: [dyn Graphic3dTransformPersSource]);

impl Graphic3dTransformPersScaledAbove {
    /// A zoom persistence anchored at `point` that keeps at least `scale`.
    /// Example: `Graphic3dTransformPersScaledAbove::new(2.0, &Point3::new(0.0, 0.0, 0.0))?`.
    pub fn new(scale: f64, point: &Point3) -> Result<Self, Failure> {
        let base =
            Graphic3dTransformPersBase::with_point(Graphic3dTransModeFlags::ZOOM_PERS, point)?;
        Ok(Self {
            transient: StandardTransient::default(),
            base,
            scale,
        })
    }
}
