#include <NCollection_AccAllocator.hxx>

#include <cstdint>
#include <iostream>

class AccAllocatorBlockProbe : public NCollection_AccAllocator
{
public:
  using Block = NCollection_AccAllocator::Block;
  using AlignedSize = NCollection_AccAllocator::AlignedSize;

  static void run()
  {
    alignas(16) uint8_t storage[32] = {};
    Block aPrevious(storage, 4);
    Block aBlock(storage + 4, 24, &aPrevious);
    std::cout << aBlock.FreeSize() << ',' << aBlock.allocCount << ','
              << aBlock.IsEmpty() << ',' << (aBlock.prevBlock == &aPrevious) << '\n';
    aBlock.SetFreeSize(13);
    std::cout << aBlock.FreeSize() << '\n';
    const uintptr_t anOffset = reinterpret_cast<uintptr_t>(static_cast<uint8_t*>(aBlock.Allocate(AlignedSize(5))))
                             - reinterpret_cast<uintptr_t>(aBlock.address);
    std::cout << anOffset << ',' << aBlock.FreeSize() << ',' << aBlock.allocCount << ','
              << aBlock.IsEmpty() << '\n';
    aBlock.Free();
    std::cout << aBlock.allocCount << ',' << aBlock.IsEmpty() << '\n';
  }
};

int main()
{
  AccAllocatorBlockProbe::run();
}
