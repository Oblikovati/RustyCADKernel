// EmplaceValue reserves every destination block before invoking constructors.
#include <NCollection_DynamicArray.hxx>
#include <NCollection_IncAllocator.hxx>
#include <cstdint>
#include <iostream>
#include <stdexcept>

struct NestedAllocation {
  std::uintptr_t nested;
  NestedAllocation() : nested(0) {}
  explicit NestedAllocation(const occ::handle<NCollection_IncAllocator>& alloc)
      : nested(reinterpret_cast<std::uintptr_t>(alloc->Allocate(8))) {}
};

static occ::handle<NCollection_IncAllocator> THE_ALLOCATOR;
static bool RECORD_GAP_ALLOCATIONS = false;
struct MultiBlockProbe {
  std::uintptr_t nested;
  MultiBlockProbe()
      : nested(RECORD_GAP_ALLOCATIONS
                   ? reinterpret_cast<std::uintptr_t>(THE_ALLOCATOR->Allocate(8))
                   : 0) {}
  explicit MultiBlockProbe(const occ::handle<NCollection_IncAllocator>& alloc)
      : nested(reinterpret_cast<std::uintptr_t>(alloc->Allocate(8))) {}
};

struct ThrowsAfterGaps {
  static int defaults;
  static int destroyed;
  ThrowsAfterGaps() { ++defaults; }
  explicit ThrowsAfterGaps(bool) { throw std::runtime_error("requested"); }
  ~ThrowsAfterGaps() { ++destroyed; }
};
int ThrowsAfterGaps::defaults = 0;
int ThrowsAfterGaps::destroyed = 0;

int main() {
  auto alloc = new NCollection_IncAllocator(4096);
  NCollection_DynamicArray<NestedAllocation> addresses(
      2, NCollection_OccAllocator<NestedAllocation>(alloc));
  addresses.Append(NestedAllocation());
  const auto& value = addresses.EmplaceValue(2, alloc);
  std::cout << "target_block_before_constructor="
            << (reinterpret_cast<std::uintptr_t>(&value) < value.nested) << '\n';

  NCollection_DynamicArray<MultiBlockProbe> multiple(2, NCollection_OccAllocator<MultiBlockProbe>(alloc));
  multiple.Append(MultiBlockProbe());
  THE_ALLOCATOR = alloc;
  RECORD_GAP_ALLOCATIONS = true;
  const auto& finalValue = multiple.EmplaceValue(6, alloc);
  RECORD_GAP_ALLOCATIONS = false;
  std::cout << "all_target_blocks_before_first_gap="
            << (reinterpret_cast<std::uintptr_t>(&finalValue) < multiple(1).nested) << '\n';

  {
    NCollection_DynamicArray<ThrowsAfterGaps> values(2);
    values.Append(ThrowsAfterGaps());
    const int before = ThrowsAfterGaps::destroyed;
    try {
      values.EmplaceValue(4, true);
    } catch (const std::runtime_error&) {
    }
    std::cout << "failed_target_length=" << values.Length() << '\n';
    std::cout << "gap_defaults_constructed=" << ThrowsAfterGaps::defaults - 1 << '\n';
    std::cout << "gap_defaults_destroyed=" << ThrowsAfterGaps::destroyed - before << '\n';
  }
}
