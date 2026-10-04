#include <NCollection_AccAllocator.hxx>
#include <cstdint>
#include <iostream>

class AccAllocatorSubclassProbe : public NCollection_AccAllocator
{
public:
  using NCollection_AccAllocator::AlignedPtr;
  using NCollection_AccAllocator::AlignedSize;

  static size_t alignedSize(const size_t value)
  {
    return static_cast<size_t>(AlignedSize(value));
  }

  static void pointerOffsets(uint8_t* const base)
  {
    AlignedPtr aligned(base + 3);
    AlignedPtr moved = aligned + AlignedSize(1);
    std::cout << (static_cast<uint8_t*>(aligned) - base) << ','
              << (static_cast<uint8_t*>(moved) - base) << ',';
    AlignedPtr afterSub = (moved -= AlignedSize(1));
    std::cout << (static_cast<uint8_t*>(afterSub) - base) << ','
              << (static_cast<uint8_t*>(moved) - base) << ',';
    AlignedPtr afterAdd = (moved += AlignedSize(1));
    std::cout << (static_cast<uint8_t*>(afterAdd) - base) << ','
              << (static_cast<uint8_t*>(moved) - base) << ',';
    moved += AlignedSize(1);
    moved -= AlignedSize(1);
    std::cout << (static_cast<uint8_t*>(moved) - base) << '\n';
  }

  static bool defaultPointerIsNull()
  {
    return static_cast<void*>(AlignedPtr()) == nullptr;
  }
};

int main()
{
  const size_t sizes[] = {0, 1, 2, 3, 4, 5, SIZE_MAX};
  for (const size_t value : sizes)
  {
    std::cout << AccAllocatorSubclassProbe::alignedSize(value) << '\n';
  }
  alignas(4) uint8_t storage[16] = {};
  AccAllocatorSubclassProbe::pointerOffsets(storage);
  std::cout << AccAllocatorSubclassProbe::defaultPointerIsNull() << '\n';
}
