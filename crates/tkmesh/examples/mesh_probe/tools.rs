//! Native IMeshTools probe twin.
use super::super::{flag, int, orientation, point3, real};
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, Transient},
};
use rustycad_tkmesh::{
    imesh_data::MeshShape,
    imesh_tools::{MeshParameters, ModelAlgo, ShapeExplorer},
};
#[path = "../../tests/support/tools_fixture.rs"]
mod fixtures;
pub fn tools_case(outcome: i32) {
    let p = MeshParameters::default();
    int(p.mesh_algo as i32);
    real(p.angle);
    real(p.deflection);
    real(p.angle_interior);
    real(p.deflection_interior);
    real(p.min_size);
    flag(p.in_parallel);
    flag(p.relative);
    flag(p.internal_vertices_mode);
    flag(p.control_surface_deflection);
    flag(p.enable_control_surface_deflection_all_surfaces);
    flag(p.clean_model);
    flag(p.adjust_min_size);
    flag(p.force_face_deflection);
    flag(p.allow_quality_decrease);
    real(MeshParameters::rel_min_size());
    let algorithm = fixtures::OracleModelAlgorithm::new(outcome);
    flag(algorithm.perform(&Handle::default(), &p, &ProgressRange::default()));
    flag(algorithm.is_kind_name("IMeshTools_ModelAlgo"));
    println!();
}
pub fn explorer_case(orient: i32, duplicate: bool) {
    let (shape, identities) = fixtures::explorer_fixture(orientation(orient), duplicate);
    let visitor = Handle::new(fixtures::RecordingShapeVisitor::default());
    let explorer = ShapeExplorer::new(&shape);
    flag(explorer.get_shape().is_same(&shape));
    explorer.accept(&visitor.cast()).unwrap();
    for visited in visitor.get().unwrap().shapes.read().iter() {
        int(identities
            .iter()
            .position(|source| source.is_partner(visited))
            .unwrap() as i32);
        int(visited.shape_type().unwrap() as i32);
        int(visited.orientation() as i32);
        let translation = visited.location().transformation().translation_part();
        point3(rustycad_tkmath::gp::Point3::new(
            translation.x(),
            translation.y(),
            translation.z(),
        ));
    }
    explorer.set_shape(&Default::default());
    explorer.accept(&visitor.cast()).unwrap();
    flag(explorer.get_shape().is_null());
    println!();
}
