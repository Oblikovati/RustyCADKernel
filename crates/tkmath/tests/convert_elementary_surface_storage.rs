//! Native-backed construction and finalization behavior for elementary surface storage.

use rustycad_tkmath::convert::ElementarySurfaceToBSplineSurface as Surface;
use rustycad_tkmath::gp::Point3;

#[test]
fn finalization_trims_initialized_storage_and_is_idempotent() {
    let mut surface = initialized_surface();
    surface.set_actual_counts(2, 2, 2, 1);
    surface.set_degrees(3, 2);
    surface.set_periodicity(true, false);
    surface.finalize().unwrap();
    assert_trimmed_state(&surface);
    surface.finalize().unwrap();
    assert_trimmed_state(&surface);
}

#[test]
fn finalization_failure_keeps_native_ordered_prior_trims() {
    let mut surface = initialized_surface();
    surface.set_actual_counts(2, 2, -1, 1);
    let error = surface.finalize().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfMemory");
    assert_eq!(
        (surface.poles().nb_rows(), surface.poles().nb_columns()),
        (2, 2)
    );
    assert_eq!(
        (surface.weights().nb_rows(), surface.weights().nb_columns()),
        (2, 2)
    );
    assert_eq!(surface.u_knots().len(), 3);
    assert_eq!(surface.u_multiplicities().len(), 3);
    assert_eq!(surface.v_knots().len(), 2);
}

#[test]
fn rebased_protected_arrays_return_out_of_range_instead_of_panicking() {
    let mut surface = initialized_surface();
    surface.set_actual_counts(2, 2, 2, 1);
    rebase_arrays(&mut surface);
    assert_storage_index_error(surface.pole(1, 1).unwrap_err(), "Pole");
    assert_storage_index_error(surface.weight(1, 1).unwrap_err(), "Weight");
    assert_storage_index_error(surface.u_knot(1).unwrap_err(), "UKnot");
    assert_storage_index_error(surface.v_knot(1).unwrap_err(), "VKnot");
    assert_storage_index_error(surface.u_multiplicity(1).unwrap_err(), "UMultiplicity");
    assert_storage_index_error(surface.v_multiplicity(1).unwrap_err(), "VMultiplicity");
}

#[test]
fn active_counts_are_checked_before_custom_array_bounds() {
    let mut surface = initialized_surface();
    surface.set_actual_counts(2, 2, 2, 1);
    rebase_arrays(&mut surface);
    let error = surface.pole(3, 1).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(
        error
            .message()
            .contains("Pole: Index out of range: 3 not in 1..=2")
    );
}

#[test]
fn zero_capacity_is_constructible_but_negative_capacity_is_rejected_safely() {
    let empty = Surface::with_capacity(0, 0, 0, 0, 0, 0).unwrap();
    assert_eq!(
        (empty.poles().nb_rows(), empty.poles().nb_columns()),
        (0, 0)
    );
    let error = Surface::with_capacity(-1, 1, 1, 1, 1, 1).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfMemory");
}

#[test]
fn finalize_restores_one_based_bounds_after_rebasing() {
    let mut surface = initialized_surface();
    rebase_arrays(&mut surface);
    surface.set_actual_counts(1, 1, 1, 1);
    surface.finalize().unwrap();
    assert_eq!(
        (surface.poles().lower_row(), surface.poles().lower_col()),
        (1, 1)
    );
    assert_eq!(
        (surface.weights().lower_row(), surface.weights().lower_col()),
        (1, 1)
    );
    assert_eq!(surface.u_knots().lower(), 1);
    assert_eq!(surface.v_knots().lower(), 1);
    assert_eq!(surface.u_multiplicities().lower(), 1);
    assert_eq!(surface.v_multiplicities().lower(), 1);
}

#[test]
fn scalar_accessors_check_custom_array_extents_after_active_counts() {
    let mut surface = initialized_surface();
    surface.set_actual_counts(2, 2, 2, 1);
    surface
        .poles_mut()
        .resize_trim_bounds(1, 1, 1, 1, true)
        .unwrap();
    let error = surface.pole(2, 2).unwrap_err();
    assert_storage_index_error(error, "Pole");
}

fn rebase_arrays(surface: &mut Surface) {
    surface
        .poles_mut()
        .resize_trim_bounds(3, 6, 3, 5, true)
        .unwrap();
    surface
        .weights_mut()
        .resize_trim_bounds(3, 6, 3, 5, true)
        .unwrap();
    surface.u_knots_mut().resize_bounds(3, 5, true).unwrap();
    surface.v_knots_mut().resize_bounds(3, 4, true).unwrap();
    surface
        .u_multiplicities_mut()
        .resize_bounds(3, 5, true)
        .unwrap();
    surface
        .v_multiplicities_mut()
        .resize_bounds(3, 4, true)
        .unwrap();
}

fn assert_storage_index_error(error: rustycad_tkernel::standard::Failure, accessor: &str) {
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(error.message().contains(accessor));
    assert!(error.message().contains("expected"));
}

fn initialized_surface() -> Surface {
    let mut surface = Surface::with_capacity(4, 3, 3, 2, 2, 1).unwrap();
    for u in 1..=4 {
        for v in 1..=3 {
            surface.poles_mut()[(u, v)] =
                Point3::new(f64::from(10 * u + v), f64::from(u), f64::from(v));
            surface.weights_mut()[(u, v)] = f64::from(100 * u + v);
        }
    }
    initialize_knots(&mut surface);
    surface
}

fn initialize_knots(surface: &mut Surface) {
    for i in 1..=3 {
        surface.u_knots_mut()[i] = f64::from(i * 10);
        surface.u_multiplicities_mut()[i] = 100 + i;
    }
    for i in 1..=2 {
        surface.v_knots_mut()[i] = f64::from(i * 20);
        surface.v_multiplicities_mut()[i] = 200 + i;
    }
}

fn assert_trimmed_state(surface: &Surface) {
    assert_eq!((surface.nb_u_poles(), surface.nb_v_poles()), (2, 2));
    assert_eq!((surface.nb_u_knots(), surface.nb_v_knots()), (2, 1));
    assert_eq!((surface.u_degree(), surface.v_degree()), (3, 2));
    assert!(surface.is_u_periodic());
    assert!(!surface.is_v_periodic());
    assert_eq!(surface.poles()[(2, 2)], Point3::new(22.0, 2.0, 2.0));
    assert_eq!(surface.weights()[(2, 2)], 202.0);
    assert_eq!(surface.u_knots()[2], 20.0);
    assert_eq!(surface.u_multiplicities()[2], 102);
    assert_eq!(surface.v_knots()[1], 20.0);
    assert_eq!(surface.v_multiplicities()[1], 201);
}
