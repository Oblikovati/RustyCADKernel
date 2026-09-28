// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Handles of abstract classes written as traits (`Handle<dyn Trait>`): RTTI,
//! down casts, identity and shared mutation through interior mutability.
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use std::sync::RwLock;

trait FakeShape: Transient {
    fn area(&self) -> f64;
    fn scale(&self, factor: f64);
}
impl_transient_interface!(dyn FakeShape, "Fake_Shape", StandardTransient);

trait FakeRound: FakeShape {
    fn radius(&self) -> f64;
}
impl_transient_interface!(dyn FakeRound, "Fake_Round", dyn FakeShape);

trait FakeUnrelated: Transient {}
impl_transient_interface!(dyn FakeUnrelated, "Fake_Unrelated", StandardTransient);

#[derive(Default)]
struct FakeDisk {
    parent: StandardTransient,
    radius: RwLock<f64>,
}
impl_transient!(FakeDisk, "Fake_Disk", dyn FakeRound, parent, interfaces: [dyn FakeShape, dyn FakeRound]);
impl FakeShape for FakeDisk {
    fn area(&self) -> f64 {
        let radius: f64 = *self.radius.read().unwrap();
        radius * radius * 3.0
    }
    fn scale(&self, factor: f64) {
        *self.radius.write().unwrap() *= factor;
    }
}
impl FakeRound for FakeDisk {
    fn radius(&self) -> f64 {
        *self.radius.read().unwrap()
    }
}

#[derive(Default)]
struct FakeSquare {
    parent: StandardTransient,
}
impl_transient!(FakeSquare, "Fake_Square", dyn FakeShape, parent, interfaces: [dyn FakeShape]);
impl FakeShape for FakeSquare {
    fn area(&self) -> f64 {
        1.0
    }
    fn scale(&self, _factor: f64) {}
}

fn disk(radius: f64) -> Handle<FakeDisk> {
    Handle::new(FakeDisk {
        parent: StandardTransient::default(),
        radius: RwLock::new(radius),
    })
}

#[test]
fn mutation_through_concrete_handle_is_visible_through_interface_handle() {
    let concrete: Handle<FakeDisk> = disk(2.0);
    let shape: Handle<dyn FakeShape> = concrete.cast::<dyn FakeShape>();
    assert!(!shape.is_null());
    assert_eq!(shape.area(), 12.0);
    concrete.scale(0.5);
    assert_eq!(shape.area(), 3.0);
    shape.scale(4.0);
    assert_eq!(concrete.radius(), 4.0);
    assert_eq!(concrete.ref_count(), 2);
}

#[test]
fn interface_handles_down_cast_and_keep_identity() {
    let concrete: Handle<FakeDisk> = disk(1.0);
    let shape: Handle<dyn FakeShape> = concrete.cast();
    let round: Handle<dyn FakeRound> = shape.cast();
    let back: Handle<FakeDisk> = round.cast();
    assert!(!round.is_null() && !back.is_null());
    assert!(back == concrete && round == shape);
    assert_eq!(back.as_ptr(), concrete.as_ptr());
    assert_eq!(shape.object_address(), concrete.object_address());
    assert!(shape.cast::<FakeSquare>().is_null());
    assert!(shape.cast::<dyn FakeUnrelated>().is_null());
    assert!(shape.cast::<StandardTransient>().is_some());
}

#[test]
fn interface_descriptors_form_the_native_hierarchy() {
    let square: Handle<dyn FakeShape> = Handle::new(FakeSquare::default()).cast();
    assert!(square.is_kind(<dyn FakeShape>::static_type()));
    assert!(!square.is_kind(<dyn FakeRound>::static_type()));
    assert!(square.cast::<dyn FakeRound>().is_null());
    let round: Handle<dyn FakeRound> = disk(1.0).cast();
    assert!(round.is_kind(<dyn FakeShape>::static_type()));
    assert!(round.is_kind_name("Standard_Transient"));
    assert!(round.is_instance_name("Fake_Disk"));
    assert_eq!(
        round.dynamic_type().unwrap().parent().unwrap().name(),
        "Fake_Round"
    );
}

#[test]
fn null_interface_handles_and_views() {
    let null: Handle<dyn FakeShape> = Handle::default();
    assert!(null.is_null() && null.get().is_none());
    assert!(null.cast::<FakeDisk>().is_null());
    assert!(null.object_address().is_null());
    let concrete: Handle<FakeDisk> = disk(3.0);
    let view = concrete.view::<dyn FakeRound>();
    assert!(!view.is_null());
    assert_eq!(view.get().unwrap().radius(), 3.0);
    assert!(concrete.view::<dyn FakeUnrelated>().is_null());
    assert_eq!(concrete.ref_count(), 1);
}

#[test]
fn interface_handles_share_mutation_across_threads() {
    let shape: Handle<dyn FakeShape> = disk(1.0).cast();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let local: Handle<dyn FakeShape> = shape.clone();
            scope.spawn(move || local.scale(2.0));
        }
    });
    assert_eq!(shape.cast::<dyn FakeRound>().radius(), 16.0);
    assert_eq!(shape.ref_count(), 1);
}
