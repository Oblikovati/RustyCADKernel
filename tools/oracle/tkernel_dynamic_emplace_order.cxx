// A nested allocation observes whether EmplaceAppend reserved the element first.
#include <NCollection_DynamicArray.hxx>
#include <NCollection_IncAllocator.hxx>
#include <cstdint>
#include <iostream>

struct NestedAllocation {
  std::uintptr_t address;
  explicit NestedAllocation(const occ::handle<NCollection_IncAllocator>& allocator)
    : address(reinterpret_cast<std::uintptr_t>(allocator->Allocate(8))) {}
};
int main() {
  occ::handle<NCollection_IncAllocator> allocator = new NCollection_IncAllocator(4096);
  NCollection_DynamicArray<NestedAllocation> array(
    4, NCollection_OccAllocator<NestedAllocation>(allocator));
  const NestedAllocation& element = array.EmplaceAppend(allocator);
  const auto stored = reinterpret_cast<std::uintptr_t>(&element);
  std::cout << "element_before_nested=" << (stored < element.address) << '\n';
}
