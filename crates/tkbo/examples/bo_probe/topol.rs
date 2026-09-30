use super::{flag, integer, real, topol_surfaces};
use rustycad_tkbo::int_tools::TopolTool;
use rustycad_tkgeomalgo::int_start::TheTopolTool;
fn state(tool: &dyn TheTopolTool) {
    integer(tool.nb_samples_u().unwrap());
    integer(tool.nb_samples_v().unwrap());
    integer(tool.nb_samples().unwrap());
    for index in [
        1,
        tool.nb_samples_u().unwrap(),
        tool.nb_samples().unwrap() - 1,
    ] {
        point(tool, index);
    }
}
fn point(tool: &dyn TheTopolTool, index: i32) {
    match tool.sample_point(index) {
        Ok((uv, p)) => {
            real(uv.x());
            real(uv.y());
            real(p.x());
            real(p.y());
            real(p.z());
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
}
fn parameters(tool: &dyn TheTopolTool) {
    for array in [tool.u_parameters().unwrap(), tool.v_parameters().unwrap()] {
        integer(array.length());
        for value in array.as_slice() {
            real(*value);
        }
    }
}
pub fn case(v: &[f64]) {
    let surface = topol_surfaces::surface(v);
    let tool = TopolTool::with_surface(&surface).unwrap();
    print!("{} ", tool.initialize().unwrap_err().exception_type());
    flag(tool.more());
    state(&tool);
    tool.compute_sample_points().unwrap();
    state(&tool);
    tool.sample_pnts(0.1, 4, 5).unwrap();
    state(&tool);
    parameters(&tool);
    tool.initialize_surface(&surface).unwrap();
    point(&tool, 1);
    state(&tool);
    parameters(&tool);
    tool.sample_pnts(0.2, 6, 7).unwrap();
    state(&tool);
    super::topol_domain::emit(&tool).unwrap();
}
