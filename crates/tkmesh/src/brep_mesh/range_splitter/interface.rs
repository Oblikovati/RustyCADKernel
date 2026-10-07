//! Virtual splitter hooks shared by native surface-specific implementations.
use super::*;
/// Polymorphic splitter contract; `let splitter: &dyn RangeSplitter = &cone`.
pub trait RangeSplitter {
    /// Read the common range state; `splitter.range_state()`.
    fn range_state(&self) -> &DefaultRangeSplitter;
    /// Change common range state; `splitter.range_state_mut().add_point(point)`.
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter;
    /// Initialize a border collection; `splitter.reset(&face, &parameters)?`.
    /// Derived splitters read the face surface here (a cylinder its radius), so the
    /// virtual form can fail with the surface accessor's Standard_NoSuchObject.
    fn reset(&mut self, face: &FaceHandle, parameters: &MeshParameters) -> Result<(), Failure> {
        self.range_state_mut().reset(face, parameters);
        Ok(())
    }
    /// Accumulate a border point; `splitter.add_point(point)`.
    fn add_point(&mut self, point: Point2) {
        self.range_state_mut().add_point(point);
    }
    /// Adjust bounds and dispatch tolerance/scaling overrides; `splitter.adjust_range()?`.
    fn adjust_range(&mut self) -> Result<(), Failure> {
        let Some((length_u, length_v)) = self.range_state_mut().adjust_lengths()? else {
            return Ok(());
        };
        self.compute_tolerance(length_u, length_v)?;
        self.compute_delta(length_u, length_v);
        Ok(())
    }
    /// Report the last range computation; `splitter.is_valid()`.
    fn is_valid(&self) -> bool {
        self.range_state().is_valid()
    }
    /// Generate surface-specific interior nodes; `splitter.generate_surface_nodes(&parameters)?`.
    /// Native `const`, but the NURBS family fills its parameter maps here (through a
    /// `const_cast` natively), so the Rust receiver is mutable.
    fn generate_surface_nodes(
        &mut self,
        parameters: &MeshParameters,
    ) -> Result<SurfaceNodes, Failure> {
        self.range_state().generate_surface_nodes(parameters)
    }
    /// Override native parametric tolerance calculation; `splitter.compute_tolerance(1.0, 1.0)?`.
    fn compute_tolerance(&mut self, length_u: f64, length_v: f64) -> Result<(), Failure> {
        self.range_state_mut().compute_tolerance(length_u, length_v)
    }
    /// Override native physical scaling; `splitter.compute_delta(1.0, 1.0)`.
    fn compute_delta(&mut self, length_u: f64, length_v: f64) {
        self.range_state_mut().compute_delta(length_u, length_v);
    }
}
impl RangeSplitter for DefaultRangeSplitter {
    fn range_state(&self) -> &DefaultRangeSplitter {
        self
    }
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter {
        self
    }
}
impl RangeSplitter for ConeRangeSplitter {
    fn range_state(&self) -> &DefaultRangeSplitter {
        self
    }
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter {
        self
    }
    fn generate_surface_nodes(
        &mut self,
        parameters: &MeshParameters,
    ) -> Result<SurfaceNodes, Failure> {
        ConeRangeSplitter::generate_surface_nodes(self, parameters)
    }
}
impl RangeSplitter for SphereRangeSplitter {
    fn range_state(&self) -> &DefaultRangeSplitter {
        self
    }
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter {
        self
    }
    fn generate_surface_nodes(
        &mut self,
        parameters: &MeshParameters,
    ) -> Result<SurfaceNodes, Failure> {
        SphereRangeSplitter::generate_surface_nodes(self, parameters)
    }
}
impl RangeSplitter for CylinderRangeSplitter {
    fn range_state(&self) -> &DefaultRangeSplitter {
        self
    }
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter {
        self
    }
    fn reset(&mut self, face: &FaceHandle, parameters: &MeshParameters) -> Result<(), Failure> {
        self.reset_cylinder(face, parameters)
    }
    fn generate_surface_nodes(
        &mut self,
        _parameters: &MeshParameters,
    ) -> Result<SurfaceNodes, Failure> {
        CylinderRangeSplitter::generate_surface_nodes(self)
    }
    fn compute_delta(&mut self, _length_u: f64, length_v: f64) {
        self.compute_cylinder_delta(length_v);
    }
}
impl RangeSplitter for UVParamRangeSplitter {
    fn range_state(&self) -> &DefaultRangeSplitter {
        self
    }
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter {
        self
    }
    fn reset(&mut self, face: &FaceHandle, parameters: &MeshParameters) -> Result<(), Failure> {
        self.reset_params(face, parameters);
        Ok(())
    }
}
impl RangeSplitter for TorusRangeSplitter {
    fn range_state(&self) -> &DefaultRangeSplitter {
        self
    }
    fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter {
        self
    }
    fn reset(&mut self, face: &FaceHandle, parameters: &MeshParameters) -> Result<(), Failure> {
        self.reset_params(face, parameters);
        Ok(())
    }
    fn add_point(&mut self, point: Point2) {
        self.add_point_params(point);
    }
    fn generate_surface_nodes(
        &mut self,
        parameters: &MeshParameters,
    ) -> Result<SurfaceNodes, Failure> {
        TorusRangeSplitter::generate_surface_nodes(self, parameters)
    }
}
/// The RangeSplitter overrides shared by the NURBS family: the UV-parameter reset, the
/// NURBS AdjustRange and GenerateSurfaceNodes, and (for the boundary-parameters
/// splitter) the recording AddPoint.
macro_rules! nurbs_range_splitter {
    ($splitter:ty, $add_point:path) => {
        impl RangeSplitter for $splitter {
            fn range_state(&self) -> &DefaultRangeSplitter {
                self
            }
            fn range_state_mut(&mut self) -> &mut DefaultRangeSplitter {
                self
            }
            fn reset(
                &mut self,
                face: &FaceHandle,
                parameters: &MeshParameters,
            ) -> Result<(), Failure> {
                self.reset_params(face, parameters);
                Ok(())
            }
            fn add_point(&mut self, point: Point2) {
                $add_point(self, point);
            }
            fn adjust_range(&mut self) -> Result<(), Failure> {
                nurbs::nurbs_adjust_range(self)
            }
            fn generate_surface_nodes(
                &mut self,
                parameters: &MeshParameters,
            ) -> Result<SurfaceNodes, Failure> {
                nurbs::nurbs_generate_surface_nodes(self, parameters)
            }
        }
    };
}
nurbs_range_splitter!(NURBSRangeSplitter, default_add_point);
nurbs_range_splitter!(BoundaryParamsRangeSplitter, params_add_point);
nurbs_range_splitter!(ExtrusionRangeSplitter, default_add_point);
nurbs_range_splitter!(UndefinedRangeSplitter, default_add_point);
/// `BRepMesh_DefaultRangeSplitter::AddPoint` on a NURBS-family splitter.
fn default_add_point<T: RangeSplitter>(splitter: &mut T, point: Point2) {
    splitter.range_state_mut().add_point(point);
}
/// `BRepMesh_BoundaryParamsRangeSplitter::AddPoint`: the range and both parameters.
fn params_add_point(splitter: &mut BoundaryParamsRangeSplitter, point: Point2) {
    splitter.add_point_params(point);
}
impl NurbsRangeSplitterHooks for NURBSRangeSplitter {
    fn nurbs_state(&self) -> &NURBSRangeSplitter {
        self
    }
    fn nurbs_state_mut(&mut self) -> &mut NURBSRangeSplitter {
        self
    }
}
impl NurbsRangeSplitterHooks for BoundaryParamsRangeSplitter {
    fn nurbs_state(&self) -> &NURBSRangeSplitter {
        self
    }
    fn nurbs_state_mut(&mut self) -> &mut NURBSRangeSplitter {
        self
    }
    fn init_parameters(&mut self) -> Result<bool, Failure> {
        Ok(true)
    }
}
impl NurbsRangeSplitterHooks for ExtrusionRangeSplitter {
    fn nurbs_state(&self) -> &NURBSRangeSplitter {
        self
    }
    fn nurbs_state_mut(&mut self) -> &mut NURBSRangeSplitter {
        self
    }
    fn get_undefined_interval_nb(
        &self,
        surface: &BRepAdaptorSurface,
        _is_u: bool,
        _continuity: rustycad_tkmath::geom_abs::Shape,
    ) -> Result<i32, Failure> {
        nurbs::extrusion_interval_nb(surface)
    }
}
impl NurbsRangeSplitterHooks for UndefinedRangeSplitter {
    fn nurbs_state(&self) -> &NURBSRangeSplitter {
        self
    }
    fn nurbs_state_mut(&mut self) -> &mut NURBSRangeSplitter {
        self
    }
    fn get_undefined_interval_nb(
        &self,
        _surface: &BRepAdaptorSurface,
        _is_u: bool,
        _continuity: rustycad_tkmath::geom_abs::Shape,
    ) -> Result<i32, Failure> {
        Ok(1)
    }
}
