// Rust translation of BRepMesh_CylinderRangeSplitter.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use rustycad_tkgeombase::gcpnts::TangentialDeflection;
use rustycad_tkmath::bvh::BvhScalar;

/// `BRepMesh_CylinderRangeSplitter`: interior nodes along the U rows of a cylinder, with
/// the U delta scaled by the arc angular step; `CylinderRangeSplitter::default()`.
pub struct CylinderRangeSplitter {
    base: DefaultRangeSplitter,
    du: f64,
}
impl Default for CylinderRangeSplitter {
    fn default() -> Self {
        Self {
            base: DefaultRangeSplitter::default(),
            du: 1.0,
        }
    }
}
impl std::ops::Deref for CylinderRangeSplitter {
    type Target = DefaultRangeSplitter;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for CylinderRangeSplitter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CylinderRangeSplitter {
    /// `Reset`: the default reset, then `myDu`, the arc angular step of the radius;
    /// `splitter.reset_cylinder(&face, &parameters)?`.
    pub fn reset_cylinder(
        &mut self,
        face: &FaceHandle,
        parameters: &MeshParameters,
    ) -> Result<(), Failure> {
        self.base.reset(face, parameters);
        let radius = self.get_surface().cylinder()?.radius();
        self.du = TangentialDeflection::arc_angular_step(
            radius,
            self.face.get_deflection(),
            parameters.angle,
            parameters.min_size,
        )?;
        Ok(())
    }

    /// `GenerateSurfaceNodes`: `(int)(su / myDu)` U steps when the arc is longer than
    /// the deflection, a single V row step (the native V count stays 0: its formula is
    /// commented out upstream); `splitter.generate_surface_nodes()?`.
    pub fn generate_surface_nodes(&self) -> Result<SurfaceNodes, Failure> {
        let radius = self.get_surface().cylinder()?.radius();
        let su = self.range_u.1 - self.range_u.0;
        let sv = self.range_v.1 - self.range_v.0;
        let mut nb_u = 0_i32;
        let nb_v = 0_i32;
        if su * radius > self.face.get_deflection() {
            nb_u = (su / self.du).truncate_to_int();
        }
        let du = su / f64::from(nb_u.wrapping_add(1));
        let dv = sv / f64::from(nb_v + 1);
        let mut nodes = new_surface_nodes();
        append_regular_grid(&mut nodes, self.range_u, self.range_v, (du, dv));
        Ok(Handle::new(Shared::new(nodes)))
    }

    /// `computeDelta`: `myDu / max(LengthV, DiffV)` along U and 1 along V.
    pub(super) fn compute_cylinder_delta(&mut self, length_v: f64) {
        let diff_v = self.range_v.1 - self.range_v.0;
        self.base.delta = (self.du / std_max(length_v, diff_v), 1.0);
    }

    /// `myDu`, the U step of the last reset; `splitter.du()`.
    pub fn du(&self) -> f64 {
        self.du
    }
}
