use super::{fclass_faces, flag};
use rustycad_tkbo::int_tools::WLineTool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::{
    int_patch::IntPatchWLine,
    int_surf::{IntSurfLineOn2S, IntSurfPntOn2S},
};
fn run(v: &[f64]) -> Result<(), Failure> {
    let first = fclass_faces::face(v[0] as i32, v[2])?;
    let last = fclass_faces::face(v[1] as i32, v[2])?;
    let points = Handle::new(IntSurfLineOn2S::new(None));
    let mut p = IntSurfPntOn2S::new();
    p.set_parameters(v[3], v[4], v[5], v[6]);
    points.add(&p);
    p.set_parameters(v[5], v[6], v[3], v[4]);
    points.add(&p);
    let line = Handle::new(IntPatchWLine::new_undecided(&points, false));
    flag(WLineTool::not_use_surfaces_for_approx(
        &first, &last, &line, 1, 2,
    )?);
    flag(WLineTool::not_use_surfaces_for_approx(
        &first, &last, &line, 2, 1,
    )?);
    flag(WLineTool::not_use_surfaces_for_approx(
        &last, &first, &line, 1, 2,
    )?);
    flag(WLineTool::not_use_surfaces_for_approx(
        &first, &last, &line, 1, 3,
    )?);
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}
