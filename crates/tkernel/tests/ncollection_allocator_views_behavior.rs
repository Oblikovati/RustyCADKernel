// Copyright (c) 2002-2023 OPEN CASCADE SAS
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

//! Buffer, typed allocators, NCollection_Handle and NCollection_Shared.
// Typed allocators mirror the native raw pointer API, which Rust marks unsafe.
#![allow(unsafe_code)]
use rustycad_tkernel::{
    ncollection::{
        AccumulatingAllocatorObject, AlignedAllocatorObject, AllocatorObject, AllocatorTypes,
        BaseAllocator, Buffer, IncrementalAllocatorObject, ObjectHandle, OccAllocator, Shared,
        StandardAllocator,
    },
    standard::{Handle, StandardTransient, dump::DumpStream},
};

#[test]
fn buffer_adopts_data_and_reports_native_pointer_state() {
    let allocator = BaseAllocator::incremental(0);
    let data = allocator.allocate(4).unwrap();
    let address = data.as_ptr();
    let mut buffer = Buffer::with_data(Some(allocator.clone()), data);
    assert_eq!((buffer.size(), buffer.data_ptr()), (4, address));
    buffer.as_mut_slice().copy_from_slice(&[1, 2, 3, 4]);
    assert_eq!(buffer.as_slice(), &[1, 2, 3, 4]);
    buffer.set_allocator(None).unwrap();
    assert!(buffer.data_ptr().is_null() && buffer.is_empty());
    assert!(!buffer.allocate(8).unwrap()); // upstream dereferences the null allocator here
    assert!(Buffer::new(None, 0).unwrap().is_empty());
    let shared = Handle::new(Buffer::new(Some(BaseAllocator::common()), 2).unwrap());
    assert!(shared.is_instance_name("NCollection_Buffer"));
    assert!(shared.is_kind_name("Standard_Transient"));
}

#[test]
fn allocator_objects_chain_to_the_base_allocator_descriptor() {
    let accumulating = Handle::new(AccumulatingAllocatorObject::with_default_block_size());
    assert!(accumulating.is_instance_name("NCollection_AccAllocator"));
    assert!(accumulating.is_kind_name("NCollection_BaseAllocator"));
    assert!(AccumulatingAllocatorObject::new(0).is_err());
    let aligned = Handle::new(AlignedAllocatorObject::new(32));
    assert!(aligned.is_instance_name("NCollection_AlignedAllocator"));
    assert_eq!(
        aligned.cast::<AllocatorObject>().resource(),
        aligned.resource()
    );
    let incremental = IncrementalAllocatorObject::new(4096);
    assert_ne!(
        incremental.resource(),
        IncrementalAllocatorObject::default().resource()
    );
}

#[test]
fn buffer_free_raises_like_the_accumulating_allocator() {
    // Allocate starts with Free(nullptr), which NCollection_AccAllocator refuses.
    let accumulating = BaseAllocator::accumulating(64).unwrap();
    let failure = Buffer::new(Some(accumulating.clone()), 8).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    let mut buffer = Buffer::new(Some(BaseAllocator::common()), 8).unwrap();
    assert!(buffer.set_allocator(Some(accumulating.clone())).is_ok()); // frees non-null data
    assert!(buffer.set_allocator(None).is_err());
    assert_eq!(buffer.allocator(), Some(&accumulating));
}

#[test]
fn buffer_dump_uses_short_pointer_info() {
    let buffer = Buffer::new(Some(BaseAllocator::common()), 16).unwrap();
    let mut stream = DumpStream::default();
    buffer.dump_json(&mut stream, -1).unwrap();
    let text = stream.text().to_owned();
    let data = format!("\"Data\": \"0xx{:x}\"", buffer.data_ptr() as usize);
    assert!(text.starts_with(&format!("{data}, \"Size\": 16, \"Allocator\": \"0xx")));
    let mut empty = DumpStream::default();
    Buffer::new(None, 0)
        .unwrap()
        .dump_json(&mut empty, -1)
        .unwrap();
    assert_eq!(
        empty.text(),
        "\"Data\": \"\", \"Size\": 0, \"Allocator\": \"\""
    );
}

