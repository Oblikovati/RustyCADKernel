//! Unit tests of the tools of the corner of n edges; the whole corner is checked against
//! the pinned kernel by tests/fil_builder.rs and the fl_make_fillet probe.
use super::batten::calcul_droite;
use super::statics::remove_sd;
use super::{indices, slot};
use crate::chfi_ds::{Stripe, SurfData, lock, shared};
use rustycad_tkmath::gp::Point2;

#[test]
fn indices_wrap_around_the_vertex() {
    assert_eq!(indices(4, 0), (1, 3));
    assert_eq!(indices(4, 2), (3, 1));
    assert_eq!(indices(4, 3), (0, 2));
    // Past the last edge (a free border parsed at `nedge`), as natively.
    assert_eq!(indices(4, 4), (5, 3));
}

#[test]
fn slots_out_of_the_array_fail_as_out_of_range() {
    let mut v = vec![0; 3];
    *slot(&mut v, 2).unwrap() = 7;
    assert_eq!(v, [0, 0, 7]);
    assert_eq!(
        slot(&mut v, 3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        slot(&mut v, -1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn remove_sd_removes_a_range_and_refuses_a_reversed_one() {
    let stripe = shared(Stripe::new());
    let seq = shared((0..4).map(|_| shared(SurfData::new())).collect::<Vec<_>>());
    let kept = lock(&seq)[3].clone();
    *lock(&stripe).change_set_of_surf_data() = Some(seq.clone());
    remove_sd(&stripe, 1, 3).unwrap();
    assert_eq!(lock(&seq).len(), 1);
    assert!(std::sync::Arc::ptr_eq(&lock(&seq)[0], &kept));
    assert_eq!(
        remove_sd(&stripe, 1, 2).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    remove_sd(&stripe, 1, 1).unwrap();
    // An empty sequence is left alone whatever the range.
    remove_sd(&stripe, 3, 1).unwrap();
}

#[test]
fn calcul_droite_is_the_segment_of_the_direction() {
    let pc = calcul_droite(Point2::new(1.0, 2.0), 3.0, 4.0).unwrap();
    let c = pc.get().unwrap();
    assert_eq!(c.first_parameter(), 0.0);
    assert_eq!(c.last_parameter(), 5.0);
    let end = c.value(5.0).unwrap();
    assert!((end.x() - 4.0).abs() < 1e-12 && (end.y() - 6.0).abs() < 1e-12);
    assert_eq!(
        calcul_droite(Point2::new(0.0, 0.0), 0.0, 0.0)
            .unwrap_err()
            .exception_type(),
        "Standard_ConstructionError"
    );
}
