//! Unit tests of `XSControl_Controller` and `XSControl_Vars`.
use super::test_norm::{fake_controller, vertex_compound};
use super::*;
use crate::if_select::IFSelectIntParam;
use crate::interface::test_norm::fake_entity;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkmath::gp::{Point2, Point3};

#[test]
fn names_keep_the_empty_ones() {
    let ctl = fake_controller("FAKE-LONG", "fk");
    assert_eq!(ctl.name(false).as_bytes(), b"FAKE-LONG");
    assert_eq!(ctl.name(true).as_bytes(), b"fk");
    ctl.set_names("", "f2");
    assert_eq!(ctl.name(false).as_bytes(), b"FAKE-LONG");
    assert_eq!(ctl.name(true).as_bytes(), b"f2");
}

#[test]
fn base_traces_the_precision_statics() {
    let ctl = fake_controller("FAKE-TRACE", "ft");
    let (items, uses) = ctl.controller_base().params();
    assert_eq!(items.len(), 4);
    assert_eq!(uses, vec![5, 5, 6, 6]);
}

#[test]
fn record_keeps_the_first_controller_of_a_kind() {
    let first = fake_controller("FAKE-REC-1", "fr1");
    first.auto_record().unwrap();
    assert!(XSControlControllerBase::recorded("fr1") == first);
    assert!(XSControlControllerBase::recorded("FAKE-REC-1") == first);
    let second = fake_controller("FAKE-REC-2", "fr2");
    second.record("fr1").unwrap();
    assert!(XSControlControllerBase::recorded("fr1") == first);
    assert!(XSControlControllerBase::recorded("no-such-norm").is_null());
}

#[test]
fn unbound_controller_cannot_record() {
    let base = XSControlControllerBase::new("FAKE-UNBOUND", "fu").unwrap();
    assert!(base.this().is_err());
}

#[test]
fn modes_of_writing_and_their_helps() {
    let ctl = fake_controller("FAKE-MODES", "fm");
    assert!(ctl.mode_write_bounds(true).is_none());
    assert!(ctl.is_mode_write(42, true));
    ctl.set_mode_write(1, 3, true);
    ctl.set_mode_write_help(2, "second", true);
    ctl.set_mode_write_help(7, "ignored", true);
    assert_eq!(ctl.mode_write_bounds(true), Some((1, 3)));
    assert!(ctl.is_mode_write(3, true) && !ctl.is_mode_write(0, true));
    assert_eq!(ctl.mode_write_help(2, true).as_bytes(), b"second");
    assert!(ctl.mode_write_help(1, true).is_empty() && ctl.mode_write_help(9, true).is_empty());
    ctl.set_mode_write(2, 1, true);
    assert!(ctl.mode_write_bounds(true).is_none());
}

#[test]
fn session_items_and_the_applied_modifiers() {
    let ctl = fake_controller("FAKE-ITEMS", "fi");
    assert!(ctl.session_item("any").unwrap().is_null());
    let item: Handle<StandardTransient> = Handle::new(IFSelectIntParam::new()).cast();
    ctl.add_session_item(&item, "my-param", true);
    ctl.add_session_item(&item, "", false);
    assert!(ctl.session_item("my-param").unwrap() == item);
    assert!(ctl.session_item("other").is_err());
    assert_eq!(ctl.adaptor_session().extent(), 1);
    assert!(ctl.controller_base().adaptor_applied().is_empty());
}

#[test]
fn write_recognition_by_the_actor() {
    let ctl = fake_controller("FAKE-RECO", "fo");
    assert!(ctl.recognize_write_shape(&vertex_compound(), 0));
    assert!(ctl.recognize_write_transient(&fake_entity("e", &[]), 0));
}

#[test]
fn vars_hold_points_shapes_and_transients() {
    let vars = XSControlVars::new();
    let ent = fake_entity("v", &[]);
    vars.set("ent", &ent);
    assert!(vars.get(&mut String::from("ent")) == ent);
    assert!(vars.get(&mut String::from("none")).is_null());
    vars.set_point("p", &Point3::new(1.0, 2.0, 3.0));
    assert_eq!(
        vars.get_point(&mut String::from("p")),
        Some(Point3::new(1.0, 2.0, 3.0))
    );
    assert!(vars.get_point2d(&mut String::from("p")).is_none());
    vars.set_point2d("q", &Point2::new(4.0, 5.0));
    assert_eq!(
        vars.get_point2d(&mut String::from("q")),
        Some(Point2::new(4.0, 5.0))
    );
    let shape = vertex_compound();
    vars.set_shape("s", &shape);
    assert!(vars.get_shape(&mut String::from("s")) == shape);
    assert!(vars.get_shape(&mut String::from("ent")) == Shape::default());
    assert!(vars.get_curve(&mut String::from("p")).is_null());
}
