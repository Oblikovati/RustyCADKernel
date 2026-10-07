use super::*;
use crate::chfi_ds::{Spine, shared};

#[test]
fn reset_keeps_the_last_pcurve_as_natively() {
    let mut stripe = Stripe::new();
    let spine: SpineHandle = shared(Spine::new());
    *stripe.change_spine() = Some(spine);
    *stripe.change_set_of_surf_data() = Some(shared(Vec::new()));
    stripe.set_orientation_at_end(Orientation::Reversed, false);
    stripe.reset().unwrap();
    assert!(stripe.set_of_surf_data().is_none());
    assert_eq!(stripe.last_pcurve_orientation(), Orientation::Forward);
}

#[test]
fn reset_without_spine_raises_null_object() {
    let mut stripe = Stripe::new();
    let failure = stripe.reset().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
}

#[test]
fn per_end_and_per_face_accessors_agree() {
    let mut stripe = Stripe::new();
    stripe.set_parameters(true, 1.0, 2.0);
    stripe.change_last_parameters(3.0, 4.0);
    stripe.set_curve(7, true);
    stripe.change_last_curve(8);
    stripe.set_orientation_on_face(Orientation::Reversed, 2);
    stripe.set_index_point(9, true, 1);
    assert_eq!(stripe.first_parameters(), (1.0, 2.0));
    assert_eq!(stripe.parameters(false), (3.0, 4.0));
    assert_eq!((stripe.curve(true), stripe.curve(false)), (7, 8));
    assert_eq!(stripe.orientation_on_face2(), Orientation::Reversed);
    assert_eq!(stripe.orientation_on_face(1), Orientation::Forward);
    assert_eq!(stripe.index_first_point_on_s1(), 9);
    assert_eq!(stripe.index_point(true, 1), 9);
}

#[test]
fn choix_solid_and_pcurves_round_trip() {
    let mut stripe = Stripe::new();
    stripe.set_choix(-1);
    stripe.set_solid_index(3);
    stripe.in_ds(false, 1);
    assert_eq!((stripe.choix(), stripe.solid_index()), (-1, 3));
    assert_eq!((stripe.is_in_ds(true), stripe.is_in_ds(false)), (0, 1));
    assert!(stripe.pcurve(true).is_null() && stripe.change_pcurve(false).is_null());
    assert!(stripe.first_pcurve().is_null() && stripe.last_pcurve().is_null());
}
