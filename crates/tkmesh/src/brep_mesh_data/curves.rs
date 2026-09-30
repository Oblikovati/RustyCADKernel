//! Concrete curve and pcurve implementations from BRepMeshData.
use super::{MeshAllocator, mesh_index, samples::CurveSamples};
use crate::imesh_data::{FacePtr, MeshCurve, MeshPCurve, ParametersList};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, SharedState, StandardTransient},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Point3};
macro_rules! parameter_members {
    ($name:ty) => {
        impl ParametersList for $name {
            fn get_parameter(&self, index: i32) -> Result<f64, Failure> {
                let samples = self.samples.read();
                Ok(
                    samples.parameters
                        [mesh_index(index, samples.parameters.len(), "GetParameter")?],
                )
            }
            fn set_parameter(&self, index: i32, value: f64) -> Result<(), Failure> {
                let mut samples = self.samples.write();
                let index = mesh_index(index, samples.parameters.len(), "GetParameter")?;
                samples.parameters[index] = value;
                Ok(())
            }
            fn parameters_nb(&self) -> i32 {
                self.samples.read().parameters.len() as i32
            }
            fn clear(&self, keep_endpoints: bool) {
                self.samples.write().clear(keep_endpoints);
            }
            fn remove_parameter(&self, index: i32) -> Result<(), Failure> {
                let mut samples = self.samples.write();
                let index = mesh_index(index, samples.parameters.len(), "removeParameter")?;
                samples.parameters.remove(index);
                Ok(())
            }
        }
    };
}
macro_rules! point_members {
    ($point:ty) => {
        fn insert_point(
            &self,
            position: i32,
            point: $point,
            parameter: f64,
        ) -> Result<(), Failure> {
            self.samples.write().insert(position, point, parameter)
        }
        fn add_point(&self, point: $point, parameter: f64) {
            self.samples.write().append(point, parameter);
        }
        fn get_point(&self, index: i32) -> Result<$point, Failure> {
            let samples = self.samples.read();
            Ok(samples.points[mesh_index(index, samples.points.len(), "GetPoint")?])
        }
        fn set_point(&self, index: i32, point: $point) -> Result<(), Failure> {
            let mut samples = self.samples.write();
            let index = mesh_index(index, samples.points.len(), "GetPoint")?;
            samples.points[index] = point;
            Ok(())
        }
        fn remove_point(&self, index: i32) -> Result<(), Failure> {
            self.samples.write().remove(index)
        }
    };
}
/// Native `BRepMeshData_Curve` with allocator-owned samples.
pub struct DiscreteCurve {
    transient: StandardTransient,
    samples: SharedState<CurveSamples<Point3>>,
}
impl_transient!(DiscreteCurve, "BRepMeshData_Curve", dyn MeshCurve, transient, interfaces: [dyn MeshCurve, dyn ParametersList]);
impl DiscreteCurve {
    /// Empty discretization; `DiscreteCurve::new(&allocator)`.
    pub fn new(allocator: &MeshAllocator) -> Self {
        Self {
            transient: StandardTransient::default(),
            samples: SharedState::new(CurveSamples::new(allocator, false)),
        }
    }
}
parameter_members!(DiscreteCurve);
impl MeshCurve for DiscreteCurve {
    point_members!(Point3);
}
/// Native `BRepMeshData_PCurve` with allocator-owned samples.
pub struct DiscretePCurve {
    transient: StandardTransient,
    samples: SharedState<CurveSamples<Point2>>,
    face: FacePtr,
    orientation: Orientation,
}
impl_transient!(DiscretePCurve, "BRepMeshData_PCurve", dyn MeshPCurve, transient, interfaces: [dyn MeshPCurve, dyn ParametersList]);
impl DiscretePCurve {
    /// Empty discretization; `DiscretePCurve::new(face, orientation, &allocator)`.
    pub fn new(face: FacePtr, orientation: Orientation, allocator: &MeshAllocator) -> Self {
        Self {
            transient: StandardTransient::default(),
            samples: SharedState::new(CurveSamples::new(allocator, true)),
            face,
            orientation,
        }
    }
}
parameter_members!(DiscretePCurve);
impl MeshPCurve for DiscretePCurve {
    point_members!(Point2);
    fn get_face(&self) -> FacePtr {
        self.face.clone()
    }
    fn get_orientation(&self) -> Orientation {
        self.orientation
    }
    fn get_index(&self, index: i32) -> Result<i32, Failure> {
        let samples = self.samples.read();
        let indices = samples.indices.as_ref().unwrap();
        Ok(indices[mesh_index(index, indices.len(), "GetIndex")?])
    }
    fn set_index(&self, index: i32, value: i32) -> Result<(), Failure> {
        let mut samples = self.samples.write();
        let indices = samples.indices.as_mut().unwrap();
        let index = mesh_index(index, indices.len(), "GetIndex")?;
        indices[index] = value;
        Ok(())
    }
}
