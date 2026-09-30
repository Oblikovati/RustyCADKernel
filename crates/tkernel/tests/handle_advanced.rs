//! Full advanced handle suite, preserving all 51 generated inheritance levels.
#![allow(non_snake_case, clippy::overly_complex_bool_expr)]
include!("handles/hierarchy.rs");
use rustycad_tkernel::ncollection::{
    AllocatorObject, HeapAllocatorObject, IncrementalAllocatorObject,
};
use rustycad_tkernel::standard::{HandleView, RuntimeType};
use std::{any::TypeId, hint::black_box, time::Instant};
fn test_function(object: HandleView<'_, StandardTransient>) -> bool {
    !object.is_null()
}
fn test_overload1(_: &Handle<StandardTransient>) -> i32 {
    1
}
fn test_overload2(_: &Handle<TransientRoot>) -> i32 {
    2
}
fn require_transient_struct<T: Transient + RuntimeTyped>() {}
#[test]
fn HandleAdvancedTest__CompilerSpecificBehavior() {
    let root = Handle::new(TransientRoot::default());
    assert!(!root.is_null());
    let _const_root = &root;
    let transient = root.cast::<StandardTransient>();
    assert!(test_function(root.view::<StandardTransient>()));
    assert_eq!(2, test_overload2(&root));
    assert_eq!(1, test_overload1(&transient));
    let transient2 = root.view::<StandardTransient>();
    assert!(!transient2.is_null());
}
#[test]
fn HandleAdvancedTest__DeepHierarchyRTTI() {
    let descriptors: [&RuntimeType; 6] = [
        QaClass00_50::static_type(),
        QaClass10_50::static_type(),
        QaClass20_50::static_type(),
        QaClass30_50::static_type(),
        QaClass40_50::static_type(),
        QaClass50_50::static_type(),
    ];
    let handle = Handle::new(QaClass40_50::default()).cast::<QaClass00_50>();
    assert_eq!("QaClass40_50", handle.dynamic_type().unwrap().name());
    for descriptor in &descriptors[..5] {
        assert!(handle.is_kind(descriptor));
    }
    assert!(!handle.is_kind(descriptors[5]));
    for name in [
        "QaClass00_50",
        "QaClass10_50",
        "QaClass20_50",
        "QaClass30_50",
        "QaClass40_50",
    ] {
        assert!(handle.is_kind_name(name));
    }
    assert!(!handle.is_kind_name("QaClass50_50"));
    for descriptor in &descriptors[..4] {
        assert!(!handle.is_instance(descriptor));
    }
    assert!(handle.is_instance(descriptors[4]));
    assert!(!handle.is_instance(descriptors[5]));
}
#[test]
fn HandleAdvancedTest__TypeInfoCompatibility() {
    let handle = Handle::new(QaClass40_50::default());
    let type_info: TypeId = handle.rust_type_id().unwrap();
    for ancestor in [
        TypeId::of::<QaClass00_50>(),
        TypeId::of::<QaClass10_50>(),
        TypeId::of::<QaClass20_50>(),
        TypeId::of::<QaClass30_50>(),
    ] {
        assert_ne!(type_info, ancestor);
    }
    assert_eq!(type_info, TypeId::of::<QaClass40_50>());
    assert_ne!(type_info, TypeId::of::<QaClass50_50>());
    let cpp_type: TypeId = handle.rust_type_id().unwrap();
    assert_ne!(cpp_type, TypeId::of::<QaClass00_50>());
    assert_eq!(cpp_type, TypeId::of::<QaClass40_50>());
    let anon = QaClass50_50Anon::default();
    let named = qa_namespace::QaClass50_50::default();
    assert_ne!(
        std::any::Any::type_id(&anon),
        std::any::Any::type_id(&named)
    );
    assert_ne!(std::any::Any::type_id(&anon), TypeId::of::<QaClass50_50>());
    assert_ne!(std::any::Any::type_id(&named), TypeId::of::<QaClass50_50>());
    require_transient_struct::<StandardTransient>();
}
#[test]
fn HandleAdvancedTest__AllocatorHandlePerformance() {
    let base = Handle::new(IncrementalAllocatorObject::default()).cast::<AllocatorObject>();
    assert!(!base.is_null());
    let started: Instant = Instant::now();
    for _ in 0..1000 {
        assert!(!base.cast::<IncrementalAllocatorObject>().is_null());
    }
    black_box(started.elapsed().as_secs_f64() * 1_000_000.0);
    let started: Instant = Instant::now();
    for _ in 0..1000 {
        assert!(base.cast::<HeapAllocatorObject>().is_null());
    }
    black_box(started.elapsed().as_secs_f64() * 1_000_000.0);
}
#[test]
fn HandleAdvancedTest__HandleArrayOperations() {
    let handles: Vec<Handle<QaClass00_50>> = vec![
        Handle::new(QaClass00_50::default()),
        Handle::new(QaClass10_50::default()).cast(),
        Handle::new(QaClass20_50::default()).cast(),
        Handle::new(QaClass30_50::default()).cast(),
        Handle::new(QaClass40_50::default()).cast(),
    ];
    assert_eq!(5, handles.len());
    for handle in &handles {
        assert!(!handle.is_null());
        assert!(handle.is_kind_name("QaClass00_50"));
        let cast: Handle<QaClass00_50> = handle.clone();
        assert!(!cast.is_null());
        assert_eq!(handle.as_ptr(), cast.as_ptr());
    }
    assert!(!handles[4].cast::<QaClass40_50>().is_null());
    assert!(handles[0].cast::<QaClass40_50>().is_null());
}
#[test]
fn HandleAdvancedTest__ConstHandleOperations() {
    let non_const = Handle::new(QaClass30_50::default());
    let const_handle = &non_const;
    assert_eq!(non_const.as_ptr(), const_handle.as_ptr());
    let const_pointer = const_handle.as_ptr();
    let non_const_pointer = non_const.as_ptr();
    assert_eq!(const_pointer, non_const_pointer);
    let const_base = const_handle.view::<QaClass00_50>();
    let non_const_base = non_const.cast::<QaClass00_50>();
    assert_eq!(const_base.as_ptr(), non_const_base.as_ptr());
    assert!(*const_handle == non_const);
    assert!(const_base == non_const_base);
    assert!(!(*const_handle != non_const));
}
#[test]
fn HandleAdvancedTest__WeakReferenceSimulation() {
    let raw_pointer: *const QaClass20_50;
    {
        let handle = Handle::new(QaClass20_50::default());
        raw_pointer = handle.as_ptr();
        assert!(!raw_pointer.is_null());
        assert!(!handle.is_null());
        assert_eq!(raw_pointer, handle.as_ptr());
    }
    let mut handles: Vec<Handle<QaClass20_50>> = Vec::new();
    let mut found_different: bool = false;
    for _ in 0..10 {
        if found_different {
            break;
        }
        handles.push(Handle::new(QaClass20_50::default()));
        if handles.last().unwrap().as_ptr() != raw_pointer {
            found_different = true;
        }
    }
    // Retain upstream's explicitly nonrestrictive allocator-address assertion.
    assert!(found_different || !found_different);
}
