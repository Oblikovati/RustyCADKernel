use super::{fclass_faces, flag, integer, real};
use rustycad_tkbo::int_tools::Context;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::{IntPatchPointLine, IntPatchWLine};
fn run(v: &[f64]) -> Result<(), Failure> {
    let f1 = fclass_faces::face(v[0] as i32, 2.)?;
    let f2 = fclass_faces::face(v[1] as i32, 2.)?;
    let s1 = BRepTool::surface(&f1)?;
    let s2 = BRepTool::surface(&f2)?;
    let a = Handle::new(GeomAdaptorSurface::with_surface(&s1)?);
    let b = Handle::new(GeomAdaptorSurface::with_surface(&s2)?);
    let line = super::wline_cases::walking(&s1, v)?;
    let constructor = super::wline_cases::constructor(&line, &a, &b, v[5] as i32)?;
    let mut output = vec![line.clone().into_cast()];
    flag(super::wline_cases::decompose(
        &line,
        [&a, &b],
        [&f1, &f2],
        &constructor,
        v[5] == 1.,
        &mut output,
        &Handle::new(Context::default()),
    )?);
    emit_output(output)
}
fn emit_output(
    output: Vec<Handle<dyn rustycad_tkgeomalgo::int_patch::IntPatchLine>>,
) -> Result<(), Failure> {
    integer(output.len() as i32);
    for item in output {
        emit(&item.cast::<IntPatchWLine>())?;
    }
    Ok(())
}
fn emit(line: &Handle<IntPatchWLine>) -> Result<(), Failure> {
    integer(line.nb_pnts()?);
    integer(line.get_creating_way() as i32);
    for index in 1..=line.nb_pnts()? {
        let p = line.point(index)?;
        let xyz = p.value();
        let (u, v, x, y) = p.parameters();
        for value in [xyz.x(), xyz.y(), xyz.z(), u, v, x, y] {
            real(value);
        }
    }
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}
