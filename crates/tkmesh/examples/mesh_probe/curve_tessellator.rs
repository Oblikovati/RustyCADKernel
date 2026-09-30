//! Exact native curve sampling on production models, with unnormalized parameters.
use super::super::{flag, int, orientation, point3, real};
use super::model_builder::fixtures;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshCurveTessellator, BRepMeshModelBuilder},
    imesh_tools::{CurveTessellator, MeshParameters, ModelBuilder},
};
pub fn curve_tessellator_case(v: &[i32]) {
    match make_tessellator(v) {
        Ok(tessellator) => {
            print!("ok ");
            report_tessellator(&tessellator);
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
    println!();
}
fn make_tessellator(v: &[i32]) -> Result<BRepMeshCurveTessellator, Failure> {
    let model = BRepMeshModelBuilder::new()
        .perform(&fixtures::model_shape(v[0]), &MeshParameters::default());
    let edge = model.get_edge(v[1] % model.edges_nb())?;
    edge.set_shape(&edge.get_shape().oriented(orientation(v[3])));
    edge.set_deflection([0.01, 0.5][v[4] as usize]);
    edge.set_angular_deflection(0.5);
    let parameters = MeshParameters {
        min_size: [-1.0, 0.001][v[5] as usize],
        adjust_min_size: v[6] != 0,
        ..Default::default()
    };
    if v[2] == 0 {
        return BRepMeshCurveTessellator::new(&edge, &parameters, [2, 5][v[7] as usize]);
    }
    let face = edge.get_pcurve(0)?.get_face().upgrade();
    BRepMeshCurveTessellator::on_face(
        &edge,
        orientation(v[3]),
        &face,
        &parameters,
        [2, 5][v[7] as usize],
    )
}
fn report_tessellator(tessellator: &BRepMeshCurveTessellator) {
    int(tessellator.points_nb());
    let mut point = Default::default();
    let mut parameter = 0.0;
    for index in 1..=tessellator.points_nb() {
        flag(
            tessellator
                .value(index, &mut point, &mut parameter)
                .unwrap(),
        );
        point3(point);
        real(parameter);
    }
}

#[path = "../../tests/support/curve_tessellator_fixture.rs"]
pub(super) mod refinement;
pub fn curve_refinement_case(v: &[i32]) {
    let (edge, face) = refinement::refinement_fixture(v[0], v[1] != 0, v[2] != 0);
    edge.set_deflection([0.01, 0.5][v[3] as usize]);
    edge.set_angular_deflection(0.5);
    edge.set_same_param(v[4] & 1 != 0);
    edge.set_same_range(v[4] & 2 != 0);
    let parameters = MeshParameters {
        min_size: [0.001, 1.0][v[5] as usize],
        ..Default::default()
    };
    let result = if v[6] == 0 {
        BRepMeshCurveTessellator::new(&edge, &parameters, 2)
    } else {
        BRepMeshCurveTessellator::on_face(
            &edge,
            rustycad_tkg3d::top_abs::Orientation::Forward,
            &face,
            &parameters,
            2,
        )
    };
    match result {
        Ok(tessellator) => {
            print!("ok ");
            report_tessellator(&tessellator);
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
    println!();
}
