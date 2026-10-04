#include <Standard_MMgrOpt.hxx>

#include <cstring>
#include <iostream>
#include <string>
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

  size_t aPartialSize = aPageSize + 1;
  auto* aPartialBlock = reinterpret_cast<unsigned char*>(aMapped.AllocateRaw(aPartialSize));
  aPartialBlock[aPageSize] = 0x5a;
  aMapped.ReleaseRaw(aPartialBlock, aPageSize);
  const bool isTailStillMapped = aPartialBlock[aPageSize] == 0x5a;
  aMapped.ReleaseRaw(aPartialBlock + aPageSize, aPageSize);

  ExposedMMgrOpt aMalloc(true, false, 16, 100, 40000);
  size_t aMallocSize = 37;
  void* aMallocBlock = aMalloc.AllocateRaw(aMallocSize);
  const bool isMallocSizePreserved = aMallocSize == 37;
  const bool isMallocZeroed = std::memcmp(aMallocBlock, std::string(37, '\0').data(), 37) == 0;
  aMalloc.ReleaseRaw(aMallocBlock, aMallocSize);

  std::cout << "page=" << aPageSize << " mapped_size=" << isMappedAligned
            << " mapped_zero=" << isMappedZeroed << " partial_tail=" << isTailStillMapped
            << " malloc_size=" << isMallocSizePreserved << " malloc_zero=" << isMallocZeroed
            << '\n';
}