#[test]
fn occ_allocator_null_handle_differs_from_common() {
    let null = OccAllocator::<i32>::default();
    assert!(null.allocator().is_none());
    assert_ne!(null, OccAllocator::<i32>::new(BaseAllocator::common()));
    assert_eq!(null, OccAllocator::<f64>::with_allocator(None));
    let mut other = OccAllocator::<u8>::new(BaseAllocator::global_heap());
    other.assign(&null);
    assert!(other.allocator().is_none());
    other.set_allocator(Some(BaseAllocator::common()));
    assert_eq!(other.allocator(), Some(&BaseAllocator::common()));
    assert_eq!(OccAllocator::<u64>::default().max_size(), usize::MAX / 8);
    assert_eq!(OccAllocator::<()>::default().max_size(), usize::MAX);
}

#[test]
fn occ_allocator_raw_elements_round_trip() {
    let allocator = OccAllocator::<u32>::new(BaseAllocator::incremental(0));
    let storage = allocator.allocate(3).unwrap();
    // SAFETY: storage holds three u32 slots from this allocator.
    unsafe {
        allocator.construct(storage, 7_u32);
        assert_eq!(*storage.as_ptr(), 7);
        allocator.destroy(storage);
        allocator.deallocate(storage, 3).unwrap();
    }
    let mut item = 5_u32;
    assert_eq!(allocator.address(&item), &item as *const u32);
    assert_eq!(allocator.address_mut(&mut item), &mut item as *mut u32);
    assert!(allocator.allocate(usize::MAX).is_none());
}

#[test]
fn standard_allocator_is_stateless_and_reallocates() {
    let ints = StandardAllocator::<i32>::from_base(&BaseAllocator::incremental(0));
    assert_eq!(ints, StandardAllocator::<f64>::new());
    let storage = ints.allocate(2).unwrap();
    // SAFETY: storage is a live two-element block of the standard policy.
    unsafe {
        storage.as_ptr().write(11);
        let grown = ints.reallocate(storage, 64).unwrap();
        assert_eq!(*grown.as_ptr(), 11);
        ints.deallocate(grown, 64).unwrap();
    }
    let mut vector = allocator_api2::vec::Vec::new_in(ints.rebind::<u16>());
    vector.extend([1_u16, 2, 3]);
    assert_eq!(vector.as_slice(), &[1, 2, 3]);
}

#[test]
fn object_handle_wraps_non_transient_objects() {
    let handle = ObjectHandle::new(vec![1, 2, 3]);
    assert_eq!(handle.get().map(Vec::len), Some(3));
    let base: Handle<StandardTransient> = handle.transient();
    assert_eq!(base.ref_count(), 2);
    assert!(base.is_instance_name("Standard_Transient"));
    let recovered = ObjectHandle::<Vec<i32>>::down_cast(&base);
    assert!(recovered == handle);
    assert!(ObjectHandle::<String>::down_cast(&base).is_null());
    assert!(ObjectHandle::<i32>::down_cast(&Handle::new(StandardTransient::default())).is_null());
    assert!(ObjectHandle::<i32>::default().get().is_none());
}

#[test]
fn shared_value_is_a_transient_holding_its_value() {
    let handle = Handle::new(Shared::new(vec![4_u8, 5]));
    assert_eq!(handle.get().map(|shared| shared.len()), Some(2));
    assert!(handle.is_instance_name("Standard_Transient"));
    assert_eq!(handle.ref_count(), 1);
    let mut local: Shared<Vec<u8>> = Shared::default();
    local.push(9);
    assert_eq!(local.into_inner(), vec![9]);
}
