//! Behavior of the GeomFill skinning classes (`GeomFill_Line`, `GeomFill_Profiler`,
//! `GeomFill_SectionGenerator`, `GeomFill_AppSurf`) beyond the numeric probe.
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::app_blend::{AppBlendApprox, AppBlendSection, TheLine};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillAppSurf, GeomFillLine, GeomFillProfiler, GeomFillSectionGenerator,
};
use rustycad_tkmath::gp::Point3;

#[path = "support/geom_fill_sections.rs"]
mod geom_fill_sections;
use geom_fill_sections::circle_and_ellipse;

/// The two-section approximation narrows the V degree to 1, as natively (occ13904).
#[test]
fn two_sections_give_a_ruled_approximation() -> Result<(), Failure> {
    let mut section = GeomFillSectionGenerator::new();
    for curve in circle_and_ellipse()? {
        section.add_curve(&curve)?;
    }
    section.perform(precision::p_confusion())?;
    let line = GeomFillLine::with_nb_points(2);
    let mut approx = GeomFillAppSurf::new(3, 8, 1e-7, 1e-9, 0, false);
    approx.perform(Some(&line), &mut section, false)?;
    assert_eq!(approx.v_degree()?, 1);
    Ok(())
}

#[test]
fn line_points_are_their_indices() {
    let line = GeomFillLine::with_nb_points(4);
    assert_eq!(line.nb_points(), 4);
    assert_eq!(line.point(-3), -3);
    assert_eq!(TheLine::point(&line, 9), 9);
    assert_eq!(GeomFillLine::new().nb_points(), 0);
}

#[test]
fn profiler_queries_raise_not_done_before_perform() -> Result<(), Failure> {
    let mut profiler = GeomFillProfiler::new();
    assert!(profiler.is_periodic());
    for curve in circle_and_ellipse()? {
        profiler.add_curve(&curve)?;
    }
    // The approximated conics are not periodic B-splines.
    assert!(!profiler.is_periodic());
    assert_eq!(
        profiler.degree().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        profiler.curve(3).err().map(|e| e.exception_type()),
        Some("Standard_OutOfRange")
    );
    profiler.perform(1e-9)?;
    let np: i32 = profiler.nb_poles()?;
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, np + 1).unwrap();
    assert_eq!(
        profiler.poles(1, &mut poles).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, np).unwrap();
    assert_eq!(
        profiler.poles(0, &mut poles).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    profiler.poles(2, &mut poles)?;
    Ok(())
}

#[test]
fn empty_profiler_perform_raises_out_of_range() {
    let error = GeomFillProfiler::new().perform(1e-9).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

/// SetParam copies the array into itself from index 1: a lower bound of 0 shifts the
/// first value over the others, then raises at the last index (native quirk).
#[test]
fn set_param_copies_in_place() -> Result<(), Failure> {
    let mut generator = GeomFillSectionGenerator::new();
    assert_eq!(
        generator.parameter(1).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    let params = Handle::new(HArray1::with_bounds(0, 2).unwrap());
    for (i, value) in [(0, 1.5), (1, 2.5), (2, 4.0)] {
        params.get().unwrap().change_array1()[i] = value;
    }
    let error = generator.set_param(&params).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    let values: Vec<f64> = params.get().unwrap().array1().iter().copied().collect();
    assert_eq!(values, vec![1.5, 1.5, 1.5]);
    assert_eq!(generator.parameter(2)?, 1.5);
    Ok(())
}

#[test]
fn section_assigns_the_poles_and_keeps_the_2d_poles() -> Result<(), Failure> {
    let mut generator = GeomFillSectionGenerator::new();
    for curve in circle_and_ellipse()? {
        generator.add_curve(&curve)?;
    }
    generator.perform(1e-9)?;
    let [np, _, _, np2d] = generator.get_shape()?;
    assert_eq!(np2d, 0);
    let mut section = AppBlendSection {
        poles: Array1::with_bounds(1, 1).unwrap(),
        d_poles: Array1::with_bounds(1, 1).unwrap(),
        poles2d: Array1::with_bounds(1, 1).unwrap(),
        d_poles2d: Array1::with_bounds(1, 1).unwrap(),
        weights: Array1::with_bounds(1, 1).unwrap(),
        d_weights: Array1::with_bounds(1, 1).unwrap(),
    };
    assert!(!generator.section_d1(1, &mut section)?);
    assert_eq!(section.poles.length(), np);
    assert_eq!(section.weights.length(), np);
    assert_eq!(section.poles2d.length(), 1);
    assert_eq!(
        generator
            .section(3, &mut section)
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
    Ok(())
}
