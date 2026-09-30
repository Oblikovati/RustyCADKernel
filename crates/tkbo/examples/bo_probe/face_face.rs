use super::{flag, integer, real};
use rustycad_tkbo::int_tools::{Context, FaceFace};
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeomalgo::int_surf::IntSurfPntOn2S;
fn state(ff: &FaceFace) {
    flag(ff.is_done());
    flag(ff.tangent_faces());
    flag(ff.face1().is_null());
    flag(ff.face2().is_null());
    flag(ff.context().is_null());
    integer(ff.points().len() as i32);
    real(ff.fuzzy_value());
    match ff.lines() {
        Ok(curves) => integer(curves.len() as i32),
        Err(error) => print!("{} ", error.exception_type()),
    }
}
pub fn case(v: &[f64]) {
    let mut ff = FaceFace::default();
    state(&ff);
    ff.set_parameters(v[0] != 0., v[1] != 0., v[2] != 0., v[3]);
    ff.set_fuzzy_value(if v[4] == 99. { f64::NAN } else { v[4] });
    let context = Handle::new(Context::default());
    ff.set_context(&context);
    flag(ff.context() == &context);
    ff.set_list(&vec![IntSurfPntOn2S::default(); v[5] as usize]);
    state(&ff);
    prepare_and_clear(&mut ff);
}
fn prepare_and_clear(ff: &mut FaceFace) {
    for split in [false, true] {
        match ff.prepare_lines_3d(split) {
            Ok(()) => flag(true),
            Err(error) => print!("{} ", error.exception_type()),
        }
    }
    ff.set_context(&Handle::default());
    state(ff);
}

pub fn clip_case(v: &[f64]) {
    use rustycad_tkmath::gp::{Direction2, Line2, Point2};
    let direction = Direction2::new(v[6], v[7]).unwrap();
    let line = Line2::new(Point2::new(v[4], v[5]), direction);
    let result = super::plane_clip::classify_line([v[0], v[1], v[2], v[3]], line, v[8]).unwrap();
    flag(result.is_some());
    let (first, last) = result.unwrap_or((9., 10.));
    real(first);
    real(last);
}
