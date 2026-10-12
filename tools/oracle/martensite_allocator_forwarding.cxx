// OCCT pinned forwarding bodies: Heap Optimal calls Allocate; Inc Allocate calls Optimal.
#include <NCollection_HeapAllocator.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
class HeapForwardingProbe : public NCollection_HeapAllocator {
public:
  void* Allocate(const size_t size) override {
    std::cout << "heap-allocate " << size << " " << GetRefCount() << '\n';
    return NCollection_HeapAllocator::Allocate(size);
  }
};
class IncForwardingProbe : public NCollection_IncAllocator {
public:
  void* AllocateOptimal(const size_t size) override {
    std::cout << "inc-optimal " << size << " " << GetRefCount() << '\n';
    return NCollection_IncAllocator::AllocateOptimal(size);
  }
};
int main() {
  Handle(HeapForwardingProbe) heap = new HeapForwardingProbe;
  Handle(NCollection_BaseAllocator) resource = heap;
  resource->Free(resource->AllocateOptimal(16));
  Handle(IncForwardingProbe) inc = new IncForwardingProbe;
  resource = inc;
  resource->Free(resource->Allocate(16));
  return 0;
}
