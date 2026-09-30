use super::super::{flag, int, point2, point3, real};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmesh::{
    brep_mesh::{ConeRangeSplitter, DefaultRangeSplitter},
    imesh_tools::MeshParameters,
};
#[path = "../../tests/support/range_splitter_fixture.rs"]
mod fixture;
pub fn range_case(v: &[i32]) {
    let parameters = MeshParameters {
        min_size: 0.001,
        angle: 0.5,
        ..Default::default()
    };
    let face = fixture::range_face(v[0], [1e-7, 0.01][v[2] as usize]);
    face.set_deflection([0.01, 0.1][v[2] as usize]);
    let mut splitter = ConeRangeSplitter::default();
    flag(splitter.is_valid());
    splitter.reset(&face, &parameters);
    let bounds = fixture::range_bounds(v[1]);
    splitter.add_point(Point2::new(bounds[0], bounds[2]));
    splitter.add_point(Point2::new(bounds[1], bounds[3]));
    if v[3] != 0 {
        splitter.adjust_range().unwrap();
    }
    report_range(&splitter);
    if v[0] == 0 && v[1] != 3 {
        report_cone(&splitter, &parameters);
    }
    flag(
        DefaultRangeSplitter::generate_surface_nodes(&splitter, &parameters)
            .unwrap()
            .is_null(),
    );
    splitter.reset(&face, &parameters);
    flag(splitter.is_valid());
    println!();
}
fn report_range(splitter: &DefaultRangeSplitter) {
    flag(splitter.is_valid());
    for pair in [
        splitter.get_range_u(),
        splitter.get_range_v(),
        splitter.get_delta(),
        splitter.get_tolerance_uv(),
    ] {
        real(pair.0);
        real(pair.1);
    }
    let point = Point2::new(0.37, 0.62);
    point2(splitter.scale(point, true));
    point2(splitter.scale(point, false));
    point3(splitter.point(point).unwrap());
    flag(!splitter.get_dface().is_null());
    flag(!splitter.get_surface().is_null());
}
fn report_cone(splitter: &ConeRangeSplitter, parameters: &MeshParameters) {
    let mut counts = (0, 0);
    let steps = splitter.get_split_steps(parameters, &mut counts).unwrap();
    real(steps.0);
    real(steps.1);
    int(counts.0);
    int(counts.1);
    let nodes = splitter.generate_surface_nodes(parameters).unwrap();
    int(nodes.length());
    for point in nodes.iter() {
        point2(*point);
    }
}
