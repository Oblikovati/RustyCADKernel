//! A tessellator callback mutating endpoints exposes the native evaluation order.
use rustycad_tkbrep::{brep::BRepBuilder, topods::Vertex};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, StandardTransient},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmesh::imesh_tools::CurveTessellator;
pub struct MutatingEndpoints {
    pub transient: StandardTransient,
    pub first: Vertex,
    pub last: Vertex,
}
impl_transient!(MutatingEndpoints,"MutatingEndpoints",dyn CurveTessellator,transient,interfaces:[dyn CurveTessellator]);
impl CurveTessellator for MutatingEndpoints {
    fn points_nb(&self) -> i32 {
        2
    }
    fn value(&self, index: i32, _point: &mut Point3, parameter: &mut f64) -> Result<bool, Failure> {
        *parameter = f64::from(index);
        let vertex = if index == 1 { &self.first } else { &self.last };
        BRepBuilder::default().update_vertex_point(
            vertex,
            Point3::new(f64::from(index) * 7.0, 0.0, 0.0),
            1e-7,
        )?;
        Ok(false)
    }
}
