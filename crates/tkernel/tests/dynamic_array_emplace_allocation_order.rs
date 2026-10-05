//! Native EmplaceAppend reserves its element block before invoking the constructor.
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray};

struct NestedAllocation {
    address: usize,
}

fn allocate_nested(allocator: &BaseAllocator) -> NestedAllocation {
    NestedAllocation {
        address: allocator
            .allocate_raw(8)
            .expect("eight-byte allocation")
            .as_ptr() as usize,
    }
}

#[test]
fn emplace_reserves_element_storage_before_nested_constructor_allocation() {
    let allocator = BaseAllocator::incremental(4096);
    let mut array = DynamicArray::with_allocator(4, Some(allocator.clone()));
    let element = array.emplace_append(|| allocate_nested(&allocator));
    let stored_address = element as *const NestedAllocation as usize;
    assert!(
        stored_address < element.address,
        "stored={stored_address}, nested={}: expected element allocation first in the same incremental block",
        element.address
    );
}
