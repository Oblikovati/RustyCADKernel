#include <Standard_MMgrOpt.hxx>

#include <cstring>
#include <cerrno>
#include <iostream>
#include <string>
#include <sys/mman.h>
#include <unistd.h>

class ExposedMMgrOpt : public Standard_MMgrOpt
{
public:
  using Standard_MMgrOpt::Standard_MMgrOpt;
  size_t* AllocateRaw(size_t& theSize) { return AllocMemory(theSize); }
  void ReleaseRaw(void* thePointer, size_t theSize) { FreeMemory(thePointer, theSize); }
};

int main()
{
  ExposedMMgrOpt aMapped(true, true, 16, 100, 40000);
  const size_t aPageSize = static_cast<size_t>(getpagesize());
  size_t aMappedSize = aPageSize + 1;
  void* aMappedBlock = aMapped.AllocateRaw(aMappedSize);
  const bool isMappedAligned = aMappedSize == 2 * aPageSize;
  const bool isMappedZeroed = std::memcmp(aMappedBlock,
                                          std::string(aMappedSize, '\0').data(),
                                          aMappedSize) == 0;
  aMapped.ReleaseRaw(aMappedBlock, aMappedSize);

  size_t aPartialSize = 2 * aPageSize + 1;
  auto* aPartialBlock = reinterpret_cast<unsigned char*>(aMapped.AllocateRaw(aPartialSize));
  aPartialBlock[2 * aPageSize] = 0x5a;
  aMapped.ReleaseRaw(aPartialBlock, aPageSize + 1);
  const bool isTailStillMapped = aPartialBlock[2 * aPageSize] == 0x5a;
  unsigned char aResidency = 0;
  errno = 0;
  const bool isPrefixUnmapped = mincore(aPartialBlock, aPageSize, &aResidency) != 0
                                && errno == ENOMEM;
  const bool isTailMapped = mincore(aPartialBlock + 2 * aPageSize, aPageSize, &aResidency) == 0;
  aMapped.ReleaseRaw(aPartialBlock + 2 * aPageSize, aPageSize);

  size_t aZeroSize = aPageSize;
  auto* aZeroBlock = reinterpret_cast<unsigned char*>(aMapped.AllocateRaw(aZeroSize));
  aZeroBlock[0] = 0x3c;
  aMapped.ReleaseRaw(aZeroBlock, 0);
  const bool isZeroSizeNoOp = aZeroBlock[0] == 0x3c;
  aMapped.ReleaseRaw(aZeroBlock, aZeroSize);

  ExposedMMgrOpt aMalloc(true, false, 16, 100, 40000);
  size_t aMallocSize = 37;
  void* aMallocBlock = aMalloc.AllocateRaw(aMallocSize);
  const bool isMallocSizePreserved = aMallocSize == 37;
  const bool isMallocZeroed = std::memcmp(aMallocBlock, std::string(37, '\0').data(), 37) == 0;
  aMalloc.ReleaseRaw(aMallocBlock, 0);

  std::cout << "page=" << aPageSize << " mapped_size=" << isMappedAligned
            << " mapped_zero=" << isMappedZeroed << " partial_tail=" << isTailStillMapped
            << " zero_size_noop=" << isZeroSizeNoOp << " malloc_size=" << isMallocSizePreserved
            << " malloc_zero=" << isMallocZeroed << " heap_free_size_ignored=1"
            << " prefix_unmapped=" << isPrefixUnmapped << " tail_mapped=" << isTailMapped << '\n';
}
