//! SweepApproximation over a named fake section, and the SweepFunction defaults.
use super::fakes::TwoPoleSweep;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::approx::{SweepApproximation, SweepFunction, SweepOptions};
use rustycad_tkmath::geom_abs::Shape;

fn sweep(nb2d: i32, derivatives: bool) -> (Handle<TwoPoleSweep>, Handle<dyn SweepFunction>) {
    let fake = Handle::new(TwoPoleSweep::new(nb2d, derivatives));
    let func: Handle<dyn SweepFunction> = fake.clone().cast();
    (fake, func)
}

fn options(continuity: Shape) -> SweepOptions {
    SweepOptions {
        continuity,
        ..SweepOptions::new(1.0e-4, 1.0e-4, 1.0e-5, 1.0e-2)
    }
}

#[test]
fn sweep_accessors_raise_before_perform() {
    let (_, func) = sweep(1, false);
    let approx = SweepApproximation::new(&func);
    assert!(!approx.is_done());
    assert_eq!(
        approx.surf_poles().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        approx.u_degree().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
}

#[test]
fn sweep_c0_approximates_the_ruled_section() -> Result<(), Failure> {
    let (fake, func) = sweep(1, false);
    let mut approx = SweepApproximation::new(&func);
    approx.perform(0.0, 1.0, options(Shape::C0))?;
    assert!(approx.is_done());
    assert_eq!(approx.u_degree()?, 1);
    assert_eq!(approx.surf_poles()?.col_length(), 2);
    assert!(approx.max_error_on_surf()? < 1.0e-4);
    assert_eq!(approx.nb_curves_2d()?, 1);
    assert!(approx.max_2d_error(1)? < 1.0e-5);
    assert!(*fake.get().unwrap().intervals_set.lock().unwrap() >= 1);
    Ok(())
}

#[test]
fn sweep_without_2d_curves_raises_on_the_2d_accessors() -> Result<(), Failure> {
    let (_, func) = sweep(0, false);
    let mut approx = SweepApproximation::new(&func);
    approx.perform(0.0, 1.0, options(Shape::C0))?;
    assert!(approx.is_done());
    assert_eq!(
        approx.curves_2d_degree().err().unwrap().exception_type(),
        "Standard_DomainError"
    );
    assert_eq!(
        approx.curves_2d_knots().err().unwrap().exception_type(),
        "Standard_DomainError"
    );
    Ok(())
}

#[test]
fn sweep_c1_raises_the_default_d1() {
    let (_, func) = sweep(1, false);
    let mut approx = SweepApproximation::new(&func);
    let failure = approx.perform(0.0, 1.0, options(Shape::C1)).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_NotImplemented");
}

#[test]
fn sweep_c2_uses_the_given_derivatives() -> Result<(), Failure> {
    let (_, func) = sweep(1, true);
    let mut approx = SweepApproximation::new(&func);
    approx.perform(0.0, 1.0, options(Shape::C2))?;
    assert!(approx.is_done());
    assert!(approx.max_error_on_surf()? < 1.0e-4);
    Ok(())
}

#[test]
fn sweep_eval_answers_2_beyond_the_second_order() -> Result<(), Failure> {
    let (_, func) = sweep(1, true);
    let mut approx = SweepApproximation::new(&func);
    approx.perform(0.0, 1.0, options(Shape::C0))?;
    let mut values = [0.0; 16];
    assert_eq!(approx.eval(0.5, 0, 0.0, 1.0, &mut values)?, 0);
    assert_eq!(approx.eval(0.5, 3, 0.0, 1.0, &mut values)?, 2);
    Ok(())
}

#[test]
fn sweep_function_defaults_raise_not_implemented() {
    let (fake, _) = sweep(0, false);
    let fake = fake.get().unwrap();
    let mut weights = Array1::with_bounds(1, 2).unwrap();
    for failure in [
        fake.barycentre_of_surf().err(),
        fake.maximal_section().err(),
        fake.get_minimal_weight(&mut weights).err(),
    ] {
        assert_eq!(failure.unwrap().exception_type(), "Standard_NotImplemented");
    }
}

#[test]
fn sweep_perform_appends_the_2d_poles_across_runs() -> Result<(), Failure> {
    // Quirk: seqPoles2d is never cleared, so the 2D curves of the first run stay in
    // front; Curve2dPoles(1) keeps answering the first run's poles.
    let (_, func) = sweep(1, false);
    let mut approx = SweepApproximation::new(&func);
    approx.perform(0.0, 1.0, options(Shape::C0))?;
    let first = approx.curve_2d_poles(1)?.length();
    approx.perform(0.0, 1.0, options(Shape::C0))?;
    assert_eq!(approx.curve_2d_poles(1)?.length(), first);
    Ok(())
}
