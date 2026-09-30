//! Native differential fixtures exercise the whole decomposition; these assert durable invariants.
#[path = "../examples/bo_probe/wline_cases.rs"]
mod cases;
#[path = "../examples/bo_probe/fclass_faces.rs"]
mod faces;
use rustycad_tkbo::int_tools::Context;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchLine, IntPatchPointLine, IntPatchWLType, IntPatchWLine,
};
fn split(pattern: i32, mode: i32) -> Result<(bool, Vec<Handle<dyn IntPatchLine>>), Failure> {
    let face = faces::face(1, 2.)?;
    let surface = BRepTool::surface(&face)?;
    let adaptor = Handle::new(GeomAdaptorSurface::with_surface(&surface)?);
    let request = [1., 1., f64::from(pattern), 0., 0., f64::from(mode)];
    let line = cases::walking(&surface, &request)?;
    let constructor = cases::constructor(&line, &adaptor, &adaptor, mode)?;
    let context = Handle::new(Context::default());
    let mut output = vec![line.clone().into_cast()];
    let done = cases::decompose(
        &line,
        [&adaptor, &adaptor],
        [&face, &face],
        &constructor,
        mode == 1,
        &mut output,
        &context,
    )?;
    Ok((done, output))
}
#[test]
fn empty_boundary_only_and_interior_only_lines_leave_output_untouched() {
    for pattern in 0..3 {
        let (done, output) = split(pattern, 1).unwrap();
        assert!(!done);
        assert_eq!(output.len(), 1);
        assert_eq!(
            output[0].cast::<IntPatchWLine>().nb_pnts().unwrap(),
            if pattern == 0 { 0 } else { 3 }
        );
    }
}
#[test]
fn periodic_endpoints_keep_creation_way_and_append_four_point_line() {
    for mode in [1, 2] {
        let (done, output) = split(3, mode).unwrap();
        assert!(done);
        assert_eq!(output.len(), 2);
        let line = output[1].cast::<IntPatchWLine>();
        assert_eq!(line.nb_pnts().unwrap(), 4);
        assert_eq!(line.get_creating_way(), IntPatchWLType::ImpImp);
        assert_eq!(line.point(1).unwrap().parameters_on_s1(), (0., 0.));
        assert_eq!(
            line.point(4).unwrap().parameters_on_s1(),
            (std::f64::consts::TAU, 0.)
        );
    }
}
#[test]
fn middle_seam_produces_two_new_lines() {
    let (done, output) = split(4, 1).unwrap();
    assert!(done);
    assert_eq!(output.len(), 3);
    for item in &output[1..] {
        assert_eq!(item.cast::<IntPatchWLine>().nb_pnts().unwrap(), 3);
    }
}
#[test]
fn constructor_parts_are_honoured_and_unperformed_constructor_fails() {
    let (done, output) = split(3, 4).unwrap();
    assert!(done);
    assert_eq!(output.len(), 2);
    let line = output[1].cast::<IntPatchWLine>();
    assert_eq!(line.nb_pnts().unwrap(), 2);
    assert_eq!(line.point(1).unwrap().parameters_on_s1(), (0.1, 0.));
    assert_eq!(line.point(2).unwrap().parameters_on_s1(), (0.2, 0.));
    let failure = match split(3, 0) {
        Ok(_) => panic!("unperformed constructor unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(failure.exception_type(), "StdFail_NotDone");
}
