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
