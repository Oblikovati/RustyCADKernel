//! Behavior of the Plate port beyond the numeric probe, which compares the solves and
//! the constraint constructions with OCCT bit for bit.
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{Array1, Array2, Sequence};
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeomalgo::plate::*;
use rustycad_tkmath::gp::{Xy, Xyz};

fn ppc(u: f64, v: f64, z: f64) -> PlatePinpointConstraint {
    PlatePinpointConstraint::new(Xy::new(u, v), Xyz::new(u, v, z), 0, 0)
}

fn corners() -> PlatePlate {
    let mut plate = PlatePlate::new();
    for (u, v, z) in [
        (0.0, 0.0, 1.0),
        (1.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        (1.0, 1.0, 0.5),
    ] {
        plate.load_pinpoint(&ppc(u, v, z));
    }
    plate
}

fn array<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut result: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

#[test]
fn linear_constraints_check_their_sizes_and_share_their_arrays() {
    let constraints = array(&[ppc(0.2, 0.2, 0.0), ppc(0.8, 0.8, 0.0)]);
    let error = PlateLinearXYZConstraint::from_row(&constraints, &array(&[1.0]))
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_DimensionMismatch");
    let row = PlateLinearXYZConstraint::from_row(&constraints, &array(&[1.0, -1.0])).unwrap();
    assert_eq!(row.coeff().unwrap().nb_rows(), 1);
    let mut matrix: Array2<'static, Xyz> = Array2::with_bounds(0, 1, 0, 1).unwrap();
    matrix.init(Xyz::new(0.0, 0.0, 1.0));
    let scalar = PlateLinearScalarConstraint::from_matrix(&constraints, &matrix).unwrap();
    let copy = scalar.clone();
    copy.set_coeff(2, 2, Xyz::new(1.0, 0.0, 0.0)).unwrap();
    assert_eq!(scalar.coeff().unwrap()[(2, 2)], Xyz::new(1.0, 0.0, 0.0));
    assert!(scalar.set_ppc(3, ppc(0.0, 0.0, 0.0)).is_err());
    let by_row =
        PlateLinearScalarConstraint::from_row(&constraints, &array(&[Xyz::new(0.0, 0.0, 1.0); 2]))
            .unwrap();
    assert_eq!(by_row.get_ppc().unwrap().len(), 2);
    assert!(
        PlateLinearXYZConstraint::from_matrix(
            &constraints,
            &Array2::with_bounds(1, 1, 1, 3).unwrap()
        )
        .is_err()
    );
}

#[test]
fn a_constraint_without_arrays_cannot_be_loaded() {
    let mut plate = PlatePlate::new();
    let error = plate
        .load_linear_xyz(&PlateLinearXYZConstraint::new())
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    assert!(
        plate
            .load_linear_scalar(&PlateLinearScalarConstraint::new())
            .is_err()
    );
    assert!(PlateLinearScalarConstraint::new().get_ppc().is_err());
}

#[test]
fn solved_plate_interpolates_and_copies() {
    let mut plate = corners();
    plate.solve_ti(3, 1.0, &ProgressRange::default()).unwrap();
    assert!(plate.is_done());
    let value = plate.evaluate(Xy::new(1.0, 0.0)).unwrap();
    assert!((value.z() - 0.0).abs() < 1e-9 && (value.x() - 1.0).abs() < 1e-9);
    let mut coefs = Handle::default();
    plate.coef_pol_into(&mut coefs).unwrap();
    assert_eq!(coefs.get().unwrap().array2().nb_rows(), 3);
    let copy = plate.clone();
    assert_eq!(
        copy.evaluate(Xy::new(0.3, 0.3)).unwrap(),
        plate.evaluate(Xy::new(0.3, 0.3)).unwrap()
    );
    let error = plate
        .evaluate_derivative(Xy::new(0.3, 0.3), 12, 0)
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    let mut seq: Sequence<Xy> = Sequence::default();
    plate.uv_constraints(&mut seq);
    assert_eq!(seq.length(), 4);
    plate.destroy();
    assert!(plate.is_done() && plate.uv_box().is_err());
}

#[test]
fn unsolved_plate_has_no_polynomial_part() {
    let mut plate = corners();
    plate.solve_ti(4, 1.0, &ProgressRange::default()).unwrap();
    plate.load_pinpoint(&ppc(0.5, 0.5, 2.0));
    assert!(!plate.is_done());
    assert_eq!(
        plate.evaluate(Xy::new(0.5, 0.5)).unwrap(),
        Xyz::new(0.0, 0.0, 0.0)
    );
    let fresh = corners();
    assert_eq!(fresh.coef_pol().unwrap().get().unwrap().array2().len(), 0);
    // A refused solve still sets the order, and CoefPol then reads the missing solution.
    let mut refused = corners();
    refused
        .solve_ti(3, 1.0e9, &ProgressRange::default())
        .unwrap();
    assert_eq!(
        refused.coef_pol().err().unwrap().exception_type(),
        "Standard_NullObject"
    );
    let mut empty = PlatePlate::default();
    empty.solve_ti(4, 1.0, &ProgressRange::default()).unwrap();
    assert!(!empty.is_done());
}

#[test]
fn shaped_constraints_refuse_bad_sizes() {
    let mut samples: Sequence<PlatePinpointConstraint> = Sequence::default();
    samples.append(ppc(0.1, 0.5, 0.0));
    let error = PlateSampledCurveConstraint::new(&samples, 2).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_DimensionMismatch");
    assert!(PlateGlobalTranslationConstraint::new(&Sequence::default()).is_err());
    assert_eq!(
        PlatePinpointConstraint::origin(),
        PlatePinpointConstraint::default()
    );
    let d1 = PlateD1::new(Xyz::new(1.0, 0.0, 0.0), Xyz::new(0.0, 1.0, 0.0));
    assert_eq!((d1.du().x(), d1.dv().y()), (1.0, 1.0));
    let g1 = PlateGtoCConstraint::g1(Xy::new(0.5, 0.5), &d1, &d1);
    assert_eq!(g1.nb_ppc(), 2);
    assert!(g1.get_ppc(9).is_err() && g1.get_ppc(-1).is_err());
    assert_eq!(g1.d1_surf_init(), d1);
    let free = PlateFreeGtoCConstraint::g1(Xy::new(0.5, 0.5), &d1, &d1, 1.0, 0).unwrap();
    assert_eq!((free.nb_ppc(), free.nb_lsc()), (0, 2));
    assert!(free.lsc(4).is_err() && free.get_ppc(5).is_err());
    let parallel = PlateFreeGtoCConstraint::g1(Xy::new(0.5, 0.5), &d1, &d1, 0.5, 0);
    assert_eq!(
        parallel.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
}
