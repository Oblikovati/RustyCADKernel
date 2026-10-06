// OCCT 3d097a0: defined unsigned raw request wrapping and zero-count storage no-op.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <limits>
#include <cstring>
struct Arena : NCollection_BaseAllocator {
  size_t allocations=0, frees=0, bytes=0;
  void* AllocateOptimal(size_t requested) override {
    ++allocations;bytes+=requested;
    return NCollection_BaseAllocator::AllocateOptimal(requested);
  }
  void Free(void* pointer) override { ++frees;NCollection_BaseAllocator::Free(pointer); }
};
struct Engine : NCollection_DynamicArray<int> {
  using NCollection_DynamicArray<int>::NCollection_DynamicArray;
  using NCollection_DynamicArray<int>::allocateBlock;
  using NCollection_DynamicArray<int>::deallocateBlock;
  using NCollection_DynamicArray<int>::ensureStorageForIndex;
  using NCollection_DynamicArray<int>::ensureBlockCount;
  using NCollection_DynamicArray<int>::blockCount;
  using NCollection_DynamicArray<int>::blockAddress;
};
static size_t byteChecksum(unsigned char* pointer,size_t bytes) {
  size_t checksum=0;
  for(size_t i=0;i<bytes;++i) { pointer[i]=0x5a;checksum+=pointer[i]; }
  return checksum;
}
static void allocationRow(size_t stride,size_t itemSize) {
  occ::handle<Arena> arena=new Arena;Engine array(stride,arena);
  auto* pointer=static_cast<unsigned char*>(array.allocateBlock(itemSize));
  const size_t checksum=byteChecksum(pointer,arena->bytes);
  array.deallocateBlock(pointer);
  std::cout<<"raw "<<stride<<' '<<itemSize<<' '<<arena->bytes<<' '<<arena->allocations<<' '
           <<arena->frees<<' '<<array.Size()<<' '<<array.blockCount()<<' '<<checksum<<'\n';
}
static void noopRow(int used,size_t reserved) {
  occ::handle<Arena> arena=new Arena;Engine array(size_t(1),arena);
  for(int value=1;value<=used;++value) array.Append(value);
  array.ensureBlockCount(reserved,sizeof(int));
  const void* first=array.blockCount()?array.blockAddress(0):nullptr;
  const size_t before=arena->allocations;
  array.ensureStorageForIndex(std::numeric_limits<size_t>::max(),sizeof(int));
  int sum=0;for(int value : array) sum+=value;
  std::cout<<"noop "<<used<<' '<<reserved<<' '<<array.Size()<<' '<<array.blockCount()<<' '
           <<arena->allocations-before<<' '<<(first==(array.blockCount()?array.blockAddress(0):nullptr))<<' '<<sum<<'\n';
}
int main() {
  const size_t maximum=std::numeric_limits<size_t>::max();
  allocationRow(1,24);allocationRow(2,maximum/2+1+4);
  allocationRow(4,maximum/4+1+4);allocationRow(8,maximum/8+1+4);
  allocationRow(256,maximum/256+2);allocationRow(2,maximum/2+1);
  allocationRow(maximum,7);
  noopRow(0,0);noopRow(0,3);noopRow(1,3);noopRow(3,3);
}
